# init-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/init-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/init-tracing-opentelemetry.svg)](https://crates.io/crates/init-tracing-opentelemetry)

A set of helpers to initialize (and more) tracing + opentelemetry (compose your own or use opinionated preset)

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple preset
    let _guard = init_tracing_opentelemetry::TracingConfig::production().init_subscriber()?;

    //...

    Ok(())
}
```

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // custom configuration
    let _guard = init_tracing_opentelemetry::TracingConfig::default()
        .with_json_format()
        .with_stderr()
        .with_log_directives("debug")
        .init_subscriber()?;

    //...

    Ok(())
}
```

The `init_subscriber()` function returns an `OtelGuard` instance. Following the guard pattern, this struct provides no functions but, when dropped, ensures that any pending traces/metrics are sent before it exits. The syntax `let _guard` is suggested to ensure that Rust does not drop the struct until the application exits.

## Configuration Options

### Presets

- `TracingConfig::development()` - Pretty format, stderr, with debug info
- `TracingConfig::production()` - JSON format, stdout, minimal metadata
- `TracingConfig::debug()` - Full verbosity with all span events
- `TracingConfig::minimal()` - Compact format, no OpenTelemetry
- `TracingConfig::testing()` - Minimal output for tests

### Custom Configuration

```rust,no_run
use init_tracing_opentelemetry::TracingConfig;

TracingConfig::default()
    .with_pretty_format()           // or .with_json_format(), .with_compact_format()
    .with_stderr()                  // or .with_stdout(), .with_file(path)
    .with_log_directives("debug")   // Custom log levels
    .with_line_numbers(true)        // Include line numbers
    .with_thread_names(true)        // Include thread names
    .with_otel(true)                // Enable OpenTelemetry
    .init_subscriber()
    .expect("valid tracing configuration");
```

### Legacy API (deprecated)

For backward compatibility, the old API is still available:

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

## Metrics

To configure opentelemetry metrics, enable the `metrics` feature, this will initialize a `SdkMeterProvider`, set it globally and add a a [`MetricsLayer`](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/struct.MetricsLayer.html) to allow using `tracing` events to produce metrics.

The `opentelemetry_sdk` can still be used to produce metrics as well, since we configured the `SdkMeterProvider` globally, so any Axum/Tonic middleware that does not use `tracing` but directly [opentelemetry::metrics](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/struct.Meter.html) will work.

Configure the following set of environment variables to configure the metrics exporter (on top of those configured above):

- `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` override to `OTEL_EXPORTER_OTLP_ENDPOINT` for the url of the exporter / collector
- `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL` override to `OTEL_EXPORTER_OTLP_PROTOCOL`, fallback to auto-detection based on ENDPOINT port
- `OTEL_EXPORTER_OTLP_METRICS_TIMEOUT` to set the timeout for the connection to the exporter
- `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` to set the temporality preference for the exporter
- `OTEL_METRIC_EXPORT_INTERVAL` to set frequence of metrics export in **_milliseconds_**, defaults to 60s

## Changelog - History

[CHANGELOG.md](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/main/CHANGELOG.md)
