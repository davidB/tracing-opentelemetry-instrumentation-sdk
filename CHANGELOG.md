# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Bug Fixes

- 🐛 attribute `http.response.status_code` should of type `int`


### Dependencies

- ⬆️ upgrade to opentelemetry 0.21 (and related dependencies)


### Features

- ✨ add attribute `rpc.grpc.status_code`


### Miscellaneous Tasks

- 👷 Bump stefanzweifel/git-auto-commit-action from 4 to 5

Bumps [stefanzweifel/git-auto-commit-action](https://github.com/stefanzweifel/git-auto-commit-action) from 4 to 5.
- [Release notes](https://github.com/stefanzweifel/git-auto-commit-action/releases)
- [Changelog](https://github.com/stefanzweifel/git-auto-commit-action/blob/master/CHANGELOG.md)
- [Commits](https://github.com/stefanzweifel/git-auto-commit-action/compare/v4...v5)

---
updated-dependencies:
- dependency-name: stefanzweifel/git-auto-commit-action
  dependency-type: direct:production
  update-type: version-update:semver-major
...

Signed-off-by: dependabot[bot] <support@github.com>

- 👷 update megalinter workflow

- 👷 setup git-cliff to generate changelog


## [0.14.0] - 2023-09-04

### Dependencies

- ⬆️ bump tracing-opentelemetry from 0.20 to 0.21

- ⬆️ bump tonic from 0.9 to 0.10 in tonic-tracing-opentelemetrty


### Documentation

- ✏️ fix typo in homepage of init-tracing-opentelemetry


### Features

- ✨ enable simple basic grpc tls endpoint (#85)

* feat: enable simple basic grpc tls endpoint

* feat: make "tls" a feature

* fix: per PR feedback

### Miscellaneous Tasks

- 👷 move version of some dependencies into workspace

- 🚧 search for memory leak, add a examples/load

WIP #87


## [0.13.0] - 2023-08-06

### Dependencies

- ⬆️ upgrade to opentelemetry 0.20 (and related dependencies)


### Features

- Feat: add span.type=web on spans

- Feat: add span_type enum and documentation


### Testing

- ✅ update test result


## [0.12.0] - 2023-07-02

### Documentation

- 📝 update README


### Refactor

- 🔥 merge use of megalinter into justfile


## [0.12.0-alpha.3] - 2023-06-28

### Refactor

- 🎨 format justfile

- ♻️ few clean-up


## [0.12.0-alpha.2] - 2023-06-28

### Bug Fixes

- 🐛 grpc client set the span context during async children processing

- 🐛 grpc server set the span context during async children processing


### Features

- 💥 use `otel::tracing` as target for trace instead on the name of the crate

- ✨  introduce new crate `tracing-opentelemetry-instrumentation-sdk`

- ✨ grpc server layer can use a filter function to not create trace for some endpoint

- 💥 rewrite axum-tracing-opentelemetry

- switch from TraceLayer from tower-http to a dedicated Layer
- deprecate the factories function
- update opentelemetry field to follow v1.22 of opentelemetry semantic
- no longer injection of `trace_id` into span'events (previous hack introduced some invalid state or value)


### Miscellaneous Tasks

- 🚧 temporary solution for grpc

- on client side the span doesn't live over the async call
- to rework to use tower_http::Trace for client and server (maybe do not encapsulate / hide TraceLayer, but provide the code + helper for client and server)

- 👷 replace direnv by rtx to setup dev environment

- 🚧 fix constraint for tonic client middleware

- 👷 add "just" commands to help run locally

- 👷 add some command to dev locally

- 🚧 add & fix missing function on sdk

- 👷 enable local sccache via rtx


### Performance

- ⚡️ tag as `inline` some helpers function


### Refactor

- 🔥 remove deprecated code

- 🎨 format comments


### Testing

- ✅ fix compilation of test


## [0.12.0-alpha.1] - 2023-06-14

### Documentation

- 📝 add notes about how to release the workspace


### Features

- ✨ add basic filtering for axum-tracing-opentelemetry


## [0.12.0-alpha.0] - 2023-06-14

### Dependencies

- ➖ remove more unused dependencies


### Features

- ✨ extract `fake-opentelemetry-collector`

- ✨ start the tonic-tracing-opentelemetry

- ✨ start the testing-tracing-opentelemetry


### Miscellaneous Tasks

- 🚧 extract initialization code into new crate `init-tracing-opentelemetry`

- 🚧 clean axum-tracing-opentelemetry to used extracted crate

- 🚧 fix test after refactor

- 🚧 improve tonic-tracing-opentelemetry (add with_filter)


### Refactor

- 🔥 remove grpc from axum-tracing-opentelemetry + move snapshot to testing


### Testing

- ✅  update examples


## [0.11.0] - 2023-06-11

### Bug Fixes

- 🐛 fix features dependencies

- Fix: fallback to req uri path for nested route (we can not get matched router in nested router handler)

- 🐛 generate root opentelemetry span with valid spanId

root  Trace span name shown as `<root span not yet received>`  #52


### Dependencies

- ⬆️ upgrade opentelemetry to 0.19 (and related dependencies)

- ⬆️ upgrade opentelemetry to 0.19 (and related dependencies)  (2)

- ⬆️ upgrade opentelemetry to 0.19 (and related dependencies)  (3)


### Features

- ✨ add a mock_collector server to to collect trace


### Miscellaneous Tasks

- 🚧 add a mock for trace collector

- 👷 run test with 1 thread else failure with mock_collector issue `cargo test` but not with `cargo nextest`

- 🚧 use same trace_id into tracing and otel

- 🚧 store to merge before reorg of the crate

- 🚧 extract feature into into there own crate (start workspace)


### Refactor

- 🎨 format  /sort `Cargo.toml`

- ♻️ use snapshot test (insta) for trace_extractor

- 🔥 remove github config about branch_protection_rules


### Testing

- ✅ add test about propagation in trace_extractor middleware

- ✅ prepare tests

- ✅  update test about nesteed route with axum >= 0.6.15

FIXE #54

- ✅ span should not be attached as link and undefined "route" should be empty


## [0.10.0] - 2023-02-26

### Documentation

- 📝 add sample to overwrite `otel.name`

- 📝 update changelog


### Features

- 💥 default configuration for otlp Sampler is now longer hardcoded to `always_on`, but read environment variables `OTEL_TRACES_SAMPLER`, `OTEL_TRACES_SAMPLER_ARG`

- ✨ add a axum layer for gRPC (#36)

Almost the same in every way, except that it uses Tower's
`TraceLayer::new_for_grpc`, and formats the initially generated span a
bit differently to match gRPC conventions.
- ✨ log under target `otel::setup` detected configuration by otel setup tools

- ✨ provide opinionated `tracing_subscriber_ext`


### Miscellaneous Tasks

- 👷fix 2 typos


### Refactor

- ♻️ rename `DetectResource.with_println` into `DetectResource.with_log_of_resources`


## [0.9.0] - 2023-02-05

### Bug Fixes

- 🐛 fix mega-linter.yml


### Documentation

- 📝 add instruction to launch jaeger for local dev

- 📝 improve sample


### Features

- ✨ add `DetectResource` builder to help detection for [Resource Semantic Conventions | OpenTelemetry](https://opentelemetry.io/docs/reference/specification/resource/semantic_conventions/#semantic-attributes-with-sdk-provided-default-value)


### Miscellaneous Tasks

- 👷 check examples'crates

- 👷  fix clippy & protoc integration into CI (#43)

* ⚗️ try to add protoc before launch mega-linter

* 👷 move clippy from mega-linter to ci

* 🚨 fix clippy suggestion

* 👷 disable megalinter on  pull_request (on on push)

* [MegaLinter] Apply linters fixes

### Testing

- ✅  use stdio tracer for test

- ✅ all test without features enabled to pass

- ✅ test better detect features misconfiguration

- ✅ update test related to doc


## [0.8.2] - 2023-01-30

### Bug Fixes

- 🐛 restore missing line in changelog

- 🐛 use correct env variable (OTEL_PROPAGATORS) when setting up propagators


### Testing

- ✅


## [0.8.1] - 2023-01-29

### Documentation

- 📝 update documentation & samples about configuration


### Features

- ✨ add `init_propagator` based on OTEL_PROPAGATORS


## [0.7.1] - 2023-01-01

### Documentation

- 📝 use more OTEL env variable into sample


### Refactor

- ♻️ allow make_resource to support different type for the 2 args


## [0.7.0] - 2022-12-28

### Documentation

- 📝 add compatibility matrix

- 📝 update changelog


### Features

- ✨ add a layer`response_with_trace_layer` to have `traceparent` injected into response


### Refactor

- ♻️ convert module middleware from file to directory


## [0.5.2] - 2022-11-06

### Bug Fixes

- Fix: do not populate http.route when not supported by the HTTP server framework

According to https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/http.md
```
[1]: 'http.route' MUST NOT be populated when this is not supported by the HTTP server framework as the route attribute should have low-cardinality and the URI path can NOT substitute it.
```

<!-- generated by git-cliff -->
