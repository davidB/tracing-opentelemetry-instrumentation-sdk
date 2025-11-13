use opentelemetry::trace::TracerProvider;
#[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::{
    trace::{SdkTracerProvider, Tracer},
    Resource,
};
use tracing::{level_filters::LevelFilter, Subscriber};
#[cfg(feature = "metrics")]
use tracing_opentelemetry::MetricsLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry::LookupSpan, Layer};

use crate::{
    config::TracingConfig,
    init_propagator, //stdio,
    otlp,
    otlp::OtelGuard,
    resource::DetectResource,
    Error,
};

#[must_use]
#[deprecated(
    since = "0.31.0",
    note = "Use `TracingConfig::default().build_layer()` instead"
)]
/// # Panics
/// Panics if the logger layer cannot be built.
pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    TracingConfig::default()
        .build_layer()
        .expect("Failed to build logger layer")
}

#[must_use]
#[deprecated = "replaced by the configurable build_level_filter_layer(\"\")"]
pub fn build_loglevel_filter_layer() -> EnvFilter {
    build_level_filter_layer("").unwrap_or_default()
}

/// Read the configuration from (first non empty used, priority top to bottom):
///
/// - from parameter `directives`
/// - from environment variable `RUST_LOG`
/// - from environment variable `OTEL_LOG_LEVEL`
/// - default to `Level::INFO`
///
/// And add directive to:
///
/// - `otel::tracing` should be a level info to emit opentelemetry trace & span
///
/// You can customize parameter "directives", by adding:
///
/// - `otel::setup=debug` set to debug to log detected resources, configuration read (optional)
///
/// see [Directives syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives)
pub fn build_level_filter_layer(log_directives: &str) -> Result<EnvFilter, Error> {
    let dirs = if log_directives.is_empty() {
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
            .unwrap_or_else(|_| "info".to_string())
    } else {
        log_directives.to_string()
    };
    let directive_to_allow_otel_trace = "otel::tracing=trace".parse()?;

    Ok(EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(dirs)
        .add_directive(directive_to_allow_otel_trace))
}

pub fn regiter_otel_layers<S>(
    subscriber: S,
) -> Result<(impl Subscriber + for<'span> LookupSpan<'span>, OtelGuard), Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    register_otel_layers_with_resource(subscriber, DetectResource::default().build())
}

pub fn register_otel_layers_with_resource<S>(
    subscriber: S,
    otel_rsrc: Resource,
) -> Result<(impl Subscriber + for<'span> LookupSpan<'span>, OtelGuard), Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    #[cfg(feature = "metrics")]
    let (metrics_layer, meter_provider) = build_metrics_layer_with_resource(otel_rsrc.clone())?;
    let (trace_layer, tracer_provider) = build_tracer_layer_with_resource(otel_rsrc)?;
    let subscriber = subscriber.with(trace_layer);
    #[cfg(feature = "metrics")]
    let subscriber = subscriber.with(metrics_layer);
    Ok((
        subscriber,
        OtelGuard {
            #[cfg(feature = "metrics")]
            meter_provider,
            tracer_provider,
        },
    ))
}

/// change (version 0.31): no longer set the glabal tracer
pub fn build_tracer_layer<S>() -> Result<(OpenTelemetryLayer<S, Tracer>, SdkTracerProvider), Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    build_tracer_layer_with_resource(
        DetectResource::default()
            //.with_fallback_service_name(env!("CARGO_PKG_NAME"))
            //.with_fallback_service_version(env!("CARGO_PKG_VERSION"))
            .build(),
    )
}

pub fn build_tracer_layer_with_resource<S>(
    otel_rsrc: Resource,
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
        .with_tracer(tracer_provider.tracer(""));
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    Ok((layer, tracer_provider))
}

#[cfg(feature = "metrics")]
pub fn build_metrics_layer<S>(
) -> Result<(MetricsLayer<S, SdkMeterProvider>, SdkMeterProvider), Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    build_metrics_layer_with_resource(DetectResource::default().build())
}

#[cfg(feature = "metrics")]
pub fn build_metrics_layer_with_resource<S>(
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

/// Initialize subscribers with default configuration
///
/// This is a convenience function that uses production-ready defaults.
/// For more control, use `TracingConfig::production().init_subscriber()`.
#[deprecated(
    since = "0.31.0",
    note = "Use `TracingConfig::production()...` instead"
)]
pub fn init_subscribers() -> Result<OtelGuard, Error> {
    let guard = TracingConfig::production().init_subscriber()?;
    match guard.otel_guard {
        Some(otel_guard) => {
            // For backward compatibility, we leak the default_guard since the caller
            // only expects an OtelGuard and won't hold onto the DefaultGuard
            if let Some(default_guard) = guard.default_guard {
                std::mem::forget(default_guard);
            }
            Ok(otel_guard)
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "OpenTelemetry is disabled but OtelGuard was requested",
        )
        .into()),
    }
}

/// Initialize subscribers with custom log directives
///
/// See [`build_level_filter_layer`] for the syntax of `log_directives`.
/// For more control, use `TracingConfig::production().with_log_directives(log_directives).init_subscriber()`.
#[deprecated(
    since = "0.31.0",
    note = "Use `TracingConfig::production().with_log_directives(log_directives)...` instead"
)]
pub fn init_subscribers_and_loglevel(log_directives: &str) -> Result<OtelGuard, Error> {
    let guard = TracingConfig::production()
        .with_log_directives(log_directives)
        .init_subscriber()?;
    match guard.otel_guard {
        Some(otel_guard) => {
            // For backward compatibility, we leak the default_guard since the caller
            // only expects an OtelGuard and won't hold onto the DefaultGuard
            if let Some(default_guard) = guard.default_guard {
                std::mem::forget(default_guard);
            }
            Ok(otel_guard)
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "OpenTelemetry is disabled but OtelGuard was requested",
        )
        .into()),
    }
}
