# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.30.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.29.1...tracing-opentelemetry-instrumentation-sdk-v0.30.0) - 2025-08-25

### <!-- 2 -->Added

- *(axum)* optional extraction of `client.address` (former `client_ip`) from http headers or socket's info
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.32.2](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.32.1...tracing-opentelemetry-instrumentation-sdk-v0.32.2) - 2025-11-13

### <!-- 2 -->Added

- add in features to docs.rs rendered content. ([#287](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/287))

## [0.32.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.32.0...tracing-opentelemetry-instrumentation-sdk-v0.32.1) - 2025-10-14

### Wip

- use `opentelemetry-semantic-conventions` instead of `static &str`

## [0.31.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.30.0...tracing-opentelemetry-instrumentation-sdk-v0.31.0) - 2025-09-27

### <!-- 2 -->Added

- [**breaking**] export grpc utils from `http::grpc` module

## [0.29.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.28.1...tracing-opentelemetry-instrumentation-sdk-v0.29.0) - 2025-06-03

### <!-- 2 -->Added

- *(deps)* update opentelemetry 0.30 & tonic 0.13 (#240)

## [0.24.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.19.0...tracing-opentelemetry-instrumentation-sdk-v0.24.0) - 2024-08-31

### <!-- 4 -->Changed
- ⬆️ upgrade to tonic 0.12
- ⬆️ upgrade to rstest 0.22

## [0.18.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.18.0...tracing-opentelemetry-instrumentation-sdk-v0.18.1) - 2024-04-24

### <!-- 2 -->Added
- ✨ allow to create span for opentelemetry at level `info` with feature flag `tracing_level_info`

## [0.17.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tracing-opentelemetry-instrumentation-sdk-v0.17.0...tracing-opentelemetry-instrumentation-sdk-v0.17.1) - 2024-02-24

### Other
- 👷 tune release-plz
