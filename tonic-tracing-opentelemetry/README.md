# axum-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/tonic-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/tonic-tracing-opentelemetry.svg)](https://crates.io/crates/tonic-tracing-opentelemetry)

[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Middlewares and tools to integrate tonic + tracing + opentelemetry.

> Really early, missing lot of features, help is welcomed.

- Read OpenTelemetry header from the incoming requests
- Start a new trace if no trace is found in the incoming request
- Trace is attached into tracing'span

## TODO

- layer for client
- add test
- add documentation
- add examples
- validate with [[opentelemetry-specification/rpc.md at main · open-telemetry/opentelemetry-specification · GitHub](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md#grpc)]

## Changelog - History

### 0.12

- 💥 extracted from axum-tracing-opentelemetry 0.11
