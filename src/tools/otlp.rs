use opentelemetry::sdk::Resource;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, trace::TraceError,
};
use opentelemetry_otlp::SpanExporterBuilder;

pub fn identity(v: opentelemetry_otlp::OtlpTracePipeline) -> opentelemetry_otlp::OtlpTracePipeline {
    v
}

// see https://opentelemetry.io/docs/reference/specification/protocol/exporter/
pub fn init_tracer<F>(resource: Resource, transform: F) -> Result<sdktrace::Tracer, TraceError>
where
    F: FnOnce(opentelemetry_otlp::OtlpTracePipeline) -> opentelemetry_otlp::OtlpTracePipeline,
{
    use opentelemetry_otlp::WithExportConfig;

    global::set_text_map_propagator(TraceContextPropagator::new());
    let protocol = std::env::var("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL"))
        .unwrap_or_else(|_| "http/protobuf".to_string());
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT"))
        .ok();

    let exporter: SpanExporterBuilder = match protocol.as_str() {
        "http/protobuf" => opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(endpoint.unwrap_or_else(|| "http://localhost:4318".to_string())) //Devskim: ignore DS137138
            .into(),
        _ => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint.unwrap_or_else(|| "http://localhost:4317".to_string())) //Devskim: ignore DS137138
            .into(),
    };

    let mut pipeline = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        );
    pipeline = transform(pipeline);
    pipeline.install_batch(opentelemetry::runtime::Tokio)
}
