//! code based on [tonic/examples/src/tower/client.rs at master · hyperium/tonic · GitHub](https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs)
use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{BoxError, Layer, Service};
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
    S: Service<Request<B>, Response = Response<B2>, Error = BoxError> + Clone + Send + 'static,
    S::Future: Send + 'static,
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
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        // let clone = self.inner.clone();
        // let mut inner = std::mem::replace(&mut self.inner, clone);
        let req = req;
        let span = if self.filter.map_or(true, |f| f(req.uri().path())) {
            let span = otel_http::grpc_server::make_span_from_request(&req);
            span.set_parent(otel_http::extract_context(req.headers()));
            span
        } else {
            tracing::Span::none()
        };
        let future = {
            let _ = span.enter();
            self.inner.call(req)
        };
        ResponseFuture {
            inner: future,
            span,
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
        // pub(crate) start: Instant,
    }
}

impl<Fut, ResBody> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response<ResBody>, BoxError>>,
{
    type Output = Result<Response<ResBody>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = futures_util::ready!(this.inner.poll(cx));
        otel_http::grpc_server::update_span_from_response_or_error(this.span, &result);
        Poll::Ready(result)
    }
}
