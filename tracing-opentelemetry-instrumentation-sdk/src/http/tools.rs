use std::borrow::Cow;

use http::{HeaderMap, Method, Uri, Version};
use opentelemetry::Context;

pub fn inject_context(context: &Context, headers: &mut http::HeaderMap) {
    use opentelemetry_http::HeaderInjector;
    let mut injector = HeaderInjector(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, &mut injector);
    });
}

// If remote request has no span data the propagator defaults to an unsampled context
#[must_use]
pub fn extract_context(headers: &http::HeaderMap) -> Context {
    use opentelemetry_http::HeaderExtractor;
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

// if let Some(host_name) = SYSTEM.host_name() {
//     attributes.push(NET_HOST_NAME.string(host_name));
// }

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::check;
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
        check!(extract_service_method(&path.parse::<Uri>().unwrap()) == (service, method));
    }

    #[rstest]
    #[case("http://example.org/hello/world", "http")]
    #[case("https://example.org/hello/world", "https")]
    #[case("foo://example.org/hello/world", "foo")]
    fn test_extract_url_scheme(#[case] input: &str, #[case] expected: &str) {
        let uri: Uri = input.parse().unwrap();
        check!(url_scheme(&uri) == expected);
    }
}
