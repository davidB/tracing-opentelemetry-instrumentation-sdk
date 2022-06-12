#[cfg(any(feature = "jaeger", feature = "otlp"))]
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, sdk::Resource,
    trace::TraceError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorKind {
    #[cfg(feature = "otlp")]
    Otlp,
    #[cfg(feature = "jaeger")]
    Jaeger,
    // Stdout,
}

#[cfg(any(feature = "jaeger", feature = "otlp"))]
pub fn init_tracer(kind: CollectorKind) -> Result<sdktrace::Tracer, TraceError> {
    // use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_semantic_conventions as semcov;
    let resource = Resource::new(vec![
        semcov::resource::SERVICE_NAME.string(env!("CARGO_PKG_NAME")),
        semcov::resource::SERVICE_VERSION.string(env!("CARGO_PKG_VERSION")),
    ]);

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

#[cfg(feature = "otlp")]
pub fn init_tracer_otlp(resource: Resource) -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // resource = resource.merge(&read_dt_metadata());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        //endpoint (default = 0.0.0.0:4317 for grpc protocol, 0.0.0.0:4318 http protocol):
        .with_exporter(
            opentelemetry_otlp::new_exporter().tonic(), //.http().with_endpoint(collector_url),
        )
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

    opentelemetry_jaeger::new_pipeline()
        .with_service_name(env!("CARGO_PKG_NAME"))
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
