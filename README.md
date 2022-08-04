# axum-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/axum-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/axum-tracing-opentelemetry.svg)](https://crates.io/crates/axum-tracing-opentelemetry)

[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Middlewares and tools to integrate axum + tracing + opentelemetry.

- Read OpenTelemetry header from incoming request
- Start a new trace if no trace found in the incoming request
- Trace is attached into tracing'span

For examples, you can look at:

- the [examples](./examples/) folder
- [davidB/sandbox_axum_observability: Sandbox to experiment axum and observability](https://github.com/davidB/sandbox_axum_observability). This example shows also propagation of the trace between tracing span and service (via reqwest).

```rust
//...
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use axum_tracing_opentelemetry::{
    // optional tools to init tracer (may require features)
    init_tracer,
    make_resource,
    CollectorKind,
};

fn init_tracing() {

    let otel_tracer = init_tracer(
        CollectorKind::Otlp,
        make_resource(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
    )
    .expect("setup of Tracer");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    let subscriber = tracing_subscriber::registry()
        //...
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
        .route("/health", get(health))
        // opentelemetry_tracing_layer setup `TraceLayer`, that is provided by tower-http so you have to add that as a dependency.
        .layer(opentelemetry_tracing_layer())
}

async fn shutdown_signal() {
    //...
    opentelemetry::global::shutdown_tracer_provider();
}
```

## History

### 0.2.0

- first public release as a crate

### 0.1.0

- Code originally created at part of axum-extra [Add OpenTelemetry middleware by davidpdrsn · Pull Request #769 · tokio-rs/axum](https://github.com/tokio-rs/axum/pull/769)
- Code copied and modified as part of [davidB/sandbox_axum_observability: Sandbox to experiment axum and observability](https://github.com/davidB/sandbox_axum_observability)
- Published as a standalone crate with OK from original author
