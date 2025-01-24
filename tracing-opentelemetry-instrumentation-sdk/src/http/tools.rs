use std::borrow::Cow;

use http::{HeaderMap, Method, Uri, Version};
use opentelemetry::Context;

use super::opentelemetry_http::{HeaderExtractor, HeaderInjector};

pub fn inject_context(context: &Context, headers: &mut http::HeaderMap) {
    let mut injector = HeaderInjector(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, &mut injector);
    });
}

// If remote request has no span data the propagator defaults to an unsampled context
#[must_use]
pub fn extract_context(headers: &http::HeaderMap) -> Context {
    let extractor = HeaderExtractor(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| propagator.extract(&extractor))
}

pub fn extract_service_method(uri: &Uri) -> (&str, &str) {
    let path = uri.path();
    let mut parts = path.split('/').filter(|x| !x.is_empty());
    let service = parts.next().unwrap_or_default();
    let method = parts.next().unwrap_or_default();
    (service, method)
}

fn parse_x_forwarded_for(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("x-forwarded-for")?;
    let value = value.to_str().ok()?;
    let mut ips = value.split(',');
    Some(ips.next()?.trim())
}

#[inline]
pub fn client_ip<B>(req: &http::Request<B>) -> &str {
    parse_x_forwarded_for(req.headers())
        // .or_else(|| {
        //     req.extensions()
        //         .get::<ConnectInfo<SocketAddr>>()
        //         .map(|ConnectInfo(client_ip)| Cow::from(client_ip.to_string()))
        // })
        .unwrap_or_default()
}

#[inline]
pub fn http_target(uri: &Uri) -> &str {
    uri.path_and_query()
        .map_or("", http::uri::PathAndQuery::as_str)
}

#[inline]
#[must_use]
pub fn http_method(method: &Method) -> Cow<'static, str> {
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

#[inline]
#[must_use]
pub fn http_flavor(version: Version) -> Cow<'static, str> {
    match version {
        Version::HTTP_09 => "0.9".into(),
        Version::HTTP_10 => "1.0".into(),
        Version::HTTP_11 => "1.1".into(),
        Version::HTTP_2 => "2.0".into(),
        Version::HTTP_3 => "3.0".into(),
        other => format!("{other:?}").into(),
    }
}

#[inline]
pub fn url_scheme(uri: &Uri) -> &str {
    uri.scheme_str().unwrap_or_default()
}

#[inline]
pub fn user_agent<B>(req: &http::Request<B>) -> &str {
    req.headers()
        .get(http::header::USER_AGENT)
        .map_or("", |h| h.to_str().unwrap_or(""))
}

#[inline]
pub fn http_host<B>(req: &http::Request<B>) -> &str {
    req.headers()
        .get(http::header::HOST)
        .map_or(req.uri().host(), |h| h.to_str().ok())
        .unwrap_or("")
}

/// [`gRPC` status codes](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc)
/// copied from tonic
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
pub fn grpc_update_span_from_response<B>(
    span: &tracing::Span,
    response: &http::Response<B>,
    is_spankind_server: bool,
) {
    let status = grpc_status_from_http_header(response.headers())
        .or_else(|| grpc_status_from_http_status(response.status()))
        .unwrap_or(GrpcCode::Ok as u16);
    span.record("rpc.grpc.status_code", status);

    if grpc_status_is_error(status, is_spankind_server) {
        span.record("otel.status_code", "ERROR");
    } else {
        span.record("otel.status_code", "OK");
    }
}

/// based on [Status in tonic](https://docs.rs/tonic/latest/tonic/struct.Status.html#method.from_header_map)
fn grpc_status_from_http_header(headers: &HeaderMap) -> Option<u16> {
    headers
        .get("grpc-status")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u16>().ok())
}

fn grpc_status_from_http_status(status_code: http::StatusCode) -> Option<u16> {
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
pub fn grpc_status_is_error(status: u16, is_spankind_server: bool) -> bool {
    if is_spankind_server {
        status == 2 || status == 4 || status == 12 || status == 13 || status == 14 || status == 15
    } else {
        status != 0
    }
}

// if let Some(host_name) = SYSTEM.host_name() {
//     attributes.push(NET_HOST_NAME.string(host_name));
// }

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::assert;
    use rstest::rstest;

    #[rstest]
    // #[case("", "", "")]
    #[case("/", "", "")]
    #[case("//", "", "")]
    #[case("/grpc.health.v1.Health/Check", "grpc.health.v1.Health", "Check")]
    fn test_extract_service_method(
        #[case] path: &str,
        #[case] service: &str,
        #[case] method: &str,
    ) {
        assert!(extract_service_method(&path.parse::<Uri>().unwrap()) == (service, method));
    }

    #[rstest]
    #[case("http://example.org/hello/world", "http")] // Devskim: ignore DS137138
    #[case("https://example.org/hello/world", "https")]
    #[case("foo://example.org/hello/world", "foo")]
    fn test_extract_url_scheme(#[case] input: &str, #[case] expected: &str) {
        let uri: Uri = input.parse().unwrap();
        assert!(url_scheme(&uri) == expected);
    }

    #[rstest]
    #[case(0)]
    #[case(16)]
    #[case(-1)]
    fn test_grpc_status_from_http_header(#[case] input: i32) {
        let mut headers = http::HeaderMap::new();
        headers.insert("grpc-status", input.to_string().parse().unwrap());
        if input > -1 {
            assert_eq!(
                grpc_status_from_http_header(&headers),
                Some(u16::try_from(input).unwrap())
            );
        } else {
            assert_eq!(grpc_status_from_http_header(&headers), None);
        }
    }
}
