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

    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
            .unwrap_or_else(|_| "INFO".to_string()),
    );

    let otel_rsrc = make_resource(
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
        env!("CARGO_PKG_VERSION"),
    );
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

To also inject the trace id into the response (could be useful for debugging) uses the layer `response_with_trace_layer`

```rust
    // build our application with a route
    Router::new()
        ...
        // include trace context as header into the response
        .layer(response_with_trace_layer())
```

## `examples/otlp`

In a terminal, run

```sh
❯ cd examples/otlp
❯ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
    Running `target/debug/examples-otlp`
{"timestamp":"   0.007110513s","level":"WARN","fields":{"message":"listening on 0.0.0.0:3003"},"target":"examples_otlp"}
{"timestamp":"   0.007163973s","level":"INFO","fields":{"message":"try to call `curl -i http://127.0.0.1:3003/` (with trace)"},"target":"examples_otlp"}
{"timestamp":"   0.007181296s","level":"INFO","fields":{"message":"try to call `curl -i http://127.0.0.1:3003/heatlh` (with NO trace)"},"target":"examples_otlp"}
...
```

Into an other terminal, call the `/` (endpoint with `opentelemetry_tracing_layer` and `response_with_trace_layer`)

```sh
❯ curl -i http://127.0.0.1:3003/
HTTP/1.1 200 OK
content-type: application/json
content-length: 50
traceparent: 00-b2611246a58fd7ea623d2264c5a1e226-b2c9b811f2f424af-01
tracestate:
date: Wed, 28 Dec 2022 17:04:59 GMT

{"my_trace_id":"b2611246a58fd7ea623d2264c5a1e226"}
```

call the `/health` (endpoint with NO layer)

```sh
❯ curl -i http://127.0.0.1:3003/health
HTTP/1.1 200 OK
content-type: application/json
content-length: 15
date: Wed, 28 Dec 2022 17:14:07 GMT

{"status":"UP"}
```

## Compatibility

|axum|axum-tracing-opentelemetry|
|----|--------------------------|
|0.6 | latest - 0.6             |
|0.5 | 0.1 - 0.5                |

## History

### 0.7

- add a layer`response_with_trace_layer` to have `traceparent` injected into response
- improve discovery of otlp configuration based on `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL`, `OTEL_EXPORTER_OTLP_PROTOCOL`

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
