use std::borrow::Cow;

use http::{HeaderMap, Uri, Version};
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

#[must_use]
// From [X-Forwarded-For - HTTP | MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)
// > If a request goes through multiple proxies, the IP addresses of each successive proxy is listed.
// > This means that, given well-behaved client and proxies,
// > the rightmost IP address is the IP address of the most recent proxy and
// > the leftmost IP address is the IP address of the originating client.
pub fn extract_client_ip_from_headers(headers: &HeaderMap) -> Option<&str> {
    extract_client_ip_from_forwarded(headers)
        .or_else(|| extract_client_ip_from_x_forwarded_for(headers))
}

#[must_use]
// From [X-Forwarded-For - HTTP | MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)
// > If a request goes through multiple proxies, the IP addresses of each successive proxy is listed.
// > This means that, given well-behaved client and proxies,
// > the rightmost IP address is the IP address of the most recent proxy and
// > the leftmost IP address is the IP address of the originating client.
fn extract_client_ip_from_x_forwarded_for(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("x-forwarded-for")?;
    let value = value.to_str().ok()?;
    let mut ips = value.split(',');
    Some(ips.next()?.trim())
}

#[must_use]
// see [Forwarded header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Forwarded)
fn extract_client_ip_from_forwarded(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("forwarded")?;
    let value = value.to_str().ok()?;
    value
        .split(';')
        .flat_map(|directive| directive.split(','))
        // select the left/first "for" key
        .find_map(|directive| directive.trim().strip_prefix("for="))
        // ipv6 are enclosed into `["..."]`
        // string are enclosed into `"..."`
        .map(|directive| {
            directive
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim_matches('"')
                .trim()
        })
}

#[inline]
pub fn http_target(uri: &Uri) -> &str {
    uri.path_and_query()
        .map_or("", http::uri::PathAndQuery::as_str)
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
    #[case("", "")]
    #[case(
        "2001:db8:85a3:8d3:1319:8a2e:370:7348",
        "2001:db8:85a3:8d3:1319:8a2e:370:7348"
    )]
    #[case("203.0.113.195", "203.0.113.195")]
    #[case("203.0.113.195,10.10.10.10", "203.0.113.195")]
    #[case("203.0.113.195, 2001:db8:85a3:8d3:1319:8a2e:370:7348", "203.0.113.195")]
    fn test_extract_client_ip_from_x_forwarded_for(#[case] input: &str, #[case] expected: &str) {
        let mut headers = HeaderMap::new();
        if !input.is_empty() {
            headers.insert("X-Forwarded-For", input.parse().unwrap());
        }

        let expected = if expected.is_empty() {
            None
        } else {
            Some(expected)
        };
        assert!(extract_client_ip_from_x_forwarded_for(&headers) == expected);
    }

    #[rstest]
    #[case("", "")]
    #[case(
        "for=[\"2001:db8:85a3:8d3:1319:8a2e:370:7348\"]",
        "2001:db8:85a3:8d3:1319:8a2e:370:7348"
    )]
    #[case("for=203.0.113.195", "203.0.113.195")]
    #[case("for=203.0.113.195, for=10.10.10.10", "203.0.113.195")]
    #[case(
        "for=203.0.113.195, for=[\"2001:db8:85a3:8d3:1319:8a2e:370:7348\"]",
        "203.0.113.195"
    )]
    #[case("for=\"_mdn\"", "_mdn")]
    #[case("for=\"secret\"", "secret")]
    #[case("for=203.0.113.195;proto=http;by=203.0.113.43", "203.0.113.195")]
    #[case("proto=http;by=203.0.113.43", "")]
    fn test_extract_client_ip_from_forwarded(#[case] input: &str, #[case] expected: &str) {
        let mut headers = HeaderMap::new();
        if !input.is_empty() {
            headers.insert("Forwarded", input.parse().unwrap());
        }

        let expected = if expected.is_empty() {
            None
        } else {
            Some(expected)
        };
        assert!(extract_client_ip_from_forwarded(&headers) == expected);
    }
}
