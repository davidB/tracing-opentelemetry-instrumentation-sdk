#![allow(clippy::let_with_type_underscore)]
#![allow(clippy::default_constructed_unit_structs)] // warning since 1.71

use axum::extract::Path;
use axum::{response::IntoResponse, routing::get, BoxError, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use serde_json::json;
use std::net::SocketAddr;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // very opinionated init of tracing, look as is source to make your own
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    let app = app();
    // run it
    let addr = &"0.0.0.0:3003".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/` (with trace)"); //Devskim: ignore DS137138
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/health` (with NO trace)"); //Devskim: ignore DS137138
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    Router::new()
        .route(
            "/proxy/{service}/{*path}",
            get(proxy_handler).post(proxy_handler),
        )
        .route("/", get(index)) // request processed inside span
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .route("/health", get(health)) // request processed without span / trace
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}

#[tracing::instrument]
async fn index() -> impl IntoResponse {
    let trace_id = find_current_trace_id();
    dbg!(&trace_id);
    //std::thread::sleep(std::time::Duration::from_secs(1));
    axum::Json(json!({ "my_trace_id": trace_id }))
}

async fn proxy_handler(Path((service, path)): Path<(String, String)>) -> impl IntoResponse {
    // Overwrite the otel.name of the span
    tracing::Span::current().record("otel.name", format!("proxy {service}"));
    let trace_id = find_current_trace_id();
    axum::Json(
        json!({ "my_trace_id": trace_id, "fake_proxy": { "service": service, "path": path } }),
    )
}
