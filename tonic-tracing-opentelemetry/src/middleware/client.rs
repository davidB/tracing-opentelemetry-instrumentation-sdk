//! code based on [tonic/examples/src/tower/client.rs at master · hyperium/tonic · GitHub](https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs)
use super::extract_service_method;
use http::{header, Request, Response};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tonic::body::BoxBody;
use tonic::transport::Body;
use tower::{Layer, Service};
use tracing::field::Empty;

/// layer for grpc (tonic client):
///
/// - propagate OpenTelemetry context (trace_id,...) to server
/// - create a Span for OpenTelemetry (and tracing) on call
///
/// OpenTelemetry context are extracted frim tracing's span.
#[derive(Default)]
pub struct OtelGrpcLayer;

impl<S> Layer<S> for OtelGrpcLayer {
    /// The wrapped service
    type Service = OtelGrpcService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OtelGrpcService { inner }
    }
}

pub struct OtelGrpcService<S> {
    inner: S,
}

impl<S> Service<Request<BoxBody>> for OtelGrpcService<S>
where
    S: Service<Request<BoxBody>, Response = Response<Body>, Error = tonic::transport::Error>
        + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    #[allow(clippy::type_complexity)]
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<BoxBody>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let mut req = req;
        // TODO set the attributes following specification
        // [opentelemetry-specification/specification/trace/semantic\_conventions/rpc.md at main · open-telemetry/opentelemetry-specification · GitHub](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md)
        let http_target = req.uri().path();
        let (service, method) = extract_service_method(http_target);
        let host = req
            .headers()
            .get(header::HOST)
            .map_or(req.uri().host(), |h| h.to_str().ok())
            .unwrap_or("");
        let _span = tracing::info_span!(
            target: "otel::tracing",
            "grpc request",
            otel.name = %http_target,
            otel.kind = ?opentelemetry::trace::SpanKind::Client,
            otel.status_code = Empty,
            rpc.system ="grpc",
            rpc.service = %service,
            rpc.method = %method,
            server.address = %host,
        )
        .entered();
        dbg!(init_tracing_opentelemetry::find_current_trace_id());
        inject_context(req.headers_mut());
        dbg!(req.headers());
        // Box::pin(async move {
        //     let response = inner.call(req).await?;
        //     Ok(response)
        // })
        inner.call(req)
    }
}

fn inject_context(headers: &mut http::HeaderMap) {
    use opentelemetry_http::HeaderInjector;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let mut injector = HeaderInjector(headers);
    // let context = opentelemetry::Context::current();
    // OpenTelemetry Context is propagation inside code is done via tracing crate
    let context = tracing::Span::current().context();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        dbg!(propagator);
        propagator.inject_context(&context, &mut injector)
    })
}
