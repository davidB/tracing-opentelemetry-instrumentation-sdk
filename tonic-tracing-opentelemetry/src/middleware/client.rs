//! code based on [tonic/examples/src/tower/client.rs at master · hyperium/tonic · GitHub](https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs)
use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    error::Error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tonic::client::GrpcService;
use tower::Layer;
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::{find_context_from_tracing, http as otel_http};

/// layer for grpc (tonic client):
///
/// - propagate `OpenTelemetry` context (`trace_id`,...) to server
/// - create a Span for `OpenTelemetry` (and tracing) on call
///
/// `OpenTelemetry` context are extracted frim tracing's span.
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
    S::Error: Error + 'static,
    B: Send + 'static,
    // B2: tonic::codegen::Body,
    B2: http_body::Body,
{
    type ResponseBody = B2;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    // #[allow(clippy::type_complexity)]
    // type Future =
    //     futures::future::BoxFuture<'static, Result<http::Response<S::ResponseBody>, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx) //.map_err(|e| e.into())
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        // let clone = self.inner.clone();
        // let mut inner = std::mem::replace(&mut self.inner, clone);
        let mut req = req;
        let span = otel_http::grpc_client::make_span_from_request(&req);
        otel_http::inject_context(&find_context_from_tracing(&span), req.headers_mut());
        let future = {
            let _enter = span.enter();
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

impl<Fut, ResBody, E> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response<ResBody>, E>>,
    E: std::error::Error + 'static,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = futures_util::ready!(this.inner.poll(cx));
        otel_http::grpc_client::update_span_from_response_or_error(this.span, &result);
        Poll::Ready(result)
    }
}
