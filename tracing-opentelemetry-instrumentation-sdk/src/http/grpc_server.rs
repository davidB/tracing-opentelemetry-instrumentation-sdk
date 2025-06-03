use crate::http::{extract_service_method, http_host, user_agent};
use crate::otel_trace_span;
use tracing::field::Empty;

use super::grpc_update_span_from_response;

//TODO create similar but with tonic::Request<B> ?
/// see [Semantic Conventions for gRPC | OpenTelemetry](https://opentelemetry.io/docs/specs/semconv/rpc/grpc/#grpc-status)
pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    let (service, method) = extract_service_method(req.uri());
    otel_trace_span!(
        "GRPC request",
        http.user_agent = %user_agent(req),
        otel.name = format!("{service}/{method}"),
        otel.kind = ?opentelemetry::trace::SpanKind::Server,
        otel.status_code = Empty,
        rpc.system ="grpc",
        rpc.service = %service,
        rpc.method = %method,
        rpc.grpc.status_code = Empty, // to set on response
        server.address = %http_host(req),
        exception.message = Empty, // to set on response
        exception.details = Empty, // to set on response
    )
}

// fn update_span_from_error<E>(span: &tracing::Span, error: &E) {
//     span.record("otel.status_code", "ERROR");
//     span.record("rpc.grpc.status_code", 2);
// }

fn update_span_from_error<E>(span: &tracing::Span, error: &E)
where
    E: std::error::Error,
{
    span.record("otel.status_code", "ERROR");
    span.record("rpc.grpc.status_code", 2);
    span.record("exception.message", error.to_string());
    error
        .source()
        .map(|s| span.record("exception.message", s.to_string()));
}

pub fn update_span_from_response_or_error<B, E>(
    span: &tracing::Span,
    response: &Result<http::Response<B>, E>,
) where
    E: std::error::Error,
{
    match response {
        Ok(response) => {
            grpc_update_span_from_response(span, response, true);
        }
        Err(err) => {
            update_span_from_error(span, err);
        }
    }
}
