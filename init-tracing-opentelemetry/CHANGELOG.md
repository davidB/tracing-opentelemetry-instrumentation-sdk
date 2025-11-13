# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.34.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.33.0...init-tracing-opentelemetry-v0.34.0) - 2025-11-13

### <!-- 1 -->Fixed

- *(init-tracing-opentelemetry)* apply custom resource configuration to OpenTelemetry layers ([#297](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/297))
- Fix timers, booleans add support for `Layer::without_time` ([#295](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/295))

### <!-- 2 -->Added

- Add support for `Layer::with_thread_ids`
- Add support for `Layer::with_file`
- add support for `tracing_subscriber::fmt::format::Full` ([#291](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/291))
- add in features to docs.rs rendered content. ([#287](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/287))

## [0.33.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.32.1...init-tracing-opentelemetry-v0.33.0) - 2025-11-02

### <!-- 2 -->Added

- allow to customize the tracing configuration (registry) with additional layers ([#281](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/281))

## [0.31.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.30.1...init-tracing-opentelemetry-v0.31.0) - 2025-09-27

### <!-- 2 -->Added

- [**breaking**] Guard struct allow future evolution, init_subscriber can be used for non global (like test,...)
- a more configurable tracing configuration with `TracingConfig`

## [0.30.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/init-tracing-opentelemetry-v0.29.0...init-tracing-opentelemetry-v0.30.0) - 2025-07-18

### <!-- 2 -->Added

- add support for Opentelemetry Metrics (#249)

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
