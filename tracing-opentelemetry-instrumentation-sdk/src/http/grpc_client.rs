use std::error::Error;

use crate::http::{extract_service_method, http_host, user_agent};
use crate::TRACING_TARGET;
use tracing::field::Empty;

use super::grpc_update_span_from_response;

// [opentelemetry-specification/.../rpc.md](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md)
//TODO create similar but with tonic::Request<B> ?
pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    let (service, method) = extract_service_method(req.uri());
    tracing::trace_span!(
        target: TRACING_TARGET,
        "GRPC request",
        http.user_agent = %user_agent(req),
        otel.name = format!("{service}/{method}"),
        otel.kind = ?opentelemetry_api::trace::SpanKind::Client,
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

fn update_span_from_error<E>(span: &tracing::Span, error: &E)
where
    E: Error,
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
    E: Error,
{
    match response {
        Ok(response) => {
            grpc_update_span_from_response(span, response, false);
        }
        Err(err) => {
            update_span_from_error(span, err);
        }
    }
}
