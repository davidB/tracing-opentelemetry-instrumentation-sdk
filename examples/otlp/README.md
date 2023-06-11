# `examples/otlp`

In a terminal, run

```sh
❯ cd examples/otlp
> # or direnv allow
❯ export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT=http://localhost:4317
❯ export OTEL_TRACES_SAMPLER=always_on
❯ cargo run
warning: `axum-tracing-opentelemetry` (lib) generated 1 warning
   Compiling examples-otlp v0.1.0 (/home/david/src/github.com/davidB/axum-tracing-opentelemetry/examples/otlp)
    Finished dev [unoptimized + debuginfo] target(s) in 2.38s
      Running `/home/david/src/github.com/davidB/axum-tracing-opentelemetry/examples/target/debug/examples-otlp`
      0.000043962s  INFO axum_tracing_opentelemetry::tools::tracing_subscriber_ext: init logging & tracing
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/tracing_subscriber_ext.rs:82 on main

      0.000472423s DEBUG otel::resource: key: service.name, value: unknown_service
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/resource.rs:84 on main

     0.000213920s  INFO examples_otlp: try to call `curl -i http://127.0.0.1:3003/health` (with NO trace)
    at src/main.rs:72 on main
      0.000501468s DEBUG otel::resource: key: os.type, value: linux
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/resource.rs:84 on main

      0.000549097s DEBUG otel::setup: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: "http://localhost:4317"
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/otlp.rs:22 on main

      0.000570727s DEBUG otel::setup: OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: "grpc"
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/otlp.rs:23 on main

      0.000623135s DEBUG otel::setup: OTEL_TRACES_SAMPLER: "always_on"
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/otlp.rs:80 on main

      0.000928215s DEBUG otel::setup: OTEL_PROPAGATORS: "tracecontext,baggage"
    at /home/david/src/github.com/davidB/axum-tracing-opentelemetry/src/tools/mod.rs:94 on main

      0.000190306s  WARN examples_otlp: listening on 0.0.0.0:3003
    at otlp/src/main.rs:15 on main

      0.000222566s  INFO examples_otlp: try to call `curl -i http://127.0.0.1:3003/` (with trace)
    at otlp/src/main.rs:16 on main

      0.000240970s  INFO examples_otlp: try to call `curl -i http://127.0.0.1:3003/heatlh` (with NO trace)
    at otlp/src/main.rs:17 on main
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
