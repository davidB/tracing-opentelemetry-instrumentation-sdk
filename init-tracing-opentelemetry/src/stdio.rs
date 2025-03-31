use crate::Error;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::trace::TracerProviderBuilder;
use opentelemetry_sdk::Resource;
use std::fmt::Debug;
use std::io::Write;

#[must_use]
pub fn identity<W: Write>(v: TracerProviderBuilder) -> TracerProviderBuilder {
    v
}

pub fn init_tracer<F, W>(resource: Resource, transform: F) -> Result<sdktrace::Tracer, Error>
where
    F: FnOnce(TracerProviderBuilder) -> TracerProviderBuilder,
    W: Write + Debug + Send + Sync + 'static,
{
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let processor = BatchSpanProcessor::builder(exporter).build();
    let mut provider_builder = SdkTracerProvider::builder()
        .with_span_processor(processor)
        .with_resource(resource)
        .with_sampler(sdktrace::Sampler::AlwaysOn);
    provider_builder = transform(provider_builder);
    // tracer used in libraries/crates that optionally includes version and schema url
    let scope = InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schema/1.0.0")
        .build();
    Ok(provider_builder.build().tracer_with_scope(scope))
}

#[derive(Debug, Default)]
pub struct WriteNoWhere;

impl Write for WriteNoWhere {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
