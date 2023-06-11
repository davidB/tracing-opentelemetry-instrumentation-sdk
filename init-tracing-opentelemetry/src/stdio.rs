use opentelemetry::sdk::export::trace::stdout::PipelineBuilder;
use opentelemetry::sdk::Resource;
use opentelemetry::{
    global, sdk::propagation::TraceContextPropagator, sdk::trace as sdktrace, trace::TraceError,
};
use std::fmt::Debug;
use std::io::Write;

pub fn identity<W: Write>(v: PipelineBuilder<W>) -> PipelineBuilder<W> {
    v
}

pub fn init_tracer<F, W>(
    resource: Resource,
    transform: F,
    w: W,
) -> Result<sdktrace::Tracer, TraceError>
where
    F: FnOnce(PipelineBuilder<W>) -> PipelineBuilder<W>,
    W: Write + Debug + Send + 'static,
{
    global::set_text_map_propagator(TraceContextPropagator::new());

    let mut pipeline = PipelineBuilder::default().with_writer(w).with_trace_config(
        sdktrace::config()
            .with_resource(resource)
            .with_sampler(sdktrace::Sampler::AlwaysOn),
    );
    pipeline = transform(pipeline);
    Ok(pipeline.install_simple())
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
