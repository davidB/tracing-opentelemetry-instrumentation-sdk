# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.18.3](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.18.2...tonic-tracing-opentelemetry-v0.18.3) - 2024-06-30

### <!-- 1 -->Fixed
- 🐛 Add tracing_level_info feature to tonic-tracing crate ([#146](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/146))

### <!-- 4 -->Changed
- ⬆️ upgrade to rstest 0.21
- ✏️ Fix the header of the tonic tracing README.md ([#138](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/pull/138))

## [0.18.2](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/compare/tonic-tracing-opentelemetry-v0.18.1...tonic-tracing-opentelemetry-v0.18.2) - 2024-04-24

### <!-- 2 -->Added
- ✨ allow to create span for opentelemetry at level `info` with feature flag `tracing_level_info`
