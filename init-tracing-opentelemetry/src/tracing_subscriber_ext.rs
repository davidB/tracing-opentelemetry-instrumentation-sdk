use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};
use tracing::{info, level_filters::LevelFilter, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry::LookupSpan, Layer};

use crate::otlp::OtelGuard;
use crate::Error;

#[cfg(not(feature = "logfmt"))]
#[must_use]
pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    use tracing_subscriber::fmt::format::FmtSpan;
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_line_number(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    }
}

#[cfg(feature = "logfmt")]
#[must_use]
pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    //FIXME tracing_logfmt use an old version of crates, how to inject trace_id and span_id into log?
    Box::new(tracing_logfmt::layer())
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

pub fn build_otel_layer<S>() -> Result<(OpenTelemetryLayer<S, Tracer>, SdkTracerProvider), Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    use crate::{
        init_propagator, //stdio,
        otlp,
        resource::DetectResource,
    };
    use opentelemetry::global;
    let otel_rsrc = DetectResource::default()
        //.with_fallback_service_name(env!("CARGO_PKG_NAME"))
        //.with_fallback_service_version(env!("CARGO_PKG_VERSION"))
        .build();
    let tracer_provider = otlp::traces::init_tracerprovider(otel_rsrc, otlp::traces::identity)?;
    // to not send trace somewhere, but continue to create and propagate,...
    // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
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
    global::set_tracer_provider(tracer_provider.clone());
    Ok((layer, tracer_provider))
}

pub fn init_subscribers() -> Result<OtelGuard, Error> {
    init_subscribers_and_loglevel("")
}

/// see [`build_level_filter_layer`] for the syntax of `log_directives`
pub fn init_subscribers_and_loglevel(log_directives: &str) -> Result<OtelGuard, Error> {
    //setup a temporary subscriber to log output during setup
    let subscriber = tracing_subscriber::registry()
        .with(build_level_filter_layer(log_directives)?)
        .with(build_logger_text());
    let _guard = tracing::subscriber::set_default(subscriber);
    info!("init logging & tracing");

    let (trace_layer, tracer_provider) = build_otel_layer()?;

    #[cfg(not(feature = "metrics"))]
    {
        let subscriber = tracing_subscriber::registry()
            .with(trace_layer)
            .with(build_level_filter_layer(log_directives)?)
            .with(build_logger_text());
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(OtelGuard { tracer_provider })
    }
    #[cfg(feature = "metrics")]
    {
        let (metrics_layer, meter_provider) = crate::otlp::metrics::build_metrics_layer()?;
        info!("init metrics");
        let subscriber = tracing_subscriber::registry()
            .with(metrics_layer)
            .with(trace_layer)
            .with(build_level_filter_layer(log_directives)?)
            .with(build_logger_text());
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(OtelGuard {
            meter_provider,
            tracer_provider,
        })
    }
}
