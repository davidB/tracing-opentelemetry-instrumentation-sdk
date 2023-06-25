//! code based on [tonic/examples/src/tower/client.rs at master · hyperium/tonic · GitHub](https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs)
use http::{Request, Response};
use std::task::{Context, Poll};
use tower::{BoxError, Layer, Service};
use tracing_opentelemetry_instrumentation_sdk::http as otel_http;

/// layer for grpc (tonic client):
///
/// - propagate OpenTelemetry context (trace_id,...) to server
/// - create a Span for OpenTelemetry (and tracing) on call
///
/// OpenTelemetry context are extracted frim tracing's span.
#[derive(Default, Debug, Clone)]
pub struct OtelGrpcLayer {
    filter: Option<Filter>,
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

pub type Filter = fn(&str, &str) -> bool;

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
    #[allow(clippy::type_complexity)]
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
    //type Future = Pin<Box<S::Future>>;
    //type Future = S::Future;
    //type Future = Inspect<S::Future, Box<dyn FnOnce(S::Response)>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let req = req;
        // if let Some(filter) = self.filter {
        //     if !filter(service, method) {
        //         return Span::none();
        //     }
        // }
        let mut span = otel_http::grpc_server::make_span_from_request(&req);
        span.set_parent(otel_http::extract_context(req.headers()));
        // span.enter();
        Box::pin(async move {
            let _ = span.enter();
            let response = inner.call(req).await;
            otel_http::grpc_client::update_span_from_response_or_error(&mut span, &response);
            response
        })
    }
}
