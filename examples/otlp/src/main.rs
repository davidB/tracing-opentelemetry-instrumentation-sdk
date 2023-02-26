use axum::extract::Path;
use axum::{response::IntoResponse, routing::get, BoxError, Router};
use axum_tracing_opentelemetry::{opentelemetry_tracing_layer, response_with_trace_layer};
use serde_json::json;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // very opinionated init of tracing, look as is source to make your own
    axum_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    let app = app();
    // run it
    let addr = &"0.0.0.0:3003".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/` (with trace)"); //Devskim: ignore DS137138
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/health` (with NO trace)"); //Devskim: ignore DS137138
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    Router::new()
        .route(
            "/proxy/:service/*path",
            get(proxy_handler).post(proxy_handler),
        )
        .route("/", get(index)) // request processed inside span
        // include trace context as header into the response
        .layer(response_with_trace_layer())
        // opentelemetry_tracing_layer setup `TraceLayer`,
        // that is provided by tower-http so you have to add that as a dependency.
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health)) // request processed without span / trace
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}

async fn index() -> impl IntoResponse {
    let trace_id = axum_tracing_opentelemetry::find_current_trace_id();
    axum::Json(json!({ "my_trace_id": trace_id }))
}

async fn proxy_handler(Path((service, path)): Path<(String, String)>) -> impl IntoResponse {
    // Overwrite the otel.name of the span
    tracing::Span::current().record("otel.name", format!("proxy {service}"));
    let trace_id = axum_tracing_opentelemetry::find_current_trace_id();
    axum::Json(
        json!({ "my_trace_id": trace_id, "fake_proxy": { "service": service, "path": path } }),
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("signal received, starting graceful shutdown");
    opentelemetry::global::shutdown_tracer_provider();
}
