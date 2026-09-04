//! code based on [tonic/examples/src/tower/client.rs at master · hyperium/tonic · GitHub](https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs)
use http::{Request, Response};
use opentelemetry::Context as OtelContext;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http as otel_http;

pub type Filter = fn(&str) -> bool;

/// layer for grpc (tonic client):
///
/// - propagate `OpenTelemetry` context (`trace_id`, ...) to server
/// - create a Span for `OpenTelemetry` (and tracing) on call
///
/// `OpenTelemetry` context are extracted frim tracing's span.
#[derive(Default, Debug, Clone)]
pub struct OtelGrpcLayer {
    filter: Option<Filter>,
}

// add a builder like api
impl OtelGrpcLayer {
    #[must_use]
    pub fn filter(self, filter: Filter) -> Self {
        OtelGrpcLayer {
            filter: Some(filter),
        }
    }
}

impl<S> Layer<S> for OtelGrpcLayer {
    /// The wrapped service
    type Service = OtelGrpcService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OtelGrpcService {
            inner,
            filter: self.filter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OtelGrpcService<S> {
    inner: S,
    filter: Option<Filter>,
}

impl<S, B, B2> Service<Request<B>> for OtelGrpcService<S>
where
    S: Service<Request<B>, Response = Response<B2>> + Clone + Send + 'static,
    //S::Future: Send + 'static,
    S::Error: std::error::Error,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    // #[allow(clippy::type_complexity)]
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    //type Future = futures_core::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
    //type Future = Pin<Box<S::Future>>;
    // type Future = S::Future;
    //type Future = Inspect<S::Future, Box<dyn FnOnce(S::Response)>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use tracing_opentelemetry::{OpenTelemetrySpanExt, SetParentError};
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        // let clone = self.inner.clone();
        // let mut inner = std::mem::replace(&mut self.inner, clone);
        let req = req;
        let (span, fallback_context) = if self.filter.is_none_or(|f| f(req.uri().path())) {
            let span = otel_http::grpc_server::make_span_from_request(&req);
            let extracted_context = otel_http::extract_context(req.headers());
            let fallback_context = match span.set_parent(extracted_context.clone()) {
                Ok(()) => None,
                Err(SetParentError::SpanDisabled) => Some(extracted_context),
                Err(error @ (SetParentError::LayerNotFound | SetParentError::AlreadyStarted)) => {
                    tracing::warn!(?error, "can not set parent trace_id to span");
                    None
                }
            };
            (span, fallback_context)
        } else {
            (tracing::Span::none(), None)
        };
        let future = {
            let _context_guard = fallback_context.clone().map(OtelContext::attach);
            let _enter = span.enter();
            self.inner.call(req)
        };
        ResponseFuture {
            inner: future,
            span,
            fallback_context,
        }
    }
}

pin_project! {
    /// Response future for [`Trace`].
    ///
    /// [`Trace`]: super::Trace
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        pub(crate) fallback_context: Option<OtelContext>,
        // pub(crate) start: Instant,
    }
}

impl<Fut, ResBody, Error> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response<ResBody>, Error>>,
    // Require that the inner service's error can be converted into a `BoxError`.
    //Error: Into<BoxError>,
    Error: std::error::Error,
{
    type Output = Result<Response<ResBody>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _context_guard = this
            .fallback_context
            .as_ref()
            .map(|context| context.clone().attach());
        let _guard = this.span.enter();
        let result = futures_util::ready!(this.inner.poll(cx));
        otel_http::grpc::update_span_from_response_or_error(this.span, &result);
        Poll::Ready(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing_tracing_opentelemetry::FakeEnvironment;
    use tower::service_fn;

    #[tokio::test]
    async fn disabled_request_span_preserves_remote_context() {
        const TRACE_ID: &str = "b2611246a58fd7ea623d2264c5a1e226";
        const PARENT_SPAN_ID: &str = "b2c9b811f2f424af";

        let mut fake_env = FakeEnvironment::setup_with_filter("warn").await;
        {
            let inner = service_fn(|_req: Request<()>| async {
                let span = tracing::warn_span!("enabled child span");
                let _guard = span.enter();
                Ok::<_, std::io::Error>(Response::new(()))
            });
            let mut svc = OtelGrpcLayer::default().layer(inner);
            let req = Request::builder()
                .header("traceparent", format!("00-{TRACE_ID}-{PARENT_SPAN_ID}-01"))
                .body(())
                .unwrap();
            let _res = svc.call(req).await.unwrap();
        }

        let (tracing_events, otel_spans) = fake_env.collect_traces().await;
        assert_eq!(tracing_events.len(), 2);
        assert_eq!(otel_spans.len(), 1);
        assert_eq!(otel_spans[0].name, "enabled child span");
        assert_eq!(otel_spans[0].trace_id, TRACE_ID);
        assert_eq!(otel_spans[0].parent_span_id, PARENT_SPAN_ID);
    }
}
