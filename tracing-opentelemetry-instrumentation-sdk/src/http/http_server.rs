use std::error::Error;

use crate::http::{http_flavor, http_host, url_scheme, user_agent};
use crate::otel_trace_span;
use crate::span_type::SpanType;
use opentelemetry_semantic_conventions::attribute::OTEL_STATUS_CODE;
use opentelemetry_semantic_conventions::trace::{EXCEPTION_MESSAGE, HTTP_RESPONSE_STATUS_CODE};
use tracing::field::Empty;

pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    // [semantic-conventions/.../http-spans.md](https://github.com/open-telemetry/semantic-conventions/blob/v1.25.0/docs/http/http-spans.md)
    // [semantic-conventions/.../general/attributes.md](https://github.com/open-telemetry/semantic-conventions/blob/v1.25.0/docs/general/attributes.md)
    // Can not use const or opentelemetry_semantic_conventions::trace::* for name of records
    let http_method = req.method();
    otel_trace_span!(
        "HTTP request",
        http.request.method = %http_method,
        http.route = Empty, // to set by router of "webframework" after
        network.protocol.version = %http_flavor(req.version()),
        server.address = http_host(req),
        // server.port = req.uri().port(),
        http.client.address = Empty, //%$request.connection_info().realip_remote_addr().unwrap_or(""),
        user_agent.original = user_agent(req),
        http.response.status_code = Empty, // to set on response
        url.path = req.uri().path(),
        url.query = req.uri().query(),
        url.scheme = url_scheme(req.uri()),
        otel.name = %http_method, // to set by router of "webframework" after
        otel.kind = ?opentelemetry::trace::SpanKind::Server,
        otel.status_code = Empty, // to set on response
        trace_id = Empty, // to set on response
        request_id = Empty, // to set
        exception.message = Empty, // to set on response
        "span.type" = %SpanType::Web, // non-official open-telemetry key, only supported by Datadog
    )
}

pub fn update_span_from_response<B>(span: &tracing::Span, response: &http::Response<B>) {
    let status = response.status();
    span.record(HTTP_RESPONSE_STATUS_CODE, status.as_u16());

    if status.is_server_error() {
        span.record(OTEL_STATUS_CODE, "ERROR");
        // see [http-spans.md#status](https://github.com/open-telemetry/semantic-conventions/blob/v1.25.0/docs/http/http-spans.md#status)
        // Span Status MUST be left unset if HTTP status code was in the 1xx, 2xx or 3xx ranges,
        // unless there was another error (e.g., network error receiving the response body;
        // or 3xx codes with max redirects exceeded), in which case status MUST be set to Error.
        // } else {
        //     span.record(OTEL_STATUS_CODE, "OK");
    }
}

pub fn update_span_from_error<E>(span: &tracing::Span, error: &E)
where
    E: Error,
{
    span.record(OTEL_STATUS_CODE, "ERROR");
    //span.record(HTTP_RESPONSE_STATUS_CODE, 500);
    span.record(EXCEPTION_MESSAGE, error.to_string());
    error
        .source()
        .map(|s| span.record(EXCEPTION_MESSAGE, s.to_string()));
}

pub fn update_span_from_response_or_error<B, E>(
    span: &tracing::Span,
    response: &Result<http::Response<B>, E>,
) where
    E: Error,
{
    match response {
        Ok(response) => {
            update_span_from_response(span, response);
        }
        Err(err) => {
            update_span_from_error(span, err);
        }
    }
}
