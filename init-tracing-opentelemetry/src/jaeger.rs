use opentelemetry::sdk::Resource;
use opentelemetry::{sdk::trace as sdktrace, trace::TraceError};
use opentelemetry_jaeger::config::agent::AgentPipeline;
use opentelemetry_semantic_conventions as semcov;

#[must_use]
pub fn identity(v: AgentPipeline) -> AgentPipeline {
    v
}

/// Setup a jaeger agent pipeline with the trace-context propagator and the service name.
/// The jaeger pipeline builder can be configured dynamically via environment variables.
/// All variables are optional, a full list of accepted options can be found in the
/// [jaeger variables spec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/sdk-environment-variables.md#jaeger-exporter).
pub fn init_tracer<F>(resource: Resource, transform: F) -> Result<sdktrace::Tracer, TraceError>
where
    F: FnOnce(AgentPipeline) -> AgentPipeline,
{
    let mut pipeline = opentelemetry_jaeger::new_agent_pipeline();
    if let Some(name) = resource.get(semcov::resource::SERVICE_NAME) {
        pipeline = pipeline.with_service_name(name.to_string());
    }
    pipeline = pipeline.with_trace_config(
        sdktrace::config()
            .with_resource(resource)
            .with_sampler(sdktrace::Sampler::AlwaysOn),
    );
    pipeline = transform(pipeline);
    pipeline.install_batch(opentelemetry::runtime::Tokio)
}
