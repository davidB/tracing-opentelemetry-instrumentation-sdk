#![allow(clippy::default_constructed_unit_structs)] // warning since 1.71

use axum::extract::{MatchedPath, State};
use axum::http::Request;
use axum::{BoxError, Router, response::IntoResponse, routing::get};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use opentelemetry::metrics::Counter;
use opentelemetry::{KeyValue, global};
use serde_json::json;
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    // instrument built once via the raw `opentelemetry::metrics::Meter` API,
    // with attributes attached per call
    http_requests: Counter<u64>,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // Same TracingConfig fluent API as the OTLP example, swapping the metrics
    // backend for pull-based Prometheus scraping.
    let guard = init_tracing_opentelemetry::TracingConfig::production()
        .with_metrics_prometheus()
        .init_subscriber()?;
    let registry = guard
        .prometheus_registry()
        .expect("prometheus metrics enabled")
        .clone();

    let state = AppState {
        http_requests: global::meter("examples-axum-prometheus")
            .u64_counter("http_requests") // Prometheus exporter appends `_total`
            .build(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .layer(OtelInResponseLayer::default())
        .layer(OtelAxumLayer::default())
        .with_state(state)
        // exposes GET /metrics in Prometheus text format
        .merge(axum_tracing_opentelemetry::prometheus_metrics::router(
            registry,
        ));

    let addr = &"0.0.0.0:3003".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!("try `curl -i http://127.0.0.1:3003/`"); //Devskim: ignore DS137138
    tracing::info!("then `curl http://127.0.0.1:3003/metrics` to see dimensional counters:"); //Devskim: ignore DS137138
    tracing::info!("  http_requests_total{{method=\"GET\",path=\"/\"}} N"); //Devskim: ignore DS137138
    tracing::info!("  index_calls_total{{method=\"GET\",path=\"/\"}} N"); //Devskim: ignore DS137138
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status": "UP" }))
}

async fn index(
    State(state): State<AppState>,
    matched_path: Option<MatchedPath>,
    request: Request<axum::body::Body>,
) -> impl IntoResponse {
    let method = request.method().to_string();
    let path = matched_path.map_or_else(|| "/".to_string(), |p| p.as_str().to_string());

    // 1. Recommended: raw `opentelemetry::metrics::Meter` API, attaching
    //    attributes per call via `KeyValue`. No log line, no tracing coupling.
    state.http_requests.add(
        1,
        &[
            KeyValue::new("method", method.clone()),
            KeyValue::new("path", path.clone()),
        ],
    );

    // 2. Optional: tracing-macro bridge. Extra fields on the event become
    //    metric attributes via `MetricsLayer` (requires
    //    `TracingConfig::with_metrics*`). Only worth it when you already
    //    emit this event for logging and want to piggyback a metric on it.
    tracing::info!(monotonic_counter.index_calls = 1, method = %method, path = %path);

    axum::Json(json!({ "status": "UP" }))
}
