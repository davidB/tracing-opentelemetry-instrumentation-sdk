#[cfg(any(feature = "jaeger", feature = "otlp", feature = "tracer"))]
use opentelemetry::sdk::Resource;
#[cfg(any(feature = "jaeger", feature = "otlp", feature = "tracer"))]
use opentelemetry::{sdk::trace as sdktrace, trace::TraceError};
#[cfg(feature = "tracer")]
use opentelemetry_semantic_conventions as semcov;

#[cfg(feature = "jaeger")]
pub mod jaeger;
#[cfg(feature = "otlp")]
pub mod otlp;
#[cfg(feature = "tracer")]
pub mod stdio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorKind {
    #[cfg(feature = "otlp")]
    Otlp,
    #[cfg(feature = "jaeger")]
    Jaeger,
    #[cfg(feature = "tracer")]
    Stdout,
    #[cfg(feature = "tracer")]
    Stderr,
    #[cfg(feature = "tracer")]
    NoWrite,
}

#[cfg(feature = "tracer")]
pub fn init_tracer(
    kind: CollectorKind,
    resource: Resource,
) -> Result<sdktrace::Tracer, TraceError> {
    match kind {
        CollectorKind::Stdout => stdio::init_tracer(resource, stdio::identity, std::io::stdout()),
        CollectorKind::Stderr => stdio::init_tracer(resource, stdio::identity, std::io::stderr()),
        CollectorKind::NoWrite => {
            stdio::init_tracer(resource, stdio::identity, stdio::WriteNoWhere::default())
        }
        #[cfg(feature = "otlp")]
        CollectorKind::Otlp => {
            // if let Some(url) = std::env::var_os("OTEL_COLLECTOR_URL")
            // "http://localhost:14499/otlp/v1/traces"
            // let collector_url = url.to_str().ok_or(TraceError::Other(
            //     anyhow!("failed to parse OTEL_COLLECTOR_URL").into(),
            // ))?;
            otlp::init_tracer(resource, otlp::identity)
        }
        #[cfg(feature = "jaeger")]
        CollectorKind::Jaeger => {
            // Or "OTEL_EXPORTER_JAEGER_ENDPOINT"
            // or now variable
            jaeger::init_tracer(resource, jaeger::identity)
        }
    }
}

/// call with service name and version
///
/// ```rust
/// use axum_tracing_opentelemetry::make_resource;
/// # fn main() {
/// let r = make_resource(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
/// # }
///
/// ```
#[cfg(feature = "tracer")]
pub fn make_resource<S>(service_name: S, service_version: S) -> Resource
where
    S: Into<String>,
{
    Resource::new(vec![
        semcov::resource::SERVICE_NAME.string(service_name.into()),
        semcov::resource::SERVICE_VERSION.string(service_version.into()),
    ])
}

/// Search the current opentelemetry trace id into the Context from the current tracing'span.
/// This function can be used to report the trace id into the error message send back to user.
///
/// ```rust
/// let trace_id = axum_tracing_opentelemetry::find_current_trace_id();
/// // json!({ "error" :  "xxxxxx", "trace_id": trace_id})
///
/// ```
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
