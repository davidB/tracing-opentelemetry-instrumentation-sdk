use opentelemetry::sdk::trace::BatchSpanProcessor;
use opentelemetry::sdk::Resource;
use opentelemetry::{sdk::trace as sdktrace, trace::TraceError};
use opentelemetry::{sdk::trace::TracerProvider, trace::TracerProvider as _};
use std::fmt::Debug;
use std::io::Write;

#[must_use]
pub fn identity<W: Write>(
    v: opentelemetry::sdk::trace::Builder,
) -> opentelemetry::sdk::trace::Builder {
    v
}

pub fn init_tracer<F, W>(
    resource: Resource,
    transform: F,
    w: W,
) -> Result<sdktrace::Tracer, TraceError>
where
    F: FnOnce(opentelemetry::sdk::trace::Builder) -> opentelemetry::sdk::trace::Builder,
    W: Write + Debug + Send + Sync + 'static,
{
    let exporter = opentelemetry_stdout::SpanExporter::builder()
        .with_writer(w)
        .build();
    let processor =
        BatchSpanProcessor::builder(exporter, opentelemetry::sdk::runtime::Tokio).build();
    let mut provider_builder: opentelemetry::sdk::trace::Builder = TracerProvider::builder()
        .with_span_processor(processor)
        .with_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn),
        );
    provider_builder = transform(provider_builder);
    Ok(provider_builder.build().tracer(""))
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
