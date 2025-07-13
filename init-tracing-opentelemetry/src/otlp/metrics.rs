use super::infer_protocol;
use crate::resource::DetectResource;
use crate::Error;
use opentelemetry::global;
use opentelemetry_otlp::{ExporterBuildError, MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::{
    MeterProviderBuilder, PeriodicReader, SdkMeterProvider, Temporality,
};
use opentelemetry_sdk::Resource;
use std::env;
use std::time::Duration;
use tracing::Subscriber;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::registry::LookupSpan;
#[cfg(feature = "tls")]
use {opentelemetry_otlp::WithTonicConfig, tonic::transport::ClientTlsConfig};

pub fn build_metrics_layer<S>() -> Result<(MetricsLayer<S>, SdkMeterProvider), Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let otel_rsrc = DetectResource::default().build();
    let meter_provider = init_meterprovider(otel_rsrc, identity)?;
    global::set_meter_provider(meter_provider.clone());
    let layer = tracing_opentelemetry::MetricsLayer::new(meter_provider.clone());
    Ok((layer, meter_provider))
}

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
    let (maybe_protocol, maybe_endpoint) = read_protocol_and_endpoint_from_env();
    let protocol = infer_protocol(maybe_protocol.as_deref(), maybe_endpoint.as_deref());
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
        let reader = PeriodicReader::builder(exporter)
            .with_interval(export_interval)
            .build();
        meter_provider = meter_provider.with_reader(reader);
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
