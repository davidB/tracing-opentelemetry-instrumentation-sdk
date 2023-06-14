//
//! OpenTelemetry middleware.
//!
//! See [`opentelemetry_tracing_layer`] for more details.

use http::{header, Request};
use opentelemetry::trace::{TraceContextExt, TraceId};
use std::time::Duration;
use tower_http::{
    classify::{GrpcErrorsAsFailures, GrpcFailureClass, SharedClassifier},
    trace::{MakeSpan, OnBodyChunk, OnEos, OnFailure, OnRequest, OnResponse, TraceLayer},
};
use tracing::{field::Empty, Span};

pub type Filter = fn(&str, &str) -> bool;

/// OpenTelemetry tracing middleware for gRPC server.
pub fn opentelemetry_tracing_layer_server() -> TraceLayer<
    SharedClassifier<GrpcErrorsAsFailures>,
    OtelMakeSpan,
    OtelOnRequest,
    OtelOnResponse,
    OtelOnBodyChunk,
    OtelOnEos,
    OtelOnFailure,
> {
    TraceLayer::new_for_grpc()
        .make_span_with(OtelMakeSpan { filter: None })
        .on_request(OtelOnRequest)
        .on_response(OtelOnResponse)
        .on_body_chunk(OtelOnBodyChunk)
        .on_eos(OtelOnEos)
        .on_failure(OtelOnFailure)
}

pub trait WithFilter {
    fn with_filter(self, filter: Filter) -> Self;
}

impl<M, OnRequest, OnResponse, OnBodyChunk, OnEos, OnFailure> WithFilter
    for TraceLayer<M, OtelMakeSpan, OnRequest, OnResponse, OnBodyChunk, OnEos, OnFailure>
{
    fn with_filter(self, filter: Filter) -> Self {
        self.make_span_with(OtelMakeSpan {
            filter: Some(filter),
        })
    }
}

// HACK duplicate tracing-opentelemetry to be able to attach trace_id, span_id without being a parent context
#[allow(dead_code, clippy::type_complexity)]
pub(crate) struct WithContext(
    fn(
        &tracing::Dispatch,
        &tracing::span::Id,
        f: &mut dyn FnMut(
            &mut tracing_opentelemetry::OtelData,
            &dyn tracing_opentelemetry::PreSampledTracer,
        ),
    ),
);

#[allow(dead_code)]
impl WithContext {
    // This function allows a function to be called in the context of the
    // "remembered" subscriber.
    pub(crate) fn with_context(
        &self,
        dispatch: &tracing::Dispatch,
        id: &tracing::span::Id,
        mut f: impl FnMut(
            &mut tracing_opentelemetry::OtelData,
            &dyn tracing_opentelemetry::PreSampledTracer,
        ),
    ) {
        (self.0)(dispatch, id, &mut f)
    }
}

#[allow(dead_code)]
fn with_span_context(tspan: &tracing::Span, cx: opentelemetry::trace::SpanContext) {
    //tspan.context().with_span(span)
    if cx.is_valid() {
        let mut cx = Some(cx);
        tspan.with_subscriber(move |(id, subscriber)| {
            if let Some(get_context) = subscriber.downcast_ref::<WithContext>() {
                get_context.with_context(subscriber, id, move |data, _tracer| {
                    if let Some(cx) = cx.take() {
                        data.builder = data
                            .builder
                            .clone()
                            .with_span_id(cx.span_id())
                            .with_trace_id(cx.trace_id());
                    }
                });
            }
        });
    }
}

/// A [`MakeSpan`] that creates tracing spans using [OpenTelemetry's conventional field names][otel] for gRPC services.
///
/// [otel]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md
#[derive(Clone, Copy, Debug, Default)]
pub struct OtelMakeSpan {
    filter: Option<Filter>,
}

impl<B> MakeSpan<B> for OtelMakeSpan {
    fn make_span(&mut self, req: &Request<B>) -> Span {
        let http_target = req.uri().path();
        let (service, method) = extract_service_method(http_target);

        if let Some(filter) = self.filter {
            if !filter(service, method) {
                return Span::none();
            }
        }

        let user_agent = req
            .headers()
            .get(header::USER_AGENT)
            .map_or("", |h| h.to_str().unwrap_or(""));

        let host = req
            .headers()
            .get(header::HOST)
            .map_or("", |h| h.to_str().unwrap_or(""));

        let (trace_id, otel_context) =
            create_context_with_trace(extract_remote_context(req.headers()));
        // based on https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md#grpc
        let span = tracing::info_span!(
            "grpc request",
            rpc.system ="grpc",
            rpc.service = %service,
            rpc.method = %method,
            otel.name = %http_target, // Convention in gRPC tracing.
            // client.address = %client_ip,
            // http.flavor = %http_flavor(req.version()),
            // http.grpc_status = Empty,
            server.address = %host,
            http.user_agent = %user_agent,
            otel.kind = %"server", //opentelemetry::trace::SpanKind::Server
            otel.status_code = Empty,
            trace_id = %trace_id,
        );
        match otel_context {
            OtelContext::Remote(cx) => {
                tracing_opentelemetry::OpenTelemetrySpanExt::set_parent(&span, cx)
            }
            OtelContext::Local(cx) => {
                tracing_opentelemetry::OpenTelemetrySpanExt::add_link(&span, cx)
            }
        }
        span
    }
}

fn extract_service_method(path: &str) -> (&str, &str) {
    let mut parts = path.split('/').filter(|x| !x.is_empty());
    let service = parts.next().unwrap_or_default();
    let method = parts.next().unwrap_or_default();
    (service, method)
}

// If remote request has no span data the propagator defaults to an unsampled context
fn extract_remote_context(headers: &http::HeaderMap) -> opentelemetry::Context {
    struct HeaderExtractor<'a>(&'a http::HeaderMap);

    impl<'a> opentelemetry::propagation::Extractor for HeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|value| value.to_str().ok())
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|value| value.as_str()).collect()
        }
    }
    let extractor = HeaderExtractor(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
}

enum OtelContext {
    Remote(opentelemetry::Context),
    Local(opentelemetry::trace::SpanContext),
}

//HACK create a context with a trace_id (if not set) before call to
// `tracing_opentelemetry::OpenTelemetrySpanExt::set_parent`
// else trace_id is defined too late and the `info_span` log `trace_id: ""`
// Use the default global tracer (named "") to start the trace
fn create_context_with_trace(remote_context: opentelemetry::Context) -> (TraceId, OtelContext) {
    if !remote_context.span().span_context().is_valid() {
        // create a fake remote context but with a fresh new trace_id
        use opentelemetry::sdk::trace::IdGenerator;
        use opentelemetry::sdk::trace::RandomIdGenerator;
        use opentelemetry::trace::SpanContext;
        //let trace_id = RandomIdGenerator::default().new_trace_id();
        let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0]);
        let span_id = RandomIdGenerator::default().new_span_id();
        let new_span_context = SpanContext::new(
            trace_id,
            span_id,
            remote_context.span().span_context().trace_flags(),
            false,
            remote_context.span().span_context().trace_state().clone(),
        );
        (trace_id, OtelContext::Local(new_span_context))
    } else {
        let remote_span = remote_context.span();
        let span_context = remote_span.span_context();
        let trace_id = span_context.trace_id();
        (trace_id, OtelContext::Remote(remote_context))
    }
}

/// Callback that [`Trace`] will call when it receives a request.
///
/// [`Trace`]: tower_http::trace::Trace
#[derive(Clone, Copy, Debug)]
pub struct OtelOnRequest;

impl<B> OnRequest<B> for OtelOnRequest {
    #[inline]
    fn on_request(&mut self, _request: &Request<B>, _span: &Span) {}
}

/// Callback that [`Trace`] will call when it receives a response.
///
/// [`Trace`]: tower_http::trace::Trace
#[derive(Clone, Copy, Debug)]
pub struct OtelOnResponse;

impl<B> OnResponse<B> for OtelOnResponse {
    fn on_response(self, response: &http::Response<B>, _latency: Duration, span: &Span) {
        let status = response.status().as_u16().to_string();
        span.record("http.status_code", &tracing::field::display(status));

        // assume there is no error, if there is `OtelOnFailure` will be called and override this
        span.record("otel.status_code", "OK");
    }
}

/// Callback that [`Trace`] will call when the response body produces a chunk.
///
/// [`Trace`]: tower_http::trace::Trace
#[derive(Clone, Copy, Debug)]
pub struct OtelOnBodyChunk;

impl<B> OnBodyChunk<B> for OtelOnBodyChunk {
    #[inline]
    fn on_body_chunk(&mut self, _chunk: &B, _latency: Duration, _span: &Span) {}
}

/// Callback that [`Trace`] will call when a streaming response completes.
///
/// [`Trace`]: tower_http::trace::Trace
#[derive(Clone, Copy, Debug)]
pub struct OtelOnEos;

impl OnEos for OtelOnEos {
    #[inline]
    fn on_eos(self, _trailers: Option<&http::HeaderMap>, _stream_duration: Duration, _span: &Span) {
    }
}

/// Callback that [`Trace`] will call when a response or end-of-stream is classified as a failure.
///
/// [`Trace`]: tower_http::trace::Trace
#[derive(Clone, Copy, Debug)]
pub struct OtelOnFailure;

impl OnFailure<GrpcFailureClass> for OtelOnFailure {
    fn on_failure(&mut self, failure: GrpcFailureClass, _latency: Duration, span: &Span) {
        match failure {
            GrpcFailureClass::Code(code) => {
                span.record("http.grpc_status", code);
            }
            GrpcFailureClass::Error(_) => {
                span.record("http.grpc_status", 1);
            }
        }
    }
}
/*

// FIXME Experimentation to allow to apply layer only on a single service like in
// ```rust
//     Server::builder()
//         .add_service(health_service)
//         .add_service(reflection_service)
//         //.add_service(GreeterServer::new(greeter))
//         .add_service(traced(GreeterServer::new(greeter)))
//         .serve(addr)
//         .await?;
// ```
type ServiceWithTrace<S> = Trace<
    S,
    SharedClassifier<GrpcErrorsAsFailures>,
    OtelMakeSpan,
    OtelOnRequest,
    OtelOnResponse,
    OtelOnBodyChunk,
    OtelOnEos,
    OtelOnFailure,
>;

pub fn traced<S, Req>(service: S) -> TracedService<S>
where
    S: Service<Req>,
    S: Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError> + Send,
{
    TracedService(
        ServiceBuilder::new()
            .layer(opentelemetry_tracing_layer_server())
            .service(service),
        //opentelemetry_tracing_layer_server().layer(service),
    )
}

/// A newtype wrapper around [`TraceLayer`] to allow
/// `traced` to implement the [`NamedService`] trait.
#[derive(Debug, Clone)]
pub struct TracedService<S>(ServiceWithTrace<S>);

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TracedService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ReqBody: Body,
    ResBody: Body,
    ResBody::Error: std::fmt::Display + 'static,
    S::Error: std::fmt::Display + 'static,
{
    type Response = Response<
        ResponseBody<
            ResBody,
            tower_http::classify::GrpcEosErrorsAsFailures, //GrpcErrorsAsFailures::ClassifyEos,
            OtelOnBodyChunk,
            OtelOnEos,
            OtelOnFailure,
        >,
    >;
    type Error = S::Error;
    type Future = ResponseFuture<
        S::Future,
        GrpcErrorsAsFailures,
        OtelOnResponse,
        OtelOnBodyChunk,
        OtelOnEos,
        OtelOnFailure,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        self.0.call(req)
    }
}

impl<S> NamedService for TracedService<S>
where
    S: NamedService,
{
    const NAME: &'static str = S::NAME;
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::*;
    use axum::{body::Body, response::Response, routing::post, Router};
    use http::{Request, StatusCode};
    use rstest::*;
    use testing_tracing_opentelemetry::*;

    #[rstest]
    #[case("", "", "")]
    #[case("/", "", "")]
    #[case("//", "", "")]
    #[case("/grpc.health.v1.Health/Check", "grpc.health.v1.Health", "Check")]
    fn test_extract_service_method(
        #[case] path: &str,
        #[case] service: &str,
        #[case] method: &str,
    ) {
        check!(extract_service_method(path) == (service, method));
    }

    #[rstest]
    #[case("grpc_status_code_on_close_for_ok", "/module.service/endpoint1", &[])]
    #[tokio::test(flavor = "multi_thread")]
    async fn check_span_event_grpc(
        #[case] name: &str,
        #[case] uri: &str,
        #[case] headers: &[(&str, &str)],
    ) {
        let fake_env = FakeEnvironment::setup().await;
        {
            use axum::body::HttpBody as _;
            use tower::{Service, ServiceExt};
            let mut svc: Router = Router::new()
                .route(
                    "/module.service/endpoint1",
                    post(|| async {
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("grpc-status", 2)
                            .body(Body::empty())
                            .unwrap()
                    }),
                )
                .layer(opentelemetry_tracing_layer_server());
            let mut builder = Request::builder();
            for (key, value) in headers.iter() {
                builder = builder.header(*key, *value);
            }
            builder = builder.method("POST");
            let req = builder.uri(uri).body(Body::empty()).unwrap();

            let mut res = svc.ready().await.unwrap().call(req).await.unwrap();

            while res.data().await.is_some() {}
            res.trailers().await.unwrap();
            drop(res);
        }
        let (tracing_events, otel_spans) = fake_env.collect_traces().await;
        assert_trace(name, tracing_events, otel_spans, false);
    }
}
