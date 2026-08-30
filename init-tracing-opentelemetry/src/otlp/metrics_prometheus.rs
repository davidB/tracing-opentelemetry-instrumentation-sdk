use crate::Error;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;

/// Build an [`SdkMeterProvider`] backed by a pull-based Prometheus exporter.
///
/// Only reachable through [`crate::TracingConfig::with_metrics_prometheus`] —
/// there is no public `init_*` entrypoint for this backend, unlike the OTLP
/// one, so `TracingConfig` stays the single place to configure metrics.
pub(crate) fn init_meterprovider_prometheus(
    resource: Resource,
) -> Result<(SdkMeterProvider, prometheus::Registry), Error> {
    let registry = prometheus::Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()?;
    let provider = SdkMeterProvider::builder()
        .with_reader(exporter)
        .with_resource(resource)
        .build();
    Ok((provider, registry))
}
