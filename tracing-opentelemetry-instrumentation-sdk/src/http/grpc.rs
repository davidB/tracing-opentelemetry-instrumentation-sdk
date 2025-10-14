use http::HeaderMap;
use opentelemetry_semantic_conventions::attribute::{
    EXCEPTION_MESSAGE, OTEL_STATUS_CODE, RPC_GRPC_STATUS_CODE,
};

/// [`gRPC` status codes](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc)
/// copied from tonic
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum GrpcCode {
    /// The operation completed successfully.
    Ok = 0,

    /// The operation was cancelled.
    Cancelled = 1,

    /// Unknown error.
    Unknown = 2,

    /// Client specified an invalid argument.
    InvalidArgument = 3,

    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,

    /// Some requested entity was not found.
    NotFound = 5,

    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,

    /// Some resource has been exhausted.
    ResourceExhausted = 8,

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition = 9,

    /// The operation was aborted.
    Aborted = 10,

    /// Operation was attempted past the valid range.
    OutOfRange = 11,

    /// Operation is not implemented or not supported.
    Unimplemented = 12,

    /// Internal error.
    Internal = 13,

    /// The service is currently unavailable.
    Unavailable = 14,

    /// Unrecoverable data loss or corruption.
    DataLoss = 15,

    /// The request does not have valid authentication credentials
    Unauthenticated = 16,
}

/// If "grpc-status" can not be extracted from http response, the status "0" (Ok) is defined
//TODO create similar but with tonic::Response<B> ? and use of [Status in tonic](https://docs.rs/tonic/latest/tonic/struct.Status.html) (more complete)
pub fn update_span_from_response<B>(
    span: &tracing::Span,
    response: &http::Response<B>,
    is_spankind_server: bool,
) {
    let status = status_from_http_header(response.headers())
        .or_else(|| status_from_http_status(response.status()))
        .unwrap_or(GrpcCode::Ok as u16);
    span.record(RPC_GRPC_STATUS_CODE, status);

    if status_is_error(status, is_spankind_server) {
        span.record(OTEL_STATUS_CODE, "ERROR");
    } else {
        span.record(OTEL_STATUS_CODE, "OK");
    }
}

/// based on [Status in tonic](https://docs.rs/tonic/latest/tonic/struct.Status.html#method.from_header_map)
fn status_from_http_header(headers: &HeaderMap) -> Option<u16> {
    headers
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u16>().ok())
}

fn status_from_http_status(status_code: http::StatusCode) -> Option<u16> {
    match status_code {
        // Borrowed from https://github.com/grpc/grpc/blob/master/doc/http-grpc-status-mapping.md
        http::StatusCode::BAD_REQUEST => Some(GrpcCode::Internal as u16),
        http::StatusCode::UNAUTHORIZED => Some(GrpcCode::Unauthenticated as u16),
        http::StatusCode::FORBIDDEN => Some(GrpcCode::PermissionDenied as u16),
        http::StatusCode::NOT_FOUND => Some(GrpcCode::Unimplemented as u16),
        http::StatusCode::TOO_MANY_REQUESTS
        | http::StatusCode::BAD_GATEWAY
        | http::StatusCode::SERVICE_UNAVAILABLE
        | http::StatusCode::GATEWAY_TIMEOUT => Some(GrpcCode::Unavailable as u16),
        // We got a 200 but no trailers, we can infer that this request is finished.
        //
        // This can happen when a streaming response sends two Status but
        // gRPC requires that we end the stream after the first status.
        //
        // https://github.com/hyperium/tonic/issues/681
        http::StatusCode::OK => None,
        _ => Some(GrpcCode::Unknown as u16),
    }
}

#[inline]
#[must_use]
/// see [Semantic Conventions for gRPC | OpenTelemetry](https://opentelemetry.io/docs/specs/semconv/rpc/grpc/)
/// see [GRPC Core: Status codes and their use in gRPC](https://grpc.github.io/grpc/core/md_doc_statuscodes.html)
pub fn status_is_error(status: u16, is_spankind_server: bool) -> bool {
    if is_spankind_server {
        status == 2 || status == 4 || status == 12 || status == 13 || status == 14 || status == 15
    } else {
        status != 0
    }
}

fn update_span_from_error<E>(span: &tracing::Span, error: &E)
where
    E: std::error::Error,
{
    span.record(OTEL_STATUS_CODE, "ERROR");
    span.record(RPC_GRPC_STATUS_CODE, 2);
    span.record(EXCEPTION_MESSAGE, error.to_string());
    error
        .source()
        .map(|s| span.record(EXCEPTION_MESSAGE, s.to_string()));
}

pub fn update_span_from_response_or_error<B, E>(
    span: &tracing::Span,
    response: &Result<http::Response<B>, E>,
) where
    E: std::error::Error,
{
    match response {
        Ok(response) => {
            update_span_from_response(span, response, true);
        }
        Err(err) => {
            update_span_from_error(span, err);
        }
    }
}

// [opentelemetry-specification/.../rpc.md](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md)
//TODO create similar but with tonic::Request<B> ?
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn make_span_from_request<B>(
    req: &http::Request<B>,
    kind: opentelemetry::trace::SpanKind,
) -> tracing::Span {
    use crate::http::{extract_service_method, http_host, user_agent};
    use crate::otel_trace_span;
    use tracing::field::Empty;

    let (service, method) = extract_service_method(req.uri());
    otel_trace_span!(
        "GRPC request",
        http.user_agent = %user_agent(req),
        otel.name = format!("{service}/{method}"),
        otel.kind = ?kind,
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

// if let Some(host_name) = SYSTEM.host_name() {
//     attributes.push(NET_HOST_NAME.string(host_name));
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0)]
    #[case(16)]
    #[case(-1)]
    fn test_status_from_http_header(#[case] input: i32) {
        let mut headers = http::HeaderMap::new();
        headers.insert("grpc-status", input.to_string().parse().unwrap());
        if input > -1 {
            assert_eq!(
                status_from_http_header(&headers),
                Some(u16::try_from(input).unwrap())
            );
        } else {
            assert_eq!(status_from_http_header(&headers), None);
        }
    }
}
