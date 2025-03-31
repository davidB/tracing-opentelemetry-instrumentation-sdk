use opentelemetry_otlp::{ExporterBuildError, SpanExporter};
use opentelemetry_sdk::{trace::SdkTracerProvider, trace::TracerProviderBuilder, Resource};
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
    debug_env();
    let (maybe_protocol, maybe_endpoint) = read_protocol_and_endpoint_from_env();
    let protocol = infer_protocol(maybe_protocol.as_deref(), maybe_endpoint.as_deref());

    let exporter: Option<SpanExporter> = match protocol.as_deref() {
        Some("http/protobuf") => Some(SpanExporter::builder().with_http().build()?),
        #[cfg(feature = "tls")]
        Some("grpc/tls") => Some(
            SpanExporter::builder()
                .with_tonic()
                .with_tls_config(ClientTlsConfig::new().with_native_roots())
                .build()?,
        ),
        Some("grpc") => Some(SpanExporter::builder().with_tonic().build()?),
        Some(x) => {
            tracing::warn!("unknown '{x}' env var set or infered for OTEL_EXPORTER_OTLP_TRACES_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no span exporter will be created");
            None
        }
        None => {
            tracing::warn!("no env var set or infered for OTEL_EXPORTER_OTLP_TRACES_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no span exporter will be created");
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

pub fn debug_env() {
    std::env::vars()
        .filter(|(k, _)| k.starts_with("OTEL_"))
        .for_each(|(k, v)| tracing::debug!(target: "otel::setup::env", key = %k, value = %v));
}

fn read_protocol_and_endpoint_from_env() -> (Option<String>, Option<String>) {
    let maybe_protocol = std::env::var("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL"))
        .ok();
    let maybe_endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .or_else(|_| {
            std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").map(|endpoint| match &maybe_protocol {
                Some(protocol) if protocol.contains("http") => {
                    format!("{endpoint}/v1/traces")
                }
                _ => endpoint,
            })
        })
        .ok();
    (maybe_protocol, maybe_endpoint)
}

#[allow(unused_mut)]
fn infer_protocol(maybe_protocol: Option<&str>, maybe_endpoint: Option<&str>) -> Option<String> {
    let mut maybe_protocol = match (maybe_protocol, maybe_endpoint) {
        (Some(protocol), _) => Some(protocol.to_string()),
        (None, Some(endpoint)) => {
            if endpoint.contains(":4317") {
                Some("grpc".to_string())
            } else {
                Some("http/protobuf".to_string())
            }
        }
        _ => None,
    };
    #[cfg(feature = "tls")]
    if maybe_protocol.as_deref() == Some("grpc")
        && maybe_endpoint.is_some_and(|e| e.starts_with("https"))
    {
        maybe_protocol = Some("grpc/tls".to_string());
    }

    maybe_protocol
}

#[cfg(test)]
mod tests {
    use assert2::assert;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(None, None, None)] //Devskim: ignore DS137138
    #[case(Some("http/protobuf"), None, Some("http/protobuf"))] //Devskim: ignore DS137138
    #[case(Some("grpc"), None, Some("grpc"))] //Devskim: ignore DS137138
    #[case(None, Some("http://localhost:4317"), Some("grpc"))] //Devskim: ignore DS137138
    #[cfg_attr(
        feature = "tls",
        case(None, Some("https://localhost:4317"), Some("grpc/tls"))
    )]
    #[cfg_attr(
        feature = "tls",
        case(Some("grpc/tls"), Some("https://localhost:4317"), Some("grpc/tls"))
    )]
    #[case(
        Some("http/protobuf"),
        Some("http://localhost:4318/v1/traces"), //Devskim: ignore DS137138
        Some("http/protobuf"),
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4318/v1/traces"),
        Some("http/protobuf")
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4317"),
        Some("http/protobuf")
    )]
    fn test_infer_protocol(
        #[case] traces_protocol: Option<&str>,
        #[case] traces_endpoint: Option<&str>,
        #[case] expected_protocol: Option<&str>,
    ) {
        assert!(infer_protocol(traces_protocol, traces_endpoint).as_deref() == expected_protocol);
    }
}
