use super::infer_protocol_from_env;
use opentelemetry_otlp::{ExporterBuildError, MetricExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{
    MeterProviderBuilder, PeriodicReader, SdkMeterProvider, Temporality,
};
use std::env;
use std::time::Duration;
#[cfg(feature = "tls")]
use {opentelemetry_otlp::WithTonicConfig, tonic::transport::ClientTlsConfig};

#[must_use]
pub fn identity(v: MeterProviderBuilder) -> MeterProviderBuilder {
    v
}

pub fn init_meterprovider<F>(
    resource: Resource,
    transform: F,
) -> Result<SdkMeterProvider, ExporterBuildError>
where
    F: FnOnce(MeterProviderBuilder) -> MeterProviderBuilder,
{
    let protocol = infer_protocol_from_env(
        "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL",
        "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT",
        "v1/metrics",
    );
    let timeout = env::var("OTEL_EXPORTER_OTLP_METRICS_TIMEOUT")
        .ok()
        .and_then(|var| var.parse::<u64>().ok())
        .map_or(Duration::from_secs(10), Duration::from_secs);
    let temporality = env::var("OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE")
        .ok()
        .and_then(|var| match var.to_lowercase().as_str() {
            "delta" => Some(Temporality::Delta),
            "cumulative" => Some(Temporality::Cumulative),
            unknown => {
                tracing::warn!("unknown '{unknown}' env var set for OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY; defaulting to cumulative");
                None
            },
        })
        .unwrap_or_default();
    let export_interval = env::var("OTEL_METRIC_EXPORT_INTERVAL")
        .ok()
        .and_then(|var| var.parse::<u64>().ok())
        .map_or(Duration::from_secs(60), Duration::from_millis);

    // builders used the environment variables to determine the endpoint (but not to setup the protocol)
    let exporter = match protocol.as_deref() {
        Some("http/protobuf") => Some(
            MetricExporter::builder()
                .with_http()
                .with_temporality(temporality)
                .with_timeout(timeout)
                .build()?,
        ),
        #[cfg(feature = "tls")]
        Some("grpc/tls") => Some(
            MetricExporter::builder()
                .with_tonic()
                .with_tls_config(ClientTlsConfig::new().with_enabled_roots())
                .with_temporality(temporality)
                .with_timeout(timeout)
                .build()?,
        ),
        Some("grpc") => Some(
            MetricExporter::builder()
                .with_tonic()
                .with_temporality(temporality)
                .with_timeout(timeout)
                .build()?,
        ),
        Some(x) => {
            tracing::warn!(
                "unknown '{x}' env var set or infered for OTEL_EXPORTER_OTLP_METRICS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no metric exporter will be created"
            );
            None
        }
        None => {
            tracing::warn!(
                "no env var set or infered for OTEL_EXPORTER_OTLP_METRICS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no metric exporter will be created"
            );
            None
        }
    };
    let mut meter_provider = SdkMeterProvider::builder().with_resource(resource);
    if let Some(exporter) = exporter {
        let reader = PeriodicReader::builder(exporter)
            .with_interval(export_interval)
            .build();
        meter_provider = meter_provider.with_reader(reader);
    }
    meter_provider = transform(meter_provider);
    Ok(meter_provider.build())
}
