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

fn init_tracing() -> Result<(), axum::BoxError> {
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber = tracing_subscriber::registry();

    // register opentelemetry tracer layer
    let otel_layer = {
        use axum_tracing_opentelemetry::{
            init_propagator, //stdio,
            resource::DetectResource,
            otlp,
        };
        let otel_rsrc = DetectResource::default()
            .with_fallback_service_name(env!("CARGO_PKG_NAME"))
            .with_fallback_service_version(env!("CARGO_PKG_VERSION"))
            .with_println()
            .build();
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
async fn main() -> Result<(), axum::BoxError> {
    init_tracing()?;
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

## Configuration based on environment variable

To ease setup and compliancy with [Opentelemetry SDK configuration](https://opentelemetry.io/docs/concepts/sdk-configuration/general-sdk-configuration/), the configuration can be done with the following environment variables (see sample `init_tracing()` above):

- `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` fallback to `OTEL_EXPORTER_OTLP_ENDPOINT` for the url of the exporter / collector
- `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` fallback to `OTEL_EXPORTER_OTLP_PROTOCOL`, fallback to auto-detection based on ENDPOINT port
- `OTEL_SERVICE_NAME` for the name of the service
- `OTEL_PROPAGATORS` for the configuration of propagator
- `OTEL_TRACES_SAMPLER` & `OTEL_TRACES_SAMPLER_ARG` for configuration of the sampler

In the context of kubernetes, the above environment variable can be injected by the Opentelemetry operator (via inject-sdk):

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    metadata:
      annotations:
        # to inject environment variables only by opentelemetry-operator
        instrumentation.opentelemetry.io/inject-sdk: "opentelemetry-operator/instrumentation"
        instrumentation.opentelemetry.io/container-names: "app"
      containers:
        - name: app
```

Or if you don't setup inject-sdk, you can manually set the environment variable eg

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    metadata:
      containers:
        - name: app
          env:
            - name: OTEL_SERVICE_NAME
              value: "app"
            - name: OTEL_EXPORTER_OTLP_PROTOCOL
              value: "grpc"
            # for otel collector in `deployment` mode, use the name of the service
            # - name: OTEL_EXPORTER_OTLP_ENDPOINT
            #   value: "http://opentelemetry-collector.opentelemetry-collector:4317"
            # for otel collector in sidecar mode (imply to deploy a sidecar CR per namespace)
            - name: OTEL_EXPORTER_OTLP_ENDPOINT
              value: "http://localhost:4317"
            # for `daemonset` mode: need to use the local daemonset (value interpolated by k8s: `$(...)`)
            # - name: OTEL_EXPORTER_OTLP_ENDPOINT
            #   value: "http://$(HOST_IP):4317"
            # - name: HOST_IP
            #   valueFrom:
            #     fieldRef:
            #       fieldPath: status.hostIP
```

## `examples/otlp`

In a terminal, run

```sh
❯ cd examples/otlp
> # or direnv allow
❯ export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4317
❯ export OTEL_TRACES_SAMPLER=always_on
❯ cargo run
   Compiling examples-otlp v0.1.0 (/home/david/src/github.com/davidB/axum-tracing-opentelemetry/examples/otlp)
    Finished dev [unoptimized + debuginfo] target(s) in 2.96s
     Running `target/debug/examples-otlp`
     0.000170750s  WARN examples_otlp: listening on 0.0.0.0:3003
    at src/main.rs:70 on main

     0.000203401s  INFO examples_otlp: try to call `curl -i http://127.0.0.1:3003/` (with trace)
    at src/main.rs:71 on main

     0.000213920s  INFO examples_otlp: try to call `curl -i http://127.0.0.1:3003/heatlh` (with NO trace)
    at src/main.rs:72 on main
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

## For local dev / demo

To collect and visualize trace on local, one of the simplest solution:

```sh
# launch Jaeger with OpenTelemetry, Jaeger, Zipking,... mode.
# see https://www.jaegertracing.io/docs/1.41/getting-started/#all-in-one

# nerctl or docker or any container runner
nerdctl run --rm --name jaeger \
  -e COLLECTOR_ZIPKIN_HOST_PORT:9411 \
  -e COLLECTOR_OTLP_ENABLED:true \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 5778:5778 \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  -p 14250:14250 \
  -p 14268:14268 \
  -p 14269:14269 \
  -p 9411:9411 \
  jaegertracing/all-in-one:1.41

open http://localhost:16686
```

Then :

- setup env variable (or not), (eg see [.envrc](.envrc))
- launch your server
- send the request
- copy trace_id from log (or response header)
- paste into Jaeger web UI

## Compatibility

| axum | axum-tracing-opentelemetry |
|------|----------------------------|
| 0.6  | latest - 0.6               |
| 0.5  | 0.1 - 0.5                  |

## History

### 0.10

- breaking: default configuration for otlp Sampler is now longer hardcoded to `always_on`, but read environment variables `OTEL_TRACES_SAMPLER`, `OTEL_TRACES_SAMPLER_ARG`

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
