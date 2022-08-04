use opentelemetry::sdk::Resource;
#[cfg(any(feature = "jaeger", feature = "otlp"))]
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, trace::TraceError,
};
use opentelemetry_semantic_conventions as semcov;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorKind {
    #[cfg(feature = "otlp")]
    Otlp,
    #[cfg(feature = "jaeger")]
    Jaeger,
    // Stdout,
}

#[cfg(any(feature = "jaeger", feature = "otlp"))]
pub fn init_tracer(
    kind: CollectorKind,
    resource: Resource,
) -> Result<sdktrace::Tracer, TraceError> {
    match kind {
        #[cfg(feature = "otlp")]
        CollectorKind::Otlp => {
            // if let Some(url) = std::env::var_os("OTEL_COLLECTOR_URL")
            // "http://localhost:14499/otlp/v1/traces"
            // let collector_url = url.to_str().ok_or(TraceError::Other(
            //     anyhow!("failed to parse OTEL_COLLECTOR_URL").into(),
            // ))?;
            init_tracer_otlp(resource)
        }
        #[cfg(feature = "jaeger")]
        CollectorKind::Jaeger => {
            // Or "OTEL_EXPORTER_JAEGER_ENDPOINT"
            // or now variable
            init_tracer_jaeger(resource)
        }
    }
}

/// call with service name and version
///
/// ```rust
/// make_resource(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
/// ```
pub fn make_resource<S>(service_name: S, service_version: S) -> Resource
where
    S: Into<String>,
{
    Resource::new(vec![
        semcov::resource::SERVICE_NAME.string(service_name.into()),
        semcov::resource::SERVICE_VERSION.string(service_version.into()),
    ])
}

#[cfg(feature = "otlp")]
pub fn init_tracer_otlp(resource: Resource) -> Result<sdktrace::Tracer, TraceError> {
    use opentelemetry_otlp::WithExportConfig;

    global::set_text_map_propagator(TraceContextPropagator::new());
    // FIXME choice the right/official env variable `OTEL_COLLECTOR_URL` or `OTEL_EXPORTER_OTLP_ENDPOINT`
    // TODO try to autodetect if http or grpc should be used (eg based on env variable, port ???)
    //endpoint (default = 0.0.0.0:4317 for grpc protocol, 0.0.0.0:4318 http protocol):
    //.http().with_endpoint(collector_url),
    let endpoint_grpc = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://0.0.0.0:4317".to_string());
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint_grpc);
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry::runtime::Tokio)
}

#[cfg(feature = "jaeger")]
// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/sdk-environment-variables.md#jaeger-exporter
pub fn init_tracer_jaeger(resource: Resource) -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );

    let mut pipeline = opentelemetry_jaeger::new_pipeline();
    if let Some(name) = resource.get(semcov::resource::SERVICE_NAME) {
        pipeline = pipeline.with_service_name(name.to_string());
    }
    pipeline
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry::runtime::Tokio)
}

/// Search the current opentelemetry trace id into the Context from the current tracing'span.
/// This function can be used to report the trace id into the error message send back to user.
pub fn find_current_trace_id() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    // let context = opentelemetry::Context::current();
    // OpenTelemetry Context is propagation inside code is done via tracing crate
    let context = tracing::Span::current().context();
    let span = context.span();
    let span_context = span.span_context();
    span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string())
}
