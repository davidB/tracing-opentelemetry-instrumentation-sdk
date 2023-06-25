//! code based on [tonic/examples/src/tower/client.rs at master · hyperium/tonic · GitHub](https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs)
use http::{Request, Response};
use std::task::{Context, Poll};
use tonic::client::GrpcService;
use tower::{BoxError, Layer, Service};
use tracing_opentelemetry_instrumentation_sdk::{find_context_from_tracing, http as otel_http};

/// layer for grpc (tonic client):
///
/// - propagate OpenTelemetry context (trace_id,...) to server
/// - create a Span for OpenTelemetry (and tracing) on call
///
/// OpenTelemetry context are extracted frim tracing's span.
#[derive(Default, Debug, Clone)]
pub struct OtelGrpcLayer;

impl<S> Layer<S> for OtelGrpcLayer {
    /// The wrapped service
    type Service = OtelGrpcService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OtelGrpcService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct OtelGrpcService<S> {
    inner: S,
}

impl<S, B, B2> GrpcService<B> for OtelGrpcService<S>
where
    S: GrpcService<B, ResponseBody = B2> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
    B2: tonic::codegen::Body,
{
    type ResponseBody = B2;
    type Error = BoxError;
    #[allow(clippy::type_complexity)]
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Future =
        futures::future::BoxFuture<'static, Result<http::Response<S::ResponseBody>, Self::Error>>;
    // type Future: Future<Output = Result<http::Response<S::ResponseBody>, Self::Error>>;
    //type Future = Pin<Box<S::Future>>;
    //type Future = S::Future;
    //type Future = Inspect<S::Future, Box<dyn FnOnce(S::Response)>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| e.into())
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let mut req = req;
        let mut span = otel_http::grpc_client::make_span_from_request(&req);
        otel_http::inject_context(find_context_from_tracing(&span), req.headers_mut());
        // let _ = span.enter();
        Box::pin(async move {
            let _ = span.enter();
            let response: Result<Response<Self::ResponseBody>, BoxError> =
                inner.call(req).await.map_err(|e| e.into());
            otel_http::grpc_client::update_span_from_response_or_error(&mut span, &response);
            response //.map_err(|e| e.)
        })
    }
}
