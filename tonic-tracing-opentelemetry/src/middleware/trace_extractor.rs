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

/// OpenTelemetry tracing middleware for gRPC server.
pub fn opentelemetry_tracing_layer_server() -> TraceLayer<
    SharedClassifier<GrpcErrorsAsFailures>,
    OtelMakeGrpcSpan,
    OtelOnRequest,
    OtelOnResponse,
    OtelOnBodyChunk,
    OtelOnEos,
    OtelOnGrpcFailure,
> {
    TraceLayer::new_for_grpc()
        .make_span_with(OtelMakeGrpcSpan)
        .on_request(OtelOnRequest)
        .on_response(OtelOnResponse)
        .on_body_chunk(OtelOnBodyChunk)
        .on_eos(OtelOnEos)
        .on_failure(OtelOnGrpcFailure)
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
#[derive(Clone, Copy, Debug)]
pub struct OtelMakeGrpcSpan;

impl<B> MakeSpan<B> for OtelMakeGrpcSpan {
    fn make_span(&mut self, req: &Request<B>) -> Span {
        let user_agent = req
            .headers()
            .get(header::USER_AGENT)
            .map_or("", |h| h.to_str().unwrap_or(""));

        let host = req
            .headers()
            .get(header::HOST)
            .map_or("", |h| h.to_str().unwrap_or(""));

        let http_target = req.uri().path();

        let (trace_id, otel_context) =
            create_context_with_trace(extract_remote_context(req.headers()));
        // based on https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md#grpc
        let span = tracing::info_span!(
            "grpc request",
            rpc.system ="grpc",
            rpc.service = "",
            rpc.method = "",
            otel.name = %http_target, // Convention in gRPC tracing.
            // http.client_ip = %client_ip,
            // http.flavor = %http_flavor(req.version()),
            // http.grpc_status = Empty,
            http.host = %host,
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
pub struct OtelOnGrpcFailure;

impl OnFailure<GrpcFailureClass> for OtelOnGrpcFailure {
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
