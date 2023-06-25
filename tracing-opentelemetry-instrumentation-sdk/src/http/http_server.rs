use crate::http::*;
use crate::TRACING_TARGET;
use tracing::field::Empty;

pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    // [opentelemetry-specification/specification/trace/semantic\_conventions/rpc.md at main · open-telemetry/opentelemetry-specification · GitHub](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md)
    // Can not use const or opentelemetry_semantic_conventions::trace::* for name of records
    let http_method = http_method(req.method());
    tracing::trace_span!(
        target: TRACING_TARGET,
        "HTTP request",
        http.method = %http_method,
        http.route = Empty, // to set by router of "webframework" after
        http.flavor = %http_flavor(req.version()),
        http.scheme = %http_scheme(req.uri()),
        http.host = %http_host(req),
        http.client_ip = Empty, //%$request.connection_info().realip_remote_addr().unwrap_or(""),
        http.user_agent = %user_agent(req),
        http.target = %http_target(req.uri()),
        http.status_code = Empty, // to set on response
        otel.name = %format!("HTTP {}", http_method), // to set by router of "webframework" after
        otel.kind = ?opentelemetry_api::trace::SpanKind::Server,
        otel.status_code =Empty, // to set on response
        // trace_id = Empty, // to set on response
        request_id = Empty, // to set
        exception.message = Empty, // to set on response
        // Not proper OpenTelemetry, but their terminology is fairly exception-centric
        exception.details = Empty, // to set on response
    )
}

pub fn update_span_from_response<B>(span: &mut tracing::Span, response: &http::Response<B>) {
    let status = response.status();
    span.record(
        "http.status_code",
        &tracing::field::display(status.as_u16()),
    );

    if status.is_server_error() {
        span.record("otel.status_code", "ERROR");
    } else {
        span.record("otel.status_code", "OK");
    }
}
