//! Expose a Prometheus [`Registry`](prometheus::Registry) over HTTP.
//!
//! Pair with `init_tracing_opentelemetry::TracingConfig::with_metrics_prometheus`,
//! which builds the registry this router serves.

use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::routing::get;
use prometheus::{Encoder, Registry, TextEncoder};

/// Build a `/metrics` route serving `registry` in Prometheus text format.
///
/// ```txt
/// use init_tracing_opentelemetry::TracingConfig;
///
/// let guard = TracingConfig::development()
///     .with_metrics_prometheus()
///     .init_subscriber()?;
/// let registry = guard.prometheus_registry().expect("prometheus enabled").clone();
/// let app = axum::Router::new().merge(axum_tracing_opentelemetry::prometheus_metrics::router(registry));
/// ```
pub fn router(registry: Registry) -> Router {
    Router::new().route(
        "/metrics",
        get(move || {
            let registry = registry.clone();
            async move {
                let encoder = TextEncoder::new();
                let mut buf = Vec::new();
                encoder
                    .encode(&registry.gather(), &mut buf)
                    .unwrap_or_else(|err| {
                        tracing::warn!("failed to encode prometheus metrics: {err}")
                    });
                ([(CONTENT_TYPE, encoder.format_type().to_string())], buf)
            }
        }),
    )
}
