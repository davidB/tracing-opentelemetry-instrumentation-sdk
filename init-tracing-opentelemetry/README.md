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
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    ...;

    Ok(())
}
```

AND Call `opentelemetry_api::global::shutdown_tracer_provider();` on shutdown of the app to be sure to send the pending trace,...

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

In the context of kubernetes, the above environment variable can be injected by the Opentelemetry operator (via `inject-sdk`):

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

## Changelog - History

### 0.13

- ⬆️ upgrade to opentelemetry 0.20 (and related dependencies)
- 💥 stdio tracer moved under feature flags "sdout" and change type to reflect change into opentelemetry

### 0.12

- 💥 extracted from axum-tracing-opentelemetry
