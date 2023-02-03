use axum::{response::IntoResponse, routing::get, BoxError, Router};
use axum_tracing_opentelemetry::{opentelemetry_tracing_layer, response_with_trace_layer};
use serde_json::json;
use std::net::SocketAddr;

fn init_tracing() -> Result<(), BoxError> {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber = tracing_subscriber::registry();

    // register opentelemetry tracer layer
    let otel_layer = {
        use axum_tracing_opentelemetry::{
            init_propagator, //stdio,
            make_resource,
            otlp,
        };
        let otel_rsrc = make_resource(
            std::env::var("OTEL_SERVICE_NAME")
                .or_else(|_| std::env::var("SERVICE_NAME"))
                .or_else(|_| std::env::var("APP_NAME"))
                .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
            std::env::var("SERVICE_VERSION")
                .or_else(|_| std::env::var("APP_VERSION"))
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
        );
        let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity)?;
        // to not send trace somewhere, but continue to create and propagate,...
        // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
        // or to `std::io::stdout()` to print
        //
        // let otel_tracer =
        //     stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())?;
        init_propagator()?;
        tracing_opentelemetry::layer().with_tracer(otel_tracer)
    };
    let subscriber = subscriber.with(otel_layer);

    // filter what is output on log (fmt), but not what is send to trace (opentelemetry collector)
    // std::env::set_var("RUST_LOG", "info,kube=trace");
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
            .unwrap_or_else(|_| "info".to_string()),
    );
    let subscriber = subscriber.with(EnvFilter::from_default_env());

    if cfg!(debug_assertions) {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_line_number(true)
            .with_thread_names(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_timer(tracing_subscriber::fmt::time::uptime());
        let subscriber = subscriber.with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_timer(tracing_subscriber::fmt::time::uptime());
        let subscriber = subscriber.with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber)?;
    };
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    init_tracing()?;
    let app = app();
    // run it
    let addr = &"0.0.0.0:3003".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/` (with trace)"); //Devskim: ignore DS137138
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/heatlh` (with NO trace)"); //Devskim: ignore DS137138
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    Router::new()
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
