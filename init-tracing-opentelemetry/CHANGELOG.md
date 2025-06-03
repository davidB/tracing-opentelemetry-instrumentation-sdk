# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.28.2](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.28.1...init-tracing-opentelemetry-v0.28.2) - 2025-06-03

### <!-- 1 -->Fixed

- *(deps)* missing `tonic` dependency on `tls`

### <!-- 3 -->Removed

- remove deprecated sample from README

## [0.28.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.27.1...init-tracing-opentelemetry-v0.28.0) - 2025-03-31

### <!-- 2 -->Added

- *(deps)* update opentelemetry to 0.29 (#227)

## [0.27.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.27.0...init-tracing-opentelemetry-v0.27.1) - 2025-02-26

### <!-- 1 -->Fixed

- reqwest must use blocking client since opentelemetry 0.28 (#220)

### <!-- 3 -->Removed

- *(deps)* remove minor constraint when major > 1

## [0.27.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.26.0...init-tracing-opentelemetry-v0.27.0) - 2025-02-24

### <!-- 1 -->Fixed

- drop on the TracingGuard also shutdown the wrapped TracerProvider

### <!-- 2 -->Added

- allow to provide log's "directives" via `init_subscribers_and_loglevel`

## [0.25.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.24.1...init-tracing-opentelemetry-v0.25.0) - 2024-12-10

### <!-- 1 -->Fixed

- inference of `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` for protocol `http/protobuf`

## [0.24.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.24.0...init-tracing-opentelemetry-v0.24.1) - 2024-11-24

### <!-- 1 -->Fixed

- Use guard pattern to allow consumers to ensure final trace is sent ([#185](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/185))

## [0.24.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.21.0...init-tracing-opentelemetry-v0.24.0) - 2024-09-23

### <!-- 2 -->Added

- [**breaking**] remove trace_id and span_id from logfmt (to avoid link with old version)

## [0.21.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.20.0...init-tracing-opentelemetry-v0.21.0) - 2024-09-22

### <!-- 2 -->Added

- *(deps)* upgrade to opentelemetry 0.25
- add a troubleshot section
- [**breaking**] disable resourcedetector (os,...) until update for new version of opentelemetry
- [**breaking**] disable support of xray (until update for new version of opentelemetry)

## [0.20.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.19.0...init-tracing-opentelemetry-v0.20.0) - 2024-08-31

### <!-- 1 -->Fixed
- 🐛 fix build of contributions (upgrade of opentelemetry, fake collector for logs,...)

### <!-- 4 -->Changed
- ⬆️ upgrade to rstest 0.22
- ⬆️ upgrade to opentelemetry 0.24 (and related dependencies) ([#151](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/151))

## [0.17.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.17.0...init-tracing-opentelemetry-v0.17.1) - 2024-02-24

### Other
- 👷 tune release-plz
