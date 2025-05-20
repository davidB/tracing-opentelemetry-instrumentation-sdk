# `examples-axum-otlp`

In a terminal, run

Configure the [environment variables](https://opentelemetry.io/docs/languages/sdk-configuration/otlp-exporter/) for the OTLP exporter:

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

```sh
❯ cd examples/axum-otlp
❯ cargo run
   Compiling examples-axum-otlp v0.1.0 (/home/david/src/github.com/davidB/axum-tracing-opentelemetry/examples/axum-otlp)
    Finished dev [unoptimized + debuginfo] target(s) in 3.60s
     Running `/home/david/src/github.com/davidB/axum-tracing-opentelemetry/target/debug/examples-axum-otlp`
     0.000041809s  INFO init_tracing_opentelemetry::tracing_subscriber_ext: init logging & tracing
    at init-tracing-opentelemetry/src/tracing_subscriber_ext.rs:82 on main

     0.000221695s DEBUG otel::setup::resource: key: service.name, value: unknown_service
    at init-tracing-opentelemetry/src/resource.rs:63 on main

     0.000242183s DEBUG otel::setup::resource: key: os.type, value: linux
    at init-tracing-opentelemetry/src/resource.rs:63 on main

     0.000280946s DEBUG otel::setup: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: "http://localhost:4317"
    at init-tracing-opentelemetry/src/otlp.rs:22 on main

     0.000293128s DEBUG otel::setup: OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: "grpc"
    at init-tracing-opentelemetry/src/otlp.rs:23 on main

     0.000377897s DEBUG otel::setup: OTEL_TRACES_SAMPLER: "always_on"
    at init-tracing-opentelemetry/src/otlp.rs:80 on main

     0.000561931s DEBUG otel::setup: OTEL_PROPAGATORS: "tracecontext,baggage"
    at init-tracing-opentelemetry/src/lib.rs:97 on main

     0.000134291s  WARN examples_axum_otlp: listening on 0.0.0.0:3003
    at examples/axum-otlp/src/main.rs:15 on main

     0.000150401s  INFO examples_axum_otlp: try to call `curl -i http://127.0.0.1:3003/` (with trace)
    at examples/axum-otlp/src/main.rs:16 on main

     0.000159659s  INFO examples_axum_otlp: try to call `curl -i http://127.0.0.1:3003/health` (with NO trace)
    at examples/axum-otlp/src/main.rs:17 on main
...
```

Into an other terminal, call the `/` (endpoint with `OtelAxumLayer` and `OtelInResponseLayer`)

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
