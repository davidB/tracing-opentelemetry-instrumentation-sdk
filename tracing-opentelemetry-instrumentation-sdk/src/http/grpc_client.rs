use crate::http::*;
use crate::BoxError;
use crate::TRACING_TARGET;
use tracing::field::Empty;

// [opentelemetry-specification/specification/trace/semantic\_conventions/rpc.md at main · open-telemetry/opentelemetry-specification · GitHub](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md)
//TODO create similar but with tonic::Request<B> ?
pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    let (service, method) = extract_service_method(req.uri());
    tracing::trace_span!(
        target: TRACING_TARGET,
        "GRPC request",
        http.user_agent = %user_agent(req),
        http.status_code = Empty, // to set on response
        otel.name = format!("GRPC {}", req.uri().path()),
        otel.kind = ?opentelemetry_api::trace::SpanKind::Client,
        otel.status_code = Empty,
        rpc.system ="grpc",
        rpc.service = %service,
        rpc.method = %method,
        server.address = %http_host(req),
        exception.message = Empty, // to set on response
        exception.details = Empty, // to set on response
    )
}

//TODO update behavior for grpc
//TODO create similar but with tonic::Response<B> ?
//TODO set `http.grpc_status`
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

pub fn update_span_from_error(span: &mut tracing::Span, error: &BoxError) {
    span.record("otel.status_code", "ERROR");
    span.record("http.grpc_status", 1);
    span.record("exception.message", error.to_string());
    error
        .source()
        .map(|s| span.record("exception.message", s.to_string()));
}

pub fn update_span_from_response_or_error<B>(
    span: &mut tracing::Span,
    response: &Result<http::Response<B>, BoxError>,
) {
    match response {
        Ok(response) => {
            update_span_from_response(span, response);
        }
        Err(err) => {
            update_span_from_error(span, err);
        }
    }
}
