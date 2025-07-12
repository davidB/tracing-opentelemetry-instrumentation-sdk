use crate::otlp::infer_protocol;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_otlp::{ExporterBuildError, MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, SdkMeterProvider, Temporality};
use opentelemetry_sdk::Resource;
use std::time::Duration;
#[cfg(feature = "tls")]
use {opentelemetry_otlp::WithTonicConfig, tonic::transport::ClientTlsConfig};

#[must_use = "Recommend holding with 'let _guard = ' pattern to ensure final metrics are sent to the server"]
pub struct MetricsGuard {
    meter_provider: SdkMeterProvider,
}

impl MetricsGuard {
    /// the wrapped MeterProvider
    #[must_use]
    pub fn meter_provider(&self) -> &impl MeterProvider {
        &self.meter_provider
    }
}

impl Drop for MetricsGuard {
    fn drop(&mut self) {
        #[allow(unused_must_use)]
        let _ = self.meter_provider.force_flush();
        let _ = self.meter_provider.shutdown();
    }
}

#[must_use]
pub fn identity(v: MeterProviderBuilder) -> MeterProviderBuilder {
    v
}

pub fn init_meterprovider<F>(
    resource: Resource,
    transform: F,
    temporality: Option<Temporality>,
    timeout: Option<Duration>,
) -> Result<SdkMeterProvider, ExporterBuildError>
where
    F: FnOnce(MeterProviderBuilder) -> MeterProviderBuilder,
{
    let (maybe_protocol, maybe_endpoint) = read_protocol_and_endpoint_from_env();
    let protocol = infer_protocol(maybe_protocol.as_deref(), maybe_endpoint.as_deref());
    let timeout = timeout.unwrap_or(Duration::from_secs(10));
    let temporality = temporality.unwrap_or(Temporality::Delta);

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
            tracing::warn!("unknown '{x}' env var set or infered for OTEL_EXPORTER_OTLP_METRICS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no metric exporter will be created");
            None
        }
        None => {
            tracing::warn!("no env var set or infered for OTEL_EXPORTER_OTLP_METRICS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no metric exporter will be created");
            None
        }
    };
    let mut meter_provider = SdkMeterProvider::builder().with_resource(resource);
    if let Some(exporter) = exporter {
        meter_provider = meter_provider.with_periodic_exporter(exporter);
    }
    meter_provider = transform(meter_provider);
    Ok(meter_provider.build())
}

fn read_protocol_and_endpoint_from_env() -> (Option<String>, Option<String>) {
    let maybe_protocol = std::env::var("OTEL_EXPORTER_OTLP_METRICS_PROTOCOL")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL"))
        .ok();
    let maybe_endpoint = std::env::var("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT")
        .or_else(|_| {
            std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").map(|endpoint| match &maybe_protocol {
                Some(protocol) if protocol.contains("http") => {
                    format!("{endpoint}/v1/metrics")
                }
                _ => endpoint,
            })
        })
        .ok();
    (maybe_protocol, maybe_endpoint)
}
