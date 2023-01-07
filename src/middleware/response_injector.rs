use axum::{body::Body, http::Request, response::Response};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

pub fn response_with_trace_layer() -> OtelInResponseLayer {
    OtelInResponseLayer {}
}

#[derive(Clone)]
pub struct OtelInResponseLayer;

impl<S> Layer<S> for OtelInResponseLayer {
    type Service = OtelInResponseMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelInResponseMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct OtelInResponseMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for OtelInResponseMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
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
    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let future = self.inner.call(request);

        Box::pin(async move {
            let mut response: Response = future.await?;
            // inject the trace context into the response (optional but useful for debugging and client)
            inject_context(response.headers_mut());
            Ok(response)
        })
    }
}

fn inject_context(headers: &mut http::HeaderMap) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    struct HeaderInjector<'a>(&'a mut http::HeaderMap);

    impl<'a> opentelemetry::propagation::Injector for HeaderInjector<'a> {
        /// Add a key and value to the underlying data.
        fn set(&mut self, key: &str, value: String) {
            // TODO manage error when failed to convert
            if let Ok(k) = http::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(v) = http::HeaderValue::from_str(&value) {
                    self.0.insert(k, v);
                }
            }
        }
    }
    let mut injector = HeaderInjector(headers);
    // let context = opentelemetry::Context::current();
    // OpenTelemetry Context is propagation inside code is done via tracing crate
    let context = tracing::Span::current().context();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut injector)
    })
}
