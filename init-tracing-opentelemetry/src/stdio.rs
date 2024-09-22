use opentelemetry::trace::{TraceError, TracerProvider as _};
use opentelemetry::InstrumentationLibrary;
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::fmt::Debug;
use std::io::Write;

#[must_use]
pub fn identity<W: Write>(
    v: opentelemetry_sdk::trace::Builder,
) -> opentelemetry_sdk::trace::Builder {
    v
}

pub fn init_tracer<F, W>(resource: Resource, transform: F) -> Result<sdktrace::Tracer, TraceError>
where
    F: FnOnce(opentelemetry_sdk::trace::Builder) -> opentelemetry_sdk::trace::Builder,
    W: Write + Debug + Send + Sync + 'static,
{
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let processor =
        BatchSpanProcessor::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();
    let mut provider_builder: opentelemetry_sdk::trace::Builder = TracerProvider::builder()
        .with_span_processor(processor)
        .with_config(
            sdktrace::Config::default()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        );
    provider_builder = transform(provider_builder);
    // tracer used in libraries/crates that optionally includes version and schema url
    let library = std::sync::Arc::new(
        InstrumentationLibrary::builder(env!("CARGO_PKG_NAME"))
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_schema_url("https://opentelemetry.io/schema/1.0.0")
            .build(),
    );
    Ok(provider_builder.build().library_tracer(library))
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
