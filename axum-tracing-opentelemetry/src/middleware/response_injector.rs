use futures_core::future::BoxFuture;
use http::{Request, Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing_opentelemetry_instrumentation_sdk as otel;
use tracing_opentelemetry_instrumentation_sdk::http as otel_http;

#[deprecated(
    since = "0.12.0",
    note = "keep for transition, replaced by OtelInResponseLayer"
)]
#[must_use]
pub fn response_with_trace_layer() -> OtelInResponseLayer {
    OtelInResponseLayer {}
}

#[derive(Default, Debug, Clone)]
pub struct OtelInResponseLayer;

impl<S> Layer<S> for OtelInResponseLayer {
    type Service = OtelInResponseService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelInResponseService { inner }
    }
}

#[derive(Default, Debug, Clone)]
pub struct OtelInResponseService<S> {
    inner: S,
}

impl<S, B, B2> Service<Request<B>> for OtelInResponseService<S>
where
    S: Service<Request<B>, Response = Response<B2>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[allow(unused_mut)]
    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let future = self.inner.call(request);

        Box::pin(async move {
            let mut response = future.await?;
            // inject the trace context into the response (optional but useful for debugging and client)
            otel_http::inject_context(&otel::find_current_context(), response.headers_mut());
            Ok(response)
        })
    }
}
