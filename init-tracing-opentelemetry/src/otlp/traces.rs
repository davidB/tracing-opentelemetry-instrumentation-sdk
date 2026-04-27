use super::infer_protocol_from_env;
use opentelemetry_otlp::{ExporterBuildError, SpanExporter};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider, trace::TracerProviderBuilder};
#[cfg(feature = "tls")]
use {opentelemetry_otlp::WithTonicConfig, tonic::transport::ClientTlsConfig};

#[must_use]
pub fn identity(v: TracerProviderBuilder) -> TracerProviderBuilder {
    v
}

// see https://opentelemetry.io/docs/reference/specification/protocol/exporter/
pub fn init_tracerprovider<F>(
    resource: Resource,
    transform: F,
) -> Result<SdkTracerProvider, ExporterBuildError>
where
    F: FnOnce(TracerProviderBuilder) -> TracerProviderBuilder,
{
    super::debug_env();
    let protocol = infer_protocol_from_env(
        "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL",
        "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
        "v1/traces",
    );

    // builders used the environment variables to determine the endpoint (but not to setup the protocol)
    let exporter: Option<SpanExporter> = match protocol.as_deref() {
        Some("http/protobuf") => Some(SpanExporter::builder().with_http().build()?),
        #[cfg(feature = "tls")]
        Some("grpc/tls") => Some(
            SpanExporter::builder()
                .with_tonic()
                .with_tls_config(ClientTlsConfig::new().with_enabled_roots())
                .build()?,
        ),
        Some("grpc") => Some(SpanExporter::builder().with_tonic().build()?),
        Some(x) => {
            tracing::warn!(
                "unknown '{x}' env var set or infered for OTEL_EXPORTER_OTLP_TRACES_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no span exporter will be created"
            );
            None
        }
        None => {
            tracing::warn!(
                "no env var set or infered for OTEL_EXPORTER_OTLP_TRACES_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no span exporter will be created"
            );
            None
        }
    };
    let mut trace_provider = SdkTracerProvider::builder().with_resource(resource);
    if let Some(exporter) = exporter {
        trace_provider = trace_provider.with_batch_exporter(exporter);
    }

    trace_provider = transform(trace_provider);
    Ok(trace_provider.build())
}
