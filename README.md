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

fn init_tracing() {
    use axum_tracing_opentelemetry::{
        make_resource,
        otlp,
        //stdio,
    };

    let otel_rsrc = make_resource(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");
    // let otel_tracer =
    //     stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())
    //         .expect("setup of Tracer");
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
        .route("/", get(health)) // request processed inside span
        // opentelemetry_tracing_layer setup `TraceLayer`, that is provided by tower-http so you have to add that as a dependency.
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health)) // request processed without span / trace
}

async fn shutdown_signal() {
    //...
    opentelemetry::global::shutdown_tracer_provider();
}
```

To retrieve the current `trace_id` (eg to add it into error message (as header or attributes))

```rust
  let trace_id = axum_tracing_opentelemetry::find_current_trace_id();
  json!({ "error" :  "xxxxxx", "trace_id": trace_id})
```

## Compatibility

|axum|axum-tracing-opentelemetry|
|----|--------------------------|
|0.6 | latest - 0.6             |
|0.5 | 0.1 - 0.5                |

## History

### 0.6

- upgrade to axum 0.6

### 0.5

- upgrade to opentelemetry 0.18
- breaking change: upgrade opentelemetry-jaeger to 0.17 (switch from PipelineBuiler to AgentPipeline)

### 0.4

- allow customization of tracer
- add tracer to export on stdout or stderr
- add tracer to export to nowhere (like `/dev/null`) to allow to have trace_id
  and the opentelemetry span & metadata on log and http response (without collector)

### 0.3

- Allow customization of exporter pipeline
- Fix name of the root span (#6)

### 0.2

- First public release as a crate

### 0.1

- Code originally created at part of axum-extra [Add OpenTelemetry middleware by davidpdrsn · Pull Request #769 · tokio-rs/axum](https://github.com/tokio-rs/axum/pull/769)
- Code copied and modified as part of [davidB/sandbox_axum_observability: Sandbox to experiment axum and observability](https://github.com/davidB/sandbox_axum_observability)
- Published as a standalone crate with OK from original author
