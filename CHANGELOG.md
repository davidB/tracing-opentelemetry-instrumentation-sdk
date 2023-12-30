<!-- markdownlint-disable MD024-->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.16.0] - 2023-12-30

### Added

- ✨ add support for OTLP headers from environment (#110) ([ccd123b](ccd123b6d7de9c1f10d3d861cb8494db9ed201ee))

### Changed

- 📝 update CHANGELOG ([319b1eb](319b1eb17cc8876d3b7f999a4e1d5b4f534d2816))
- 📝 Update link to changelog, remove homepage, ... ([7f38094](7f380949f73c76a315779db727b75be32211804d))
- ➖ remove dependency to opentelemetry-http ([e049fb0](e049fb0e0c67140b3252bf465aa3c74e6838400d))
- ⬆️ upgrade dependencies for axum-0.7 ([d4ad2d3](d4ad2d31bf8787b8c99332f6b8a7e44e34088886))
- 📝 update example in doc ([b74c686](b74c68604ab359b19d4e43da1a3b0514e1ec2e68))

### Fixed

- 🐛 fix compilation & linter ([24d1eca](24d1eca18a2f85bd2fda98389583684a89d42e7c))

## [0.15.0] - 2023-11-25

### Added

- ✨ add attribute `rpc.grpc.status_code` ([d885954](d8859542f80cf0df365ee18c3fcce654e2e1a843))

### Changed

- ⬆️ upgrade to openteletry 0.21 (and related dependencies) ([21ceb34](21ceb3450973b288743c9fc026cc45072364bb5e))

### Fixed

- 🐛 attribute `http.response.status_code` should of type `int` ([6ff9209](6ff9209175101ed767fd5eee0f5a33f663755dce))

## [0.14.0] - 2023-09-04

### Added

- ✨ enable simple basic grpc tls endpoint (#85) ([ecf4f9d](ecf4f9decce5e14766e6e7c24138bcc3519cd540))

### Changed

- ✏️ fix typo in homepage of init-tracing-opentelemetry ([9cfbaff](9cfbaff8f344e3ba918c7b0fa2587d0d18945172))
- ⬆️ bump tracing-opentelemetry from 0.20 to 0.21 ([6763c41](6763c41ba06e34fe1382e1f797c539e57cbe9cf5))
- ⬆️ bump tonic from 0.9 to 0.10 in tonic-tracing-opentelemetrty ([f33bfe6](f33bfe6f77fd1013497d79f147ce2732d3f4e3ea))

## [0.13.0] - 2023-08-06

### Added

- Feat: add span.type=web on spans ([d76017f](d76017f797b5b9cf2a649824aaea07c81cf84dcf))
- Feat: add span_type enum and documentation ([4871359](487135955342241b2633968c2162415159b9cdab))

### Changed

- ⬆️ upgrade to opentelemetry 0.20 (and related dependencies) ([8b8281e](8b8281ee5e938143379db7d5ef645a830ba87c51))

## [0.12.0] - 2023-07-02

### Changed

- 📝 update README ([400adeb](400adeb7b1105f0c197a29b6a27ec35fe4b1f722))

## [0.12.0-alpha.2] - 2023-06-28

### Added

- 💥 use `otel::tracing` as target for trace instead on the name of the crate ([1fda7c3](1fda7c3d566d4a622710116616d9d28680b7b475))
- ✨  introduce new crate `tracing-opentelemetry-instrumentation-sdk` ([51c45ae](51c45ae5f892e0efbea0ce957d3f3a7524bfe927))
- ✨ grpc server layer can use a filter function to not create trace for some endpoint ([2f3ca50](2f3ca5045ab43fcd5c2f3985f9117c9940d5f3ae))
- 💥 rewrite axum-tracing-opentelemetry ([661b891](661b8917d61b52e8d682863e75bece9ad76e9f9b))

### Changed

- ⚡️ tag as `inline` some helpers function ([753b1a7](753b1a72ece620a46461f7860360ebc347f518bb))

### Fixed

- 🐛 grpc client set the span context during async children processing ([cec0ce5](cec0ce531fbca3caf371ee290593e9cf5e226bf7))
- 🐛 grpc server set the span context during async children processing ([83d88e4](83d88e466049c4613cdd10e2d6668cd3a3d0428e))

## [0.12.0-alpha.1] - 2023-06-14

### Added

- ✨ add basic filtering for axum-tracing-opentelemetry ([bb510a3](bb510a32148182090264d5be9d1c9abe21895083))

### Changed

- 📝 add notes about how to release the workspace ([d1abae1](d1abae15855cfb6fb3d058fbb134f31da82018e3))

## [0.12.0-alpha.0] - 2023-06-14

### Added

- ✨ extract `fake-opentelemetry-collector` ([25becbb](25becbb6633336c189cb2ab02ff94f7530e8ac57))
- ✨ start the tonic-tracing-opentelemetry ([43c179f](43c179f28aa295a81428d2b08ebae83397329943))
- ✨ start the testing-tracing-opentelemetry ([d7ecb0d](d7ecb0dd416fd4d7d78e51abaa8d10a4c0fbb63a))

### Changed

- ➖ remove more unused dependencies ([46793cf](46793cf708952e21ee316dcedaf1873acf175600))

## [0.11.0] - 2023-06-11

### Added

- ✨ add a mock_collector server to to collect trace ([b36f5b1](b36f5b1557963d5678f8a337e0bf45606fb03dcf))

### Changed

- ⬆️ upgrade opentelemetry to 0.19 (and related dependencies) ([36b52a0](36b52a0bad4babfc8ace5fcaab79e897907890d3))
- ⬆️ upgrade opentelemetry to 0.19 (and related dependencies)  (2) ([b7a2a0e](b7a2a0ed9990c35424335e6cf71fa7e28ba1e60b))
- ⬆️ upgrade opentelemetry to 0.19 (and related dependencies)  (3) ([b8719a2](b8719a2912edac8e6556774530fbf99afb82a955))

### Fixed

- 🐛 fix features dependencies ([bdc949d](bdc949d2d0f1eafe0e44ecdbf4607f040150641d))
- Fix: fallback to req uri path for nested route (we can not get matched router in nested router handler) ([36a4302](36a43025ba54721dbb41306086a9135b80350f6c))
- 🐛 generate root opentelemetry span with valid spanId ([c5738a6](c5738a6a4586f9cabd66330335c8353b528498ed))

## [0.10.0] - 2023-02-26

### Added

- 💥 default configuration for otlp Sampler is now longer hardcoded to `always_on`, but read environment variables `OTEL_TRACES_SAMPLER`, `OTEL_TRACES_SAMPLER_ARG` ([c20e7c7](c20e7c77bb2da30737f78a48e4a513d6f3117f24))
- ✨ add a axum layer for gRPC (#36) ([bf7daee](bf7daeeebe1ffd07834388c81349ab7a972abdbe))
- ✨ log under target `otel::setup` detected configuration by otel setup tools ([6c2f5c1](6c2f5c119bea731cb3de770dabcfe726c8edc227))
- ✨ provide opinionated `tracing_subscriber_ext` ([53963eb](53963eb1ee543f3b1c0a0a90a9c00a319694f71b))

### Changed

- 📝 add sample to overwrite `otel.name` ([1dae1aa](1dae1aab7edb2b1cc793ab1e609ea6153e73f2d3))
- 📝 update changelog ([2945358](29453580794da60dacf449676820a8731fd036e9))

## [0.9.0] - 2023-02-05

### Added

- ✨ add `DetectResource` builder to help detection for [Resource Semantic Conventions | OpenTelemetry](https://opentelemetry.io/docs/reference/specification/resource/semantic_conventions/#semantic-attributes-with-sdk-provided-default-value) ([db7552e](db7552efdc5ea842bc17e19604a46ebe77283d0c))

### Changed

- 📝 add instruction to launch jaeger for local dev ([95411e9](95411e9640fc7a1e4eaf7bc5a6d9d07362cfe752))
- 📝 improve sample ([1b91fbf](1b91fbf9172578a81598292666fa9bd854a7f4c5))

### Fixed

- 🐛 fix mega-linter.yml ([6494dd6](6494dd6dab76f09e3364b1fe508ee6edae39356b))

## [0.8.2] - 2023-01-30

### Fixed

- 🐛 restore missing line in changelog ([f46c342](f46c3427fa6f31e8aa4e550315160ce2bcbafb1b))
- 🐛 use correct env variable (OTEL_PROPAGATORS) when setting up propagators ([c2d34eb](c2d34eb54a7672b62aefd3429cc264235c5f952d))

## [0.8.1] - 2023-01-29

### Added

- ✨ add `init_propagator` based on OTEL_PROPAGATORS ([b45b2f3](b45b2f3a39afaf86a589b5cef01e147f11416c3d))

### Changed

- 📝 update documentation & samples about configuration ([75a040d](75a040d08ebd41901a803562b1b8788f9a38e031))

## [0.7.1] - 2023-01-01

### Changed

- 📝 use more OTEL env variable into sample ([048f57c](048f57c668a352739172ffd3af965f263452a4a2))

## [0.7.0] - 2022-12-28

### Added

- ✨ add a layer`response_with_trace_layer` to have `traceparent` injected into response ([368c59d](368c59d0b0a928459b24e21ff26eac337c79a283))

### Changed

- 📝 add compatibility matrix ([9312737](93127375a8393a4b8df9dddbd108f875c1ab9cee))
- 📝 update changelog ([820ae63](820ae63eedcfaa76d5182c9c628c233a007ce8e0))

## [0.5.2] - 2022-11-06

### Fixed

- Fix: do not populate http.route when not supported by the HTTP server framework ([93cedaa](93cedaa7a94904cff127a966db94b70dd697cc6a))

## [0.5.1] - 2022-11-01

### Added

- :green_heart: add protoc into the CI require by `opentelemetry-proto` ([a1777c6](a1777c631c603074f9928787d156a5d4806bc708))

### Removed

- :rotating_light: remove useless code (after validation that experiment is ok) ([b17d9f0](b17d9f0a54a76ea3b185acbcdb97bfd0efffca98))

## [0.3.0] - 2022-08-04

### Added

- :pencil: add a sample about how to retrieve trace_id ([6dd26ff](6dd26ff288f95b719a42c4c1939596532a3f9e4c))

### Removed

- :heavy_minus_sign: remove unused tansitive dependencies ([bca0c14](bca0c1485ac47ee2756f5dc7963863b2f6d39057))

## [0.2.1] - 2022-06-11

### Added

- :sparkles: add code for opentelemetry_tracing_layer ([9403583](94035838f97aa61ad304791f0f3174e042a566f3))
- :sparkles: add tools to init tracer and find trace_id ([acb52a3](acb52a3ed98aeed7733ce3dba7aba894122a8949))
- :pencil: add examples code ([0482b59](0482b59f19af6d0e74082b43c19783e1d08e4c95))
- :pencil: add missing info for release ([a2f7c09](a2f7c0961366bcfd160ee89360d30c8874cfb6fd))

<!-- generated by git-cliff -->
