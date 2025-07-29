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

## [0.29.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.28.1...tonic-tracing-opentelemetry-v0.29.0) - 2025-06-03

### <!-- 2 -->Added

- *(deps)* update opentelemetry 0.30 & tonic 0.13 (#240)

## [0.26.1](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.26.0...tonic-tracing-opentelemetry-v0.26.1) - 2025-02-26

### <!-- 3 -->Removed

- *(deps)* remove minor constraint when major > 1

## [0.24.3](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.24.2...tonic-tracing-opentelemetry-v0.24.3) - 2025-01-07

### <!-- 1 -->Fixed

- Implement tower::Service for OtelGrpcService (#201)

## [0.21.0](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.19.0...tonic-tracing-opentelemetry-v0.21.0) - 2024-08-31

### <!-- 4 -->Changed
- 💄 update deprecated syntax "default_features" in Cargo.toml
- ⬆️ upgrade to tonic 0.12
- ⬆️ upgrade to rstest 0.22

## [0.18.2](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.18.1...tonic-tracing-opentelemetry-v0.18.2) - 2024-04-24

### <!-- 2 -->Added
- ✨ allow to create span for opentelemetry at level `info` with feature flag `tracing_level_info`
