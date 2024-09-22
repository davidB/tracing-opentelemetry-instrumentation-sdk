# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
