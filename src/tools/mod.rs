use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
#[cfg(feature = "tracer")]
use opentelemetry::sdk::trace::Tracer;
#[cfg(feature = "tracer")]
use opentelemetry::sdk::Resource;
use opentelemetry::trace::TraceError;
#[cfg(feature = "tracer")]
use opentelemetry_semantic_conventions as semcov;

#[cfg(feature = "jaeger")]
pub mod jaeger;
#[cfg(feature = "otlp")]
pub mod otlp;
#[cfg(feature = "tracer")]
pub mod stdio;

#[cfg(feature = "tracer")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorKind {
    #[cfg(feature = "otlp")]
    Otlp,
    #[cfg(feature = "jaeger")]
    Jaeger,
    Stdout,
    Stderr,
    NoWrite,
}

#[cfg(feature = "tracer")]
pub fn init_tracer(kind: CollectorKind, resource: Resource) -> Result<Tracer, TraceError> {
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
pub fn make_resource<S1, S2>(service_name: S1, service_version: S2) -> Resource
where
    S1: Into<String>,
    S2: Into<String>,
{
    Resource::new(vec![
        semcov::resource::SERVICE_NAME.string(service_name.into()),
        semcov::resource::SERVICE_VERSION.string(service_version.into()),
    ])
}

/// Configure the global propagator based on content of the env variable [OTEL_PROPAGATORS](https://opentelemetry.io/docs/concepts/sdk-configuration/general-sdk-configuration/#otel_propagators)
/// Specifies Propagators to be used in a comma-separated list.
/// Default value: `"tracecontext,baggage"`
/// Example: `export OTEL_PROPAGATORS="b3"`
/// Accepted values for `OTEL_PROPAGATORS` are:
///
/// - "tracecontext": W3C Trace Context
/// - "baggage": W3C Baggage
/// - "b3": B3 Single (require feature "zipkin")
/// - "b3multi": B3 Multi (require feature "zipkin")
/// - "jaeger": Jaeger (require feature "jaeger")
/// - "xray": AWS X-Ray (require feature "xray")
/// - "ottrace": OT Trace (third party) (not supported)
/// - "none": No automatically configured propagator.
pub fn init_propagator() -> Result<(), TraceError> {
    let value_from_env = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .unwrap_or_else(|_| "tracecontext,baggage".to_string());
    let propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>> = value_from_env
        .split(',')
        .map(|s| propagator_from_string(s.trim().to_lowercase().as_str()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    if !propagators.is_empty() {
        let composite_propagator = TextMapCompositePropagator::new(propagators);
        opentelemetry::global::set_text_map_propagator(composite_propagator);
    }
    Ok(())
}

#[allow(clippy::box_default)]
fn propagator_from_string(
    v: &str,
) -> Result<Option<Box<dyn TextMapPropagator + Send + Sync>>, TraceError> {
    match v {
        "tracecontext" => Ok(Some(Box::new(TraceContextPropagator::new()))),
        "baggage" => Ok(Some(Box::new(BaggagePropagator::new()))),
        #[cfg(feature = "zipkin")]
        "b3" => Ok(Some(Box::new(
            opentelemetry_zipkin::Propagator::with_encoding(
                opentelemetry_zipkin::B3Encoding::SingleHeader,
            ),
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'b3', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "zipkin")]
        "b3multi" => Ok(Some(Box::new(
            opentelemetry_zipkin::Propagator::with_encoding(
                opentelemetry_zipkin::B3Encoding::MultipleHeader,
            ),
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3multi" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'b3multi', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "jaeger")]
        "jaeger" => Ok(Some(Box::new(
            opentelemetry_jaeger::Propagator::default()
        ))),
        #[cfg(not(feature = "jaeger"))]
        "jaeger" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'jaeger', try to enable compile feature 'jaeger'"
        )),
        #[cfg(feature = "xray")]
        "xray" => Ok(Some(Box::new(
            opentelemetry_aws::trace::XrayPropagator::default(),
        ))),
        #[cfg(not(feature = "xray"))]
        "xray" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'xray', try to enable compile feature 'xray'"
        )),
        "none" => Ok(None),
        unknown => Err(TraceError::from(format!(
            "unsupported propagators form env OTEL_PROPAGATORS: '{unknown}'"
        ))),
    }
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
