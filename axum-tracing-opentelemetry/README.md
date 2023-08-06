# axum-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/axum-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/axum-tracing-opentelemetry.svg)](https://crates.io/crates/axum-tracing-opentelemetry)

[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Middlewares to integrate axum + tracing + opentelemetry.

- Read OpenTelemetry header from incoming request
- Start a new trace if no trace found in the incoming request
- Trace is attached into tracing'span
- OpenTelemetry Span is created on close of the tracing's span (behavior from [tracing-opentelemetry])

For examples, you can look at the [examples](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/examples/) folder.

```txt
//...
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;

#[tokio::main]
async fn main() -> Result<(), axum::BoxError> {
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

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
    Router::new()
        .route("/", get(index)) // request processed inside span
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .route("/health", get(health)) // request processed without span / trace
}

async fn shutdown_signal() {
    //...
    opentelemetry::global::shutdown_tracer_provider();
}
```

For more info about how to initialize, you can look at crate [`init-tracing-opentelemetry`] or [`tracing-opentelemetry`].

## Compatibility

| axum | axum-tracing-opentelemetry |
|------|----------------------------|
| 0.6  | latest - 0.6               |
| 0.5  | 0.1 - 0.5                  |

## Changelog - History

### 0.13

- ⬆️ upgrade to opentelemetry 0.20 (and related dependencies)

### 0.12

- 💥 upgrade opentelemetry attributes to follow semantic 1.22
- 💥 extract tools, tonic,... into separate crates [`init-tracing-opentelemetry`], [`tonic-tracing-opentelemetry`], [`tracing-opentelemetry-instrumentation-sdk`], without re-export and features
- 💥 remove `trace_id` from attributes (opnetelemetry) and field in trace (log,...) on creation
  because the previous workaround created invalid states in some context
- deprecate factory `opentelemetry_tracing_layer`, `response_with_trace_layer`
- full rewrite without tower-http/Tracing

### 0.11

- upgrade to opentelemetry 0.19

### 0.10

- 💥 default configuration for otlp Sampler is no longer hardcoded to `always_on`, but read environment variables `OTEL_TRACES_SAMPLER`, `OTEL_TRACES_SAMPLER_ARG`
- ✨ provide opinionated `tracing_subscriber_ext`
- ✨ log under target `otel::setup` detected configuration by otel setup tools
- ✨ add a axum layer for gRPC (#36) (wip)

### 0.9

- add `DetectResource` builder to help detection for [Resource Semantic Conventions | OpenTelemetry](https://opentelemetry.io/docs/reference/specification/resource/semantic_conventions/#semantic-attributes-with-sdk-provided-default-value)

### 0.8

- add `init_propagator` to configure the global propagator based on content of the env variable [OTEL_PROPAGATORS](https://opentelemetry.io/docs/concepts/sdk-configuration/general-sdk-configuration/#otel_propagators)

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

[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[`init-tracing-opentelemetry`]: https://crates.io/crates/init-tracing-opentelemetry
[`tonic-tracing-opentelemetry`]: https://crates.io/crates/tonic-tracing-opentelemetry
[`tracing-opentelemetry-instrumentation-sdk`]: https://crates.io/crates/tracing-opentelemetry-instrumentation-sdk
