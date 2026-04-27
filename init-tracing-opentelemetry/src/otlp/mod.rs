#[cfg(feature = "logs")]
pub mod logs;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod traces;

#[cfg(feature = "logs")]
use opentelemetry::logs::LoggerProvider;
#[cfg(feature = "metrics")]
use opentelemetry::metrics::MeterProvider;
#[cfg(feature = "logs")]
use opentelemetry_sdk::logs::SdkLoggerProvider;
#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::SdkMeterProvider;

use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

#[must_use = "Recommend holding with 'let _guard = ' pattern to ensure final traces/logs/metrics are sent to the server"]
/// On Drop of the `OtelGuard` instance,
/// the wrapped Tracer/Logger/Meter Provider is force to flush and to shutdown (ignoring error).
#[allow(clippy::struct_field_names)]
pub struct OtelGuard {
    #[cfg(feature = "logs")]
    pub(crate) logger_provider: SdkLoggerProvider,
    #[cfg(feature = "metrics")]
    pub(crate) meter_provider: SdkMeterProvider,
    pub(crate) tracer_provider: SdkTracerProvider,
}

impl OtelGuard {
    #[cfg(feature = "logs")]
    #[must_use]
    pub fn logger_provider(&self) -> &impl LoggerProvider {
        &self.logger_provider
    }

    #[must_use]
    pub fn tracer_provider(&self) -> &impl TracerProvider {
        &self.tracer_provider
    }

    #[cfg(feature = "metrics")]
    #[must_use]
    pub fn meter_provider(&self) -> &impl MeterProvider {
        &self.meter_provider
    }
}

impl Drop for OtelGuard {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        let _ = self.tracer_provider.force_flush();
        let _ = self.tracer_provider.shutdown();
        #[cfg(feature = "logs")]
        {
            let _ = self.logger_provider.force_flush();
            let _ = self.logger_provider.shutdown();
        }
        #[cfg(feature = "metrics")]
        {
            let _ = self.meter_provider.force_flush();
            let _ = self.meter_provider.shutdown();
        }
    }
}

#[allow(unused_mut)]
pub(crate) fn infer_protocol_from_env(
    protocol_key: &str,
    endpoint_key: &str,
    endpoint_path: &str,
) -> Option<String> {
    let (maybe_protocol, maybe_endpoint) =
        read_protocol_and_endpoint_from_env(protocol_key, endpoint_key, endpoint_path);
    infer_protocol(maybe_protocol.as_deref(), maybe_endpoint.as_deref())
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

pub fn debug_env() {
    const SENSITIVE_KEYS: &[&str] = &[
        "OTEL_EXPORTER_OTLP_HEADERS",
        "OTEL_EXPORTER_OTLP_CERTIFICATE",
    ];
    std::env::vars()
        .filter(|(k, _)| k.starts_with("OTEL_"))
        .for_each(|(k, v)| {
            let display_value = if SENSITIVE_KEYS.iter().any(|s| k == *s) {
                "[redacted]"
            } else {
                &v
            };
            tracing::debug!(target: "otel::setup::env", key = %k, value = %display_value);
        });
}

fn read_protocol_and_endpoint_from_env(
    protocol_key: &str,
    endpoint_key: &str,
    endpoint_path: &str,
) -> (Option<String>, Option<String>) {
    let maybe_protocol = std::env::var(protocol_key)
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL"))
        .ok();
    let maybe_endpoint = std::env::var(endpoint_key)
        .or_else(|_| {
            std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").map(|endpoint| match &maybe_protocol {
                Some(protocol) if protocol.contains("http") => {
                    format!("{endpoint}/{endpoint_path}")
                }
                _ => endpoint,
            })
        })
        .ok();
    (maybe_protocol, maybe_endpoint)
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
