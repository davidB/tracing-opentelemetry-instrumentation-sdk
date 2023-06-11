# toolkit-tracing-opentelemetry

A set of rust crates to help working with tracing + opentelemetry

- `init-tracing-opentelemetry`: A set of helpers to initialize (and more) tracing + opentelemetry (compose your own or use opinionated preset)
- `axum-tracing-opentelemetry`: Middlewares and tools to integrate axum + tracing + opentelemetry.
- `fake-opentelemetry-collector`: A Fake (basic) opentelemetry collector, useful to test what is collected opentelemetry

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
