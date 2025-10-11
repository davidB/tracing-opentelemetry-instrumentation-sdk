//
//! `OpenTelemetry` tracing middleware.
//!
//! This returns a [`OtelAxumLayer`] configured to use [`OpenTelemetry`'s conventional span field
//! names][otel].
//!
//! # Span fields
//!
//! Try to provide some of the field define at
//! [semantic-conventions/.../http-spans.md](https://github.com/open-telemetry/semantic-conventions/blob/v1.25.0/docs/http/http-spans.md)
//! (Please report or provide fix for missing one)
//!
//! # Example
//!
//! ```
//! use axum::{Router, routing::get, http::Request};
//! use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
//! use std::net::SocketAddr;
//! use tower::ServiceBuilder;
//!
//! let app = Router::new()
//!     .route("/", get(|| async {}))
//!     .layer(OtelAxumLayer::default());
//!
//! # async {
//! let addr = &"0.0.0.0:3000".parse::<SocketAddr>().unwrap();
//! let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//! axum::serve(listener, app.into_make_service())
//!     .await
//!     .expect("server failed");
//! # };
//! ```
//!

use axum::extract::{ConnectInfo, MatchedPath};
use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    error::Error,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http::{
    self as otel_http, extract_client_ip_from_headers,
};

#[deprecated(
    since = "0.12.0",
    note = "keep for transition, replaced by OtelAxumLayer"
)]
#[must_use]
pub fn opentelemetry_tracing_layer() -> OtelAxumLayer {
    OtelAxumLayer::default()
}

pub type Filter = fn(&str) -> bool;

/// layer/middleware for axum:
///
/// - propagate `OpenTelemetry` context (`trace_id`,...) to server
/// - create a Span for `OpenTelemetry` (and tracing) on call
///
/// `OpenTelemetry` context are extracted from tracing's span.
#[derive(Default, Debug, Clone)]
pub struct OtelAxumLayer {
    filter: Option<Filter>,
    try_extract_client_ip: bool,
}

// add a builder like api
impl OtelAxumLayer {
    #[must_use]
    pub fn filter(self, filter: Filter) -> Self {
        let mut me = self;
        me.filter = Some(filter);
        me
    }

    /// Enable or disable (default) the extraction of client's ip.
    /// Extraction from (in order):
    ///
    /// 1. http header 'Forwarded'
    /// 2. http header `X-Forwarded-For`
    /// 3. socket connection ip, use the `axum::extract::ConnectionInfo` (see [`Router::into_make_service_with_connect_info`] for more details)
    /// 4. empty (failed to extract the information)
    ///
    /// The extracted value could an ip v4, ip v6, a string (as `Forwarded` can use label or hide the client).
    /// The extracted value is stored it as `client.address` in the span/trace
    ///
    /// [`Router::into_make_service_with_connect_info`]: axum::routing::Router::into_make_service_with_connect_info
    #[must_use]
    pub fn try_extract_client_ip(self, enable: bool) -> Self {
        let mut me = self;
        me.try_extract_client_ip = enable;
        me
    }
}

impl<S> Layer<S> for OtelAxumLayer {
    /// The wrapped service
    type Service = OtelAxumService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        OtelAxumService {
            inner,
            filter: self.filter,
            try_extract_client_ip: self.try_extract_client_ip,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OtelAxumService<S> {
    inner: S,
    filter: Option<Filter>,
    try_extract_client_ip: bool,
}

impl<S, B, B2> Service<Request<B>> for OtelAxumService<S>
where
    S: Service<Request<B>, Response = Response<B2>> + Clone + Send + 'static,
    S::Error: Error + 'static, //fmt::Display + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // #[allow(clippy::type_complexity)]
    // type Future = futures_core::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        let req = req;
        let span = if self.filter.is_none_or(|f| f(req.uri().path())) {
            let route = http_route(&req);
            let method = req.method();
            let client_ip = if self.try_extract_client_ip {
                extract_client_ip_from_headers(req.headers())
                    .map(ToString::to_string)
                    .or_else(|| {
                        req.extensions()
                            .get::<ConnectInfo<SocketAddr>>()
                            .map(|ConnectInfo(client_ip)| client_ip.to_string())
                    })
            } else {
                None
            };

            let span = otel_http::http_server::make_span_from_request(&req);
            span.record("http.route", route);
            span.record("otel.name", format!("{method} {route}").trim());
            if let Some(client_ip) = client_ip {
                span.record("http.client.address", client_ip);
            }
            if let Err(error) = span.set_parent(otel_http::extract_context(req.headers())) {
                tracing::warn!(?error, "can not set parent trace_id to span");
            }
            span
        } else {
            tracing::Span::none()
        };
        let future = {
            let _enter = span.enter();
            self.inner.call(req)
        };
        ResponseFuture {
            inner: future,
            span,
        }
    }
}

pin_project! {
    /// Response future for [`Trace`].
    ///
    /// [`Trace`]: super::Trace
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        // pub(crate) start: Instant,
    }
}

impl<Fut, ResBody, E> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response<ResBody>, E>>,
    E: std::error::Error + 'static,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = futures_util::ready!(this.inner.poll(cx));
        otel_http::http_server::update_span_from_response_or_error(this.span, &result);
        Poll::Ready(result)
    }
}

#[inline]
fn http_route<B>(req: &Request<B>) -> &str {
    req.extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "", |mp| mp.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, routing::get, Router};
    use http::{Request, StatusCode};
    use rstest::rstest;
    use testing_tracing_opentelemetry::{assert_trace, FakeEnvironment};
    use tower::Service;

    #[rstest]
    #[case("filled_http_route_for_existing_route", "http://example.com/users/123", &[], false)]
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
        let mut fake_env = FakeEnvironment::setup().await;
        {
            let mut svc = Router::new()
                .route("/users/{id}", get(|| async { StatusCode::OK }))
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
                        .route("/{nest_id}", get(|| async {}))
                        .fallback(|| async { (StatusCode::NOT_FOUND, "inner fallback") }),
                )
                .fallback(|| async { (StatusCode::NOT_FOUND, "outer fallback") })
                .layer(opentelemetry_tracing_layer());
            let mut builder = Request::builder();
            for (key, value) in headers {
                builder = builder.header(*key, *value);
            }
            let req = builder.uri(uri).body(Body::empty()).unwrap();
            let _res = svc.call(req).await.unwrap();

            // while res.data().await.is_some() {}
            // res.trailers().await.unwrap();
            // drop(res);
        }
        let (tracing_events, otel_spans) = fake_env.collect_traces().await;
        assert_trace(name, tracing_events, otel_spans, is_trace_id_constant);
    }
}
