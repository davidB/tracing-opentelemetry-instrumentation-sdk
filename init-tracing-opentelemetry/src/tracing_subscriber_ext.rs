//! Shared implementation helpers used by [`crate::config::TracingConfig`].
//!
//! The old procedural API that used to live here (`init_subscribers`, `build_tracer_layer`,
//! `register_otel_layers_with_resource`, etc.) was removed in 0.40.0. Use `TracingConfig` instead.
use std::borrow::Cow;

use opentelemetry::trace::TracerProvider;
#[cfg(feature = "logs")]
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
#[cfg(feature = "logs")]
use opentelemetry_sdk::logs::{SdkLogger, SdkLoggerProvider};
#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::{
    Resource,
    trace::{SdkTracerProvider, Tracer},
};
use tracing::Subscriber;
#[cfg(feature = "metrics")]
use tracing_opentelemetry::MetricsLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

use crate::{Error, init_propagator, otlp};

#[cfg(feature = "logs")]
pub(crate) fn build_logger_layer_with_resource(
    otel_rsrc: Resource,
) -> Result<
    (
        OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>,
        SdkLoggerProvider,
    ),
    crate::Error,
> {
    let logger_provider = otlp::logs::init_loggerprovider(otel_rsrc, otlp::logs::identity)?;
    let layer = OpenTelemetryTracingBridge::new(&logger_provider);
    Ok((layer, logger_provider))
}

pub(crate) fn build_tracer_layer_with_resource_and_name<S>(
    otel_rsrc: Resource,
    tracer_name: impl Into<Cow<'static, str>>,
) -> Result<(OpenTelemetryLayer<S, Tracer>, SdkTracerProvider), Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let tracer_provider = otlp::traces::init_tracerprovider(otel_rsrc, otlp::traces::identity)?;
    // to not send trace somewhere, but continue to create and propagate,...
    // then send them to `init_tracing_opentelemetry::stdio::WriteNoWhere::default()`
    // or to `std::io::stdout()` to print
    //
    // let otel_tracer = stdio::init_tracer(
    //     otel_rsrc,
    //     stdio::identity::<stdio::WriteNoWhere>,
    //     stdio::WriteNoWhere::default(),
    // )?;
    init_propagator()?;
    let layer = tracing_opentelemetry::layer()
        .with_error_records_to_exceptions(true)
        .with_tracer(tracer_provider.tracer(tracer_name));
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    Ok((layer, tracer_provider))
}

#[cfg(feature = "metrics")]
pub(crate) fn build_metrics_layer_with_resource<S>(
    otel_rsrc: Resource,
) -> Result<(MetricsLayer<S, SdkMeterProvider>, SdkMeterProvider), Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let meter_provider = otlp::metrics::init_meterprovider(otel_rsrc, otlp::metrics::identity)?;
    let layer = MetricsLayer::new(meter_provider.clone());
    opentelemetry::global::set_meter_provider(meter_provider.clone());
    Ok((layer, meter_provider))
}

#[cfg(feature = "metrics-prometheus")]
pub(crate) fn build_prometheus_metrics_layer_with_resource<S>(
    otel_rsrc: Resource,
) -> Result<
    (
        MetricsLayer<S, SdkMeterProvider>,
        SdkMeterProvider,
        prometheus::Registry,
    ),
    Error,
>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let (meter_provider, registry) =
        otlp::metrics_prometheus::init_meterprovider_prometheus(otel_rsrc)?;
    let layer = MetricsLayer::new(meter_provider.clone());
    opentelemetry::global::set_meter_provider(meter_provider.clone());
    Ok((layer, meter_provider, registry))
}
