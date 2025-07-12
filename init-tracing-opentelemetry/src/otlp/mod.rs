#[cfg(feature = "metrics")]
pub mod metrics;
pub mod traces;

use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;

#[must_use = "Recommend holding with 'let _guard = ' pattern to ensure final traces/metrics are sent to the server"]
/// On Drop of the `OtelGuard` instance,
/// the wrapped Tracer/Meter Provider is force to flush and to shutdown (ignoring error).
pub struct OtelGuard {
    #[cfg(feature = "metrics")]
    pub meter_provider: opentelemetry_sdk::metrics::SdkMeterProvider,
    pub tracer_provider: SdkTracerProvider,
}

impl OtelGuard {
    #[must_use]
    pub fn tracer_provider(&self) -> &impl TracerProvider {
        &self.tracer_provider
    }

    #[cfg(feature = "metrics")]
    #[must_use]
    pub fn meter_provider(&self) -> &impl opentelemetry::metrics::MeterProvider {
        &self.meter_provider
    }
}

impl Drop for OtelGuard {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        let _ = self.tracer_provider.force_flush();
        let _ = self.tracer_provider.shutdown();
        #[cfg(feature = "metrics")]
        {
            let _ = self.meter_provider.force_flush();
            let _ = self.meter_provider.shutdown();
        }
    }
}

#[allow(unused_mut)]
pub(crate) fn infer_protocol(
    maybe_protocol: Option<&str>,
    maybe_endpoint: Option<&str>,
) -> Option<String> {
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
