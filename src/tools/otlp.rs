use opentelemetry::sdk::Resource;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, trace::TraceError,
};

pub fn identity(v: opentelemetry_otlp::OtlpTracePipeline) -> opentelemetry_otlp::OtlpTracePipeline {
    v
}

pub fn init_tracer<F>(resource: Resource, transform: F) -> Result<sdktrace::Tracer, TraceError>
where
    F: FnOnce(opentelemetry_otlp::OtlpTracePipeline) -> opentelemetry_otlp::OtlpTracePipeline,
{
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
