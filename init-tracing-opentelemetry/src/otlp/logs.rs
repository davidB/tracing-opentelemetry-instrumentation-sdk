use super::infer_protocol_from_env;
use opentelemetry_otlp::{ExporterBuildError, LogExporter};
use opentelemetry_sdk::{Resource, logs::LoggerProviderBuilder, logs::SdkLoggerProvider};
#[cfg(feature = "tls")]
use {opentelemetry_otlp::WithTonicConfig, tonic::transport::ClientTlsConfig};

/// No-op transform for [`init_loggerprovider`]. Pass when no builder customization is needed.
#[must_use]
pub fn identity(v: LoggerProviderBuilder) -> LoggerProviderBuilder {
    v
}

/// Build an [`SdkLoggerProvider`] driven by env vars.
///
/// Protocol is inferred from `OTEL_EXPORTER_OTLP_LOGS_PROTOCOL` (or the endpoint port).
/// Pass [`identity`] as `transform` for no customization.
pub fn init_loggerprovider<F>(
    resource: Resource,
    transform: F,
) -> Result<SdkLoggerProvider, ExporterBuildError>
where
    F: FnOnce(LoggerProviderBuilder) -> LoggerProviderBuilder,
{
    let protocol = infer_protocol_from_env(
        "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL",
        "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT",
        "v1/logs",
    );

    // builders used the environment variables to determine the endpoint (but not to setup the protocol)
    let exporter: Option<LogExporter> = match protocol.as_deref() {
        Some("http/protobuf") => Some(LogExporter::builder().with_http().build()?),
        #[cfg(feature = "tls")]
        Some("grpc/tls") => Some(
            LogExporter::builder()
                .with_tonic()
                .with_tls_config(ClientTlsConfig::new().with_enabled_roots())
                .build()?,
        ),
        Some("grpc") => Some(LogExporter::builder().with_tonic().build()?),
        Some(x) => {
            tracing::warn!(
                "unknown '{x}' env var set or infered for OTEL_EXPORTER_OTLP_LOGS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no log exporter will be created"
            );
            None
        }
        None => {
            tracing::warn!(
                "no env var set or infered for OTEL_EXPORTER_OTLP_LOGS_PROTOCOL or OTEL_EXPORTER_OTLP_PROTOCOL; no log exporter will be created"
            );
            None
        }
    };
    let mut logger_provider = SdkLoggerProvider::builder().with_resource(resource);
    if let Some(exporter) = exporter {
        logger_provider = logger_provider.with_batch_exporter(exporter);
    }

    logger_provider = transform(logger_provider);
    Ok(logger_provider.build())
}
