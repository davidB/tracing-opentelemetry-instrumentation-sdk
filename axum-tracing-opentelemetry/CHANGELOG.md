# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.30.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.29.0...axum-tracing-opentelemetry-v0.30.0) - 2025-08-25

### <!-- 2 -->Added

- *(axum)* optional extraction of `client.address` (former `client_ip`) from http headers or socket's info
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.32.2](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.32.1...axum-tracing-opentelemetry-v0.32.2) - 2025-11-03

### <!-- 4 -->Changed

- update sample

## [0.32.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.32.0...axum-tracing-opentelemetry-v0.32.1) - 2025-10-14

### Wip

- use `opentelemetry-semantic-conventions` instead of `static &str`

## [0.30.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.30.0...axum-tracing-opentelemetry-v0.30.1) - 2025-09-27

## [0.28.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.28.0...axum-tracing-opentelemetry-v0.28.1) - 2025-06-03

### <!-- 3 -->Removed

- remove deprecated sample from README

## [0.26.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.26.0...axum-tracing-opentelemetry-v0.26.1) - 2025-02-26

### <!-- 3 -->Removed

- *(deps)* remove minor constraint when major > 1

## [0.25.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.24.2...axum-tracing-opentelemetry-v0.25.0) - 2025-01-02

### <!-- 2 -->Added

- *(deps)* update rust crate axum to 0.8 (#197)

## [0.24.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.24.0...axum-tracing-opentelemetry-v0.24.1) - 2024-11-24

### <!-- 1 -->Fixed

- Use guard pattern to allow consumers to ensure final trace is sent ([#185](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/185))

## [0.21.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.19.0...axum-tracing-opentelemetry-v0.21.0) - 2024-08-31

### <!-- 1 -->Fixed
- 🐛 workaround for a delay, batch,... behavior in otlp exporter and test with fake-opentelemetry-collector (closed too early)
- 🐛 fix build of contributions (upgrade of opentelemetry, fake collector for logs,...)
- 🐛  Re-export tracing_level_info feature from axum to sdk ([#147](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/147))

### <!-- 4 -->Changed
- 💄 update deprecated syntax "default_features" in Cargo.toml
- ⬆️ upgrade to rstest 0.22

## [0.17.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/axum-tracing-opentelemetry-v0.17.0...axum-tracing-opentelemetry-v0.17.1) - 2024-02-24

### Other
- 👷 tune release-plz
- ✏️ Fix broken /examples URLs ([#129](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/129))
