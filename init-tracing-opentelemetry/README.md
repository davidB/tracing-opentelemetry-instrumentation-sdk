# init-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/init-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/init-tracing-opentelemetry.svg)](https://crates.io/crates/init-tracing-opentelemetry)

A set of helpers to initialize (and more) tracing + opentelemetry (compose your own or use opinionated preset)

```txt
//...
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;

#[tokio::main]
async fn main() -> Result<(), axum::BoxError> {
    // very opinionated init of tracing, look as is source to compose your own
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    ...;

    Ok(())
}
```

The `init_subscribers` function returns a `TracingGuard` instance. Following the guard pattern, this struct provides no functions but, when dropped, ensures that any pending traces are sent before it exits. The syntax `let _guard` is suggested to ensure that Rust does not drop the struct until the application exits.

To configure opentelemetry tracer & tracing, you can use the functions from `init_tracing_opentelemetry::tracing_subscriber_ext`, but they are very opinionated (and WIP to make them more customizable and friendly), so we recommend making your composition, but look at the code (to avoid some issue) and share your feedback.

```txt
pub fn build_loglevel_filter_layer() -> tracing_subscriber::filter::EnvFilter {
    // filter what is output on log (fmt)
    // std::env::set_var("RUST_LOG", "warn,axum_tracing_opentelemetry=info,otel=debug");
    std::env::set_var(
        "RUST_LOG",
        format!(
            // `otel::tracing` should be a level trace to emit opentelemetry trace & span
            // `otel::setup` set to debug to log detected resources, configuration read and infered
            "{},otel::tracing=trace,otel=debug",
            std::env::var("RUST_LOG")
                .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
                .unwrap_or_else(|_| "info".to_string())
        ),
    );
    EnvFilter::from_default_env()
}

pub fn build_otel_layer<S>() -> Result<OpenTelemetryLayer<S, Tracer>, BoxError>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    use crate::{
        init_propagator, //stdio,
        otlp,
        resource::DetectResource,
    };
    let otel_rsrc = DetectResource::default()
        //.with_fallback_service_name(env!("CARGO_PKG_NAME"))
        //.with_fallback_service_version(env!("CARGO_PKG_VERSION"))
        .build();
    let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity)?;
    // to not send trace somewhere, but continue to create and propagate,...
    // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
    // or to `std::io::stdout()` to print
    //
    // let otel_tracer =
    //     stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())?;
    init_propagator()?;
    Ok(tracing_opentelemetry::layer().with_tracer(otel_tracer))
}
```

To retrieve the current `trace_id` (eg to add it into error message (as header or attributes))

```rust
  # use tracing_opentelemetry_instrumentation_sdk;

  let trace_id = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
  //json!({ "error" :  "xxxxxx", "trace_id": trace_id})
```

## Configuration based on the environment variables

To ease setup and compliance with [OpenTelemetry SDK configuration](https://opentelemetry.io/docs/concepts/sdk-configuration/general-sdk-configuration/), the configuration can be done with the following environment variables (see sample `init_tracing()` above):

- `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` fallback to `OTEL_EXPORTER_OTLP_ENDPOINT` for the url of the exporter / collector
- `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` fallback to `OTEL_EXPORTER_OTLP_PROTOCOL`, fallback to auto-detection based on ENDPOINT port
- `OTEL_SERVICE_NAME` for the name of the service
- `OTEL_PROPAGATORS` for the configuration of the propagators
- `OTEL_TRACES_SAMPLER` & `OTEL_TRACES_SAMPLER_ARG` for configuration of the sampler

Few other environment variables can also be used to configure OTLP exporter (eg to configure headers, authentication,, etc...):

- [`OTEL_EXPORTER_OTLP_HEADERS`](https://opentelemetry.io/docs/languages/sdk-configuration/otlp-exporter/#otel_exporter_otlp_headers)
- [`OTEL_EXPORTER_OTLP_TRACES_HEADERS`](https://opentelemetry.io/docs/languages/sdk-configuration/otlp-exporter/#otel_exporter_otlp_traces_headers)

```sh
# For GRPC:
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4317"
export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL="grpc"
export OTEL_TRACES_SAMPLER="always_on"

# For HTTP:
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4318/v1/traces"
export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL="http/protobuf"
export OTEL_TRACES_SAMPLER="always_on"
```

In the context of **kubernetes**, some of the above environment variables can be injected by the Opentelemetry operator (via `inject-sdk`):

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

Or if you don't setup `inject-sdk`, you can manually set the environment variable eg

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

## Troubleshot why no trace?

- check you only have a single version of opentelemtry (could be part of your CI/build), use `cargo-deny` or `cargo tree`

    ```sh
    # Check only one version of opentelemetry should be used
    # else issue with setup of global (static variable)
    # check_single_version_opentelemtry:
    cargo tree -i opentelemetry
    ```

- check the code of your exporter and the integration with `tracing` (as subscriber's layer)
- check the environment variables of opentelemetry `OTEL_EXPORTER...` and `OTEL_TRACES_SAMPLER` (values are logged on target `otel::setup` )
- check that log target `otel::tracing` enable log level `trace` (or `info` if you use `tracing_level_info` feature) to generate span to send to opentelemetry collector.

## Changelog - History

[CHANGELOG.md](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/main/CHANGELOG.md)
