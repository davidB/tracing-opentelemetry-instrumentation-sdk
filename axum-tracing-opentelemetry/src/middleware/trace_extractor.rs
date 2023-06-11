//
//! OpenTelemetry middleware.
//!
//! See [`opentelemetry_tracing_layer`] for more details.

use axum::{
    extract::{ConnectInfo, MatchedPath, OriginalUri},
    response::Response,
};
use http::{header, uri::Scheme, HeaderMap, Method, Request, Version};
use opentelemetry::trace::{TraceContextExt, TraceId};
use std::{borrow::Cow, net::SocketAddr, time::Duration};
use tower_http::{
    classify::{
        GrpcErrorsAsFailures, GrpcFailureClass, ServerErrorsAsFailures, ServerErrorsFailureClass,
        SharedClassifier,
    },
    trace::{MakeSpan, OnBodyChunk, OnEos, OnFailure, OnRequest, OnResponse, TraceLayer},
};
use tracing::{field::Empty, Span};

/// OpenTelemetry tracing middleware.
///
/// This returns a [`TraceLayer`] configured to use [OpenTelemetry's conventional span field
/// names][otel].
///
/// # Span fields
///
/// The following fields will be set on the span:
///
/// - `http.client_ip`: The client's IP address. Requires using
/// [`Router::into_make_service_with_connect_info`]
/// - `http.flavor`: The protocol version used (http 1.1, http 2.0, etc)
/// - `http.host`: The value of the `Host` header
/// - `http.method`: The request method
/// - `http.route`: The matched route
/// - `http.scheme`: The URI scheme used (`HTTP` or `HTTPS`)
/// - `http.status_code`: The response status code
/// - `http.target`: The full request target including path and query parameters
/// - `http.user_agent`: The value of the `User-Agent` header
/// - `otel.kind`: Always `server`
/// - `otel.status_code`: `OK` if the response is success, `ERROR` if it is a 5xx
/// - `trace_id`: The trace id as tracted via the remote span context.
///
/// # Example
///
/// ```
/// use axum::{Router, routing::get, http::Request};
/// use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
/// use std::net::SocketAddr;
/// use tower::ServiceBuilder;
///
/// let app = Router::new()
///     .route("/", get(|| async {}))
///     .layer(opentelemetry_tracing_layer());
///
/// # async {
/// axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
///     // we must use `into_make_service_with_connect_info` for `opentelemetry_tracing_layer` to
///     // access the client ip
///     .serve(app.into_make_service_with_connect_info::<SocketAddr>())
///     .await
///     .expect("server failed");
/// # };
/// ```
///
/// # Complete example
///
/// See the "opentelemetry-jaeger" example for a complete setup that includes an OpenTelemetry
/// pipeline sending traces to jaeger.
///
/// [otel]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md
/// [`Router::into_make_service_with_connect_info`]: axum::Router::into_make_service_with_connect_info
pub fn opentelemetry_tracing_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    OtelMakeSpan,
    OtelOnRequest,
    OtelOnResponse,
    OtelOnBodyChunk,
    OtelOnEos,
    OtelOnFailure,
> {
    TraceLayer::new_for_http()
        .make_span_with(OtelMakeSpan)
        .on_request(OtelOnRequest)
        .on_response(OtelOnResponse)
        .on_body_chunk(OtelOnBodyChunk)
        .on_eos(OtelOnEos)
        .on_failure(OtelOnFailure)
}

/// OpenTelemetry tracing middleware for gRPC.
pub fn opentelemetry_tracing_layer_grpc() -> TraceLayer<
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

/// A [`MakeSpan`] that creates tracing spans using [OpenTelemetry's conventional field names][otel].
///
/// [otel]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md
#[derive(Clone, Copy, Debug)]
pub struct OtelMakeSpan;

impl<B> MakeSpan<B> for OtelMakeSpan {
    fn make_span(&mut self, req: &Request<B>) -> Span {
        let user_agent = req
            .headers()
            .get(header::USER_AGENT)
            .map_or("", |h| h.to_str().unwrap_or(""));

        let host = req
            .headers()
            .get(header::HOST)
            .map_or("", |h| h.to_str().unwrap_or(""));

        let scheme = req
            .uri()
            .scheme()
            .map_or_else(|| "HTTP".into(), http_scheme);

        let http_route = req
            .extensions()
            .get::<MatchedPath>()
            .map_or_else(|| "", |mp| mp.as_str())
            .to_owned();

        let uri = if let Some(uri) = req.extensions().get::<OriginalUri>() {
            uri.0.clone()
        } else {
            req.uri().clone()
        };
        let http_target = uri
            .path_and_query()
            .map(|path_and_query| path_and_query.to_string())
            .unwrap_or_else(|| uri.path().to_owned());

        let client_ip = parse_x_forwarded_for(req.headers())
            .or_else(|| {
                req.extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ConnectInfo(client_ip)| Cow::from(client_ip.to_string()))
            })
            .unwrap_or_default();
        let http_method_v = http_method(req.method());
        let name = format!("{http_method_v} {http_route}").trim().to_string();
        let remote_context = extract_remote_context(req.headers());
        let (trace_id, otel_context) = create_context_with_trace(remote_context.clone());
        let span = tracing::info_span!(
            "HTTP request",
            otel.name= %name,
            http.client_ip = %client_ip,
            http.flavor = %http_flavor(req.version()),
            http.host = %host,
            http.method = %http_method_v,
            http.route = %http_route,
            http.scheme = %scheme,
            http.status_code = Empty,
            http.target = %http_target,
            http.user_agent = %user_agent,
            otel.kind = %"server", //opentelemetry::trace::SpanKind::Server
            otel.status_code = Empty,
            trace_id = %trace_id,
        );
        match otel_context {
            OtelContext::Remote(cx) => {
                tracing_opentelemetry::OpenTelemetrySpanExt::set_parent(&span, cx)
            }
            // OtelContext::Local(cx) => with_span_context(&span, cx),
            OtelContext::Local(cx) => {
                remote_context.with_remote_span_context(cx);
                tracing_opentelemetry::OpenTelemetrySpanExt::set_parent(&span, remote_context)
            }
        }
        span
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

        let scheme = req
            .uri()
            .scheme()
            .map_or_else(|| "HTTP".into(), http_scheme);

        let http_route = req
            .extensions()
            .get::<MatchedPath>()
            .map_or("", |mp| mp.as_str())
            .to_owned();

        let uri = if let Some(uri) = req.extensions().get::<OriginalUri>() {
            uri.0.clone()
        } else {
            req.uri().clone()
        };
        let http_target = uri
            .path_and_query()
            .map(|path_and_query| path_and_query.to_string())
            .unwrap_or_else(|| uri.path().to_owned());

        let client_ip = parse_x_forwarded_for(req.headers())
            .or_else(|| {
                req.extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ConnectInfo(client_ip)| Cow::from(client_ip.to_string()))
            })
            .unwrap_or_default();
        let http_method_v = http_method(req.method());
        let (trace_id, otel_context) =
            create_context_with_trace(extract_remote_context(req.headers()));
        let span = tracing::info_span!(
            "grpc request",
            otel.name = %http_target, // Convetion in gRPC tracing.
            http.client_ip = %client_ip,
            http.flavor = %http_flavor(req.version()),
            http.grpc_status = Empty,
            http.host = %host,
            http.method = %http_method_v,
            http.route = %http_route,
            http.scheme = %scheme,
            http.status_code = Empty,
            http.target = %http_target,
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

fn parse_x_forwarded_for(headers: &HeaderMap) -> Option<Cow<'_, str>> {
    let value = headers.get("x-forwarded-for")?;
    let value = value.to_str().ok()?;
    let mut ips = value.split(',');
    Some(ips.next()?.trim().into())
}

fn http_method(method: &Method) -> Cow<'static, str> {
    match method {
        &Method::CONNECT => "CONNECT".into(),
        &Method::DELETE => "DELETE".into(),
        &Method::GET => "GET".into(),
        &Method::HEAD => "HEAD".into(),
        &Method::OPTIONS => "OPTIONS".into(),
        &Method::PATCH => "PATCH".into(),
        &Method::POST => "POST".into(),
        &Method::PUT => "PUT".into(),
        &Method::TRACE => "TRACE".into(),
        other => other.to_string().into(),
    }
}

fn http_flavor(version: Version) -> Cow<'static, str> {
    match version {
        Version::HTTP_09 => "0.9".into(),
        Version::HTTP_10 => "1.0".into(),
        Version::HTTP_11 => "1.1".into(),
        Version::HTTP_2 => "2.0".into(),
        Version::HTTP_3 => "3.0".into(),
        other => format!("{other:?}").into(),
    }
}

fn http_scheme(scheme: &Scheme) -> Cow<'static, str> {
    if scheme == &Scheme::HTTP {
        "http".into()
    } else if scheme == &Scheme::HTTPS {
        "https".into()
    } else {
        scheme.to_string().into()
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
    fn on_response(self, response: &Response<B>, _latency: Duration, span: &Span) {
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

impl OnFailure<ServerErrorsFailureClass> for OtelOnFailure {
    fn on_failure(&mut self, failure: ServerErrorsFailureClass, _latency: Duration, span: &Span) {
        match failure {
            ServerErrorsFailureClass::StatusCode(status) => {
                if status.is_server_error() {
                    span.record("otel.status_code", "ERROR");
                }
            }
            ServerErrorsFailureClass::Error(_) => {
                span.record("otel.status_code", "ERROR");
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::*;
    use axum::{
        body::Body,
        routing::{get, post},
        Router,
    };
    use http::{Request, StatusCode};
    use opentelemetry::sdk::propagation::TraceContextPropagator;
    use rstest::*;
    use serde_json::Value;
    use std::sync::mpsc::{self, Receiver, SyncSender};

    use tracing_subscriber::{
        fmt::{format::FmtSpan, MakeWriter},
        util::SubscriberInitExt,
        EnvFilter,
    };

    #[rstest]
    #[case("filled_http_route_for_existing_route", "/users/123", &[], false)]
    #[case("empty_http_route_for_nonexisting_route", "/idontexist/123", &[], false)]
    #[case("status_code_on_close_for_ok", "/users/123", &[], false)]
    #[case("status_code_on_close_for_error", "/status/500", &[], false)]
    #[case("filled_http_headers", "/users/123", &[("user-agent", "tests"), ("x-forwarded-for", "127.0.0.1")], false)]
    #[case("call_with_w3c_trace", "/users/123", &[("traceparent", "00-b2611246a58fd7ea623d2264c5a1e226-b2c9b811f2f424af-01")], true)]
    #[case("trace_id_in_child_span", "/with_child_span", &[], false)]
    #[case("trace_id_in_child_span_for_remote", "/with_child_span", &[("traceparent", "00-b2611246a58fd7ea623d2264c5a1e226-b2c9b811f2f424af-01")], true)]
    // failed to extract "http.route" before axum-0.6.15
    // - https://github.com/davidB/axum-tracing-opentelemetry/pull/54 (reverted)
    // - https://github.com/tokio-rs/axum/issues/1441#issuecomment-1272158039
    #[case("extract_route_from_nested", "/nest/123", &[], false)]
    #[tokio::test(flavor = "multi_thread")]
    async fn check_span_event(
        #[case] name: &str,
        #[case] uri: &str,
        #[case] headers: &[(&str, &str)],
        #[case] is_trace_id_constant: bool,
    ) {
        let svc = Router::new()
            .route("/users/:id", get(|| async { StatusCode::OK }))
            .route(
                "/status/500",
                get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
            )
            .route(
                "/with_child_span",
                get(|| async {
                    let span = tracing::span!(tracing::Level::INFO, "my child span");
                    span.in_scope(|| {
                        // Any trace events in this closure or code called by it will occur within
                        // the span.
                    });
                    StatusCode::OK
                }),
            )
            .nest(
                "/nest",
                Router::new()
                    .route("/:nest_id", get(|| async {}))
                    .fallback(|| async { (StatusCode::NOT_FOUND, "inner fallback") }),
            )
            .fallback(|| async { (StatusCode::NOT_FOUND, "outer fallback") })
            .layer(opentelemetry_tracing_layer());
        let mut builder = Request::builder();
        for (key, value) in headers.iter() {
            builder = builder.header(*key, *value);
        }
        let req = builder.uri(uri).body(Body::empty()).unwrap();
        let (tracing_events, otel_spans) = span_event_for_request(svc, req).await;
        assert_trace(name, tracing_events, otel_spans, is_trace_id_constant);
    }

    #[rstest]
    #[case("grpc_status_code_on_close_for_ok", "/module.service/endpoint1", &[])]
    #[tokio::test(flavor = "multi_thread")]
    async fn check_span_event_grpc(
        #[case] name: &str,
        #[case] uri: &str,
        #[case] headers: &[(&str, &str)],
    ) {
        let svc = Router::new()
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
            .layer(opentelemetry_tracing_layer_grpc());
        let mut builder = Request::builder();
        for (key, value) in headers.iter() {
            builder = builder.header(*key, *value);
        }
        builder = builder.method("POST");
        let req = builder.uri(uri).body(Body::empty()).unwrap();
        let (tracing_events, otel_spans) = span_event_for_request(svc, req).await;
        assert_trace(name, tracing_events, otel_spans, false);
    }

    fn assert_trace(
        name: &str,
        tracing_events: Vec<Value>,
        otel_spans: Vec<fake_opentelemetry_collector::ExportedSpan>,
        is_trace_id_constant: bool,
    ) {
        let trace_id_0 = tracing_events
            .get(0)
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("span"))
            .and_then(|v| v.as_object())
            .and_then(|v| v.get("trace_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned();
        // let trace_id_3 = trace_id_0.clone();
        let trace_id_1 = trace_id_0.clone();
        let trace_id_2 = trace_id_0;
        insta::assert_yaml_snapshot!(name, tracing_events, {
            "[].timestamp" => "[timestamp]",
            "[].fields[\"time.busy\"]" => "[duration]",
            "[].fields[\"time.idle\"]" => "[duration]",
            "[].span.trace_id" => insta::dynamic_redaction(move |value, _path| {
                let_assert!(Some(tracing_trace_id) = value.as_str());
                check!(trace_id_1 == tracing_trace_id);
                if is_trace_id_constant {
                    tracing_trace_id.to_string()
                } else {
                    format!("[trace_id:lg{}]", tracing_trace_id.len())
                }
            }),
            "[].spans[].trace_id" => insta::dynamic_redaction(move |value, _path| {
                let_assert!(Some(tracing_trace_id) = value.as_str());
                check!(trace_id_2 == tracing_trace_id);
                if is_trace_id_constant {
                    tracing_trace_id.to_string()
                } else {
                    format!("[trace_id:lg{}]", tracing_trace_id.len())
                }
            }),
        });
        insta::assert_yaml_snapshot!(format!("{}_otel_spans", name), otel_spans, {
            "[].start_time_unix_nano" => "[timestamp]",
            "[].end_time_unix_nano" => "[timestamp]",
            "[].events[].time_unix_nano" => "[timestamp]",
            "[].trace_id" => insta::dynamic_redaction(move |value, _path| {
                assert2::let_assert!(Some(otel_trace_id) = value.as_str());
                //FIXME check!(trace_id_3 == otel_trace_id);
                format!("[trace_id:lg{}]", otel_trace_id.len())
            }),
            "[].span_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(span_id) = value.as_str());
                format!("[span_id:lg{}]", span_id.len())
            }),
            "[].parent_span_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(span_id) = value.as_str());
                format!("[span_id:lg{}]", span_id.len())
            }),
            "[].links[].trace_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(otel_trace_id) = value.as_str());
                format!("[trace_id:lg{}]", otel_trace_id.len())
            }),
            "[].links[].span_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(span_id) = value.as_str());
                format!("[span_id:lg{}]", span_id.len())
            }),
            "[].attributes.busy_ns" => "ignore",
            "[].attributes.idle_ns" => "ignore",
            "[].attributes.trace_id" => "ignore",
            "[].attributes[\"code.lineno\"]" => "ignore",
            "[].attributes[\"code.filepath\"]" => "ignore",
            "[].attributes[\"thread.id\"]" => "ignore",
        });
    }

    async fn span_event_for_request(
        mut router: Router,
        req: Request<Body>,
    ) -> (Vec<Value>, Vec<fake_opentelemetry_collector::ExportedSpan>) {
        use axum::body::HttpBody as _;
        use tower::{Service, ServiceExt};
        use tracing_subscriber::layer::SubscriberExt;

        // setup a non Noop OpenTelemetry tracer to have non-empty trace_id
        let fake_collector = fake_opentelemetry_collector::FakeCollectorServer::start()
            .await
            .unwrap();
        let tracer = fake_opentelemetry_collector::setup_tracer(&fake_collector).await;
        //let (tracer, mut req_rx) = fake_opentelemetry_collector::setup_tracer().await;
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let (make_writer, rx) = duplex_writer();
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(make_writer)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::try_new("axum_extra=trace,info").unwrap())
            .with(fmt_layer)
            .with(otel_layer);
        let _guard = subscriber.set_default();

        let mut res = router.ready().await.unwrap().call(req).await.unwrap();

        while res.data().await.is_some() {}
        res.trailers().await.unwrap();
        drop(res);

        opentelemetry_api::global::shutdown_tracer_provider();

        let otel_span = fake_collector.exported_spans();
        // insta::assert_debug_snapshot!(first_span);
        let tracing_events = std::iter::from_fn(|| rx.try_recv().ok())
            .map(|bytes| serde_json::from_slice::<Value>(&bytes).unwrap())
            .collect::<Vec<_>>();
        (tracing_events, otel_span)
    }

    fn duplex_writer() -> (DuplexWriter, Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::sync_channel(1024);
        (DuplexWriter { tx }, rx)
    }

    #[derive(Clone)]
    struct DuplexWriter {
        tx: SyncSender<Vec<u8>>,
    }

    impl<'a> MakeWriter<'a> for DuplexWriter {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    impl std::io::Write for DuplexWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.tx.send(buf.to_vec()).unwrap();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
