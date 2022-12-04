use axum::{response::IntoResponse, routing::get, Router};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use serde_json::json;
use std::net::SocketAddr;

fn init_tracing() {
    use axum_tracing_opentelemetry::{
        make_resource,
        otlp,
        //stdio,
    };
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::layer::SubscriberExt;
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or("INFO".to_string()),
    );

    let otel_rsrc = make_resource(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");
    // let otel_tracer =
    //     stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())
    //         .expect("setup of Tracer");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    // Build a subscriber that combines the access log and stdout log
    // layers.
    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_default_env())
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let app = app();
    // run it
    let addr = &"0.0.0.0:3000".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    Router::new()
        .route("/", get(health))  // request processed inside span
        // opentelemetry_tracing_layer setup `TraceLayer`,
        // that is provided by tower-http so you have to add that as a dependency.
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health)) // request processed without span / trace
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
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
