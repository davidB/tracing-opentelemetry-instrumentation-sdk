# tracing-opentelemetry-instrumentation-sdk

A set of rust crates to help working with tracing + opentelemetry

- `init-tracing-opentelemetry`: A set of helpers to initialize (and more) tracing + opentelemetry (compose your own or use opinionated preset)
- `axum-tracing-opentelemetry`: Middlewares and tools to integrate axum + tracing + opentelemetry.
- `fake-opentelemetry-collector`: A Fake (basic) opentelemetry collector, useful to test what is collected opentelemetry

## For local dev / demo

To collect and visualize trace on local, some ofthe simplest solutions:

### Otel Desktop Viewer

[CtrlSpice/otel-desktop-viewer: desktop-collector](https://github.com/CtrlSpice/otel-desktop-viewer)

```sh
# also available via `brew install --cask ctrlspice/tap/otel-desktop-viewer`
# For AMD64 (most common)
# docker/nerdctl/podman or any container runner
docker run -p 8000:8000 -p 4317:4317 -p 4318:4318 ghcr.io/ctrlspice/otel-desktop-viewer:latest-amd64

open http://localhost:8000
```

### Jaeger all-in-one

```sh
# launch Jaeger with OpenTelemetry, Jaeger, Zipking,... mode.
# see https://www.jaegertracing.io/docs/1.49/getting-started/#all-in-one

# docker/nerdctl/podman or any container runner
docker run --rm --name jaeger \
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
  jaegertracing/all-in-one:1.49

open http://localhost:16686
```

Then :

- setup env variable (or not), (eg see [.envrc](.envrc))
- launch your server
- send the request
- copy trace_id from log (or response header)
- paste into Jaeger web UI

## To release

Use the github workflow `release-plz`.
