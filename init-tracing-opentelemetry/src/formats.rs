//! Format-specific layer builders for tracing output.
//!
//! Provides implementations for different log formats (Pretty, JSON, Compact, Logfmt)
//! using the strategy pattern with the [`LayerBuilder`] trait.

use tracing::Subscriber;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::{FmtSpan, Writer};
use tracing_subscriber::fmt::time::{Uptime, time, uptime};
use tracing_subscriber::{Layer, registry::LookupSpan};

use crate::config::{LogFormat, LogTimer, TracingConfig, WriterConfig};
use crate::{Error, FeatureSet};

/// Trait for building format-specific tracing layers
pub trait LayerBuilder: Send + Sync {
    /// Construct a boxed tracing layer configured by `config`.
    fn build_layer<S>(
        &self,
        config: &TracingConfig,
    ) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>;
}

fn configure_layer<S, N, L, T, W>(
    mut layer: fmt::Layer<S, N, fmt::format::Format<L, T>, W>,
    config: &TracingConfig,
) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> fmt::FormatFields<'writer> + Send + Sync + 'static,
    L: Send + Sync + 'static,
    fmt::format::Format<L, ()>: fmt::FormatEvent<S, N>,
    fmt::format::Format<L, Uptime>: fmt::FormatEvent<S, N>,
    fmt::format::Format<L>: fmt::FormatEvent<S, N>,
    W: for<'writer> fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    // NOTE: Destructure to make sure we don’t miss a feature
    let FeatureSet {
        file_names,
        line_numbers,
        thread_names,
        thread_ids,
        timer,
        span_events,
        target_display,
        fmt_trace_id,
    } = &config.features;
    let span_events = span_events
        .as_ref()
        .map_or(FmtSpan::NONE, ToOwned::to_owned);

    // Configure features
    layer = layer
        .with_file(*file_names)
        .with_line_number(*line_numbers)
        .with_thread_names(*thread_names)
        .with_thread_ids(*thread_ids)
        .with_span_events(span_events)
        .with_target(*target_display);

    // Configure timer, trace_id and writer
    let trace_id = fmt_trace_id.then(|| matches!(config.format, LogFormat::Json));
    match timer {
        LogTimer::None => configure_trace_id(layer.without_time(), trace_id, &config.writer),
        LogTimer::Time => configure_trace_id(layer.with_timer(time()), trace_id, &config.writer),
        LogTimer::Uptime => {
            configure_trace_id(layer.with_timer(uptime()), trace_id, &config.writer)
        }
    }
}

/// `trace_id`: `None` = disabled, `Some(json)` = enabled (JSON or text output)
fn configure_trace_id<S, N, E, W>(
    layer: fmt::Layer<S, N, E, W>,
    trace_id: Option<bool>,
    writer: &WriterConfig,
) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> fmt::FormatFields<'writer> + Send + Sync + 'static,
    E: fmt::FormatEvent<S, N> + Send + Sync + 'static,
    W: for<'writer> fmt::MakeWriter<'writer> + 'static,
{
    match trace_id {
        Some(json) => configure_writer(
            layer.map_event_format(|inner| WithTraceId { inner, json }),
            writer,
        ),
        None => configure_writer(layer, writer),
    }
}

/// Event formatter wrapper that adds the current OpenTelemetry `trace_id` (when inside a span).
///
// ponytail: uses the entered span, not an explicit `parent:` of the event;
// fine for `trace_id` as parent and child share the same trace.
struct WithTraceId<F> {
    inner: F,
    json: bool,
}

impl<S, N, F> fmt::FormatEvent<S, N> for WithTraceId<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> fmt::FormatFields<'writer> + 'static,
    F: fmt::FormatEvent<S, N>,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // `Span::current()` (so `find_current_trace_id()`) is disabled while an event is dispatched,
        // but tracing-opentelemetry activates the OTel context of the entered span.
        let Some(trace_id) = tracing_opentelemetry_instrumentation_sdk::find_trace_id(
            &opentelemetry::Context::current(),
        ) else {
            return self.inner.format_event(ctx, writer, event);
        };
        if self.json {
            // JSON object: inject `trace_id` as first field
            let mut buf = String::new();
            self.inner.format_event(ctx, Writer::new(&mut buf), event)?;
            match buf.strip_prefix('{') {
                Some(rest) => write!(writer, "{{\"trace_id\":\"{trace_id}\",{rest}"),
                None => writer.write_str(&buf),
            }
        } else {
            write!(writer, "trace_id={trace_id} ")?;
            self.inner.format_event(ctx, writer, event)
        }
    }
}

fn configure_writer<S, N, E, W>(
    layer: fmt::Layer<S, N, E, W>,
    writer: &WriterConfig,
) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> fmt::FormatFields<'writer> + Send + Sync + 'static,
    E: fmt::FormatEvent<S, N> + Send + Sync + 'static,
{
    match writer {
        WriterConfig::Stdout => Ok(Box::new(layer.with_writer(std::io::stdout))),
        WriterConfig::Stderr => Ok(Box::new(layer.with_writer(std::io::stderr))),
        WriterConfig::File(path) => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            Ok(Box::new(layer.with_writer(file)))
        }
    }
}

/// Builder for pretty-formatted logs (development style)
#[derive(Debug, Default, Clone)]
pub struct PrettyLayerBuilder;

impl LayerBuilder for PrettyLayerBuilder {
    fn build_layer<S>(
        &self,
        config: &TracingConfig,
    ) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let layer = tracing_subscriber::fmt::layer().pretty();

        configure_layer(layer, config)
    }
}

/// Builder for JSON-formatted logs (production style)
#[derive(Debug, Default, Clone)]
pub struct JsonLayerBuilder;

impl LayerBuilder for JsonLayerBuilder {
    fn build_layer<S>(
        &self,
        config: &TracingConfig,
    ) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let layer = tracing_subscriber::fmt::layer().json();

        configure_layer(layer, config)
    }
}

/// Builder for full-formatted logs (default `tracing` style)
#[derive(Debug, Default, Clone)]
pub struct FullLayerBuilder;

impl LayerBuilder for FullLayerBuilder {
    fn build_layer<S>(
        &self,
        config: &TracingConfig,
    ) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let layer = tracing_subscriber::fmt::layer();

        configure_layer(layer, config)
    }
}

/// Builder for compact-formatted logs (minimal style)
#[derive(Debug, Default, Clone)]
pub struct CompactLayerBuilder;

impl LayerBuilder for CompactLayerBuilder {
    fn build_layer<S>(
        &self,
        config: &TracingConfig,
    ) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let layer = tracing_subscriber::fmt::layer().compact();

        configure_layer(layer, config)
    }
}

/// Builder for logfmt-formatted logs
#[cfg(feature = "logfmt")]
#[derive(Debug, Default, Clone)]
pub struct LogfmtLayerBuilder;

#[cfg(feature = "logfmt")]
impl LayerBuilder for LogfmtLayerBuilder {
    fn build_layer<S>(
        &self,
        config: &TracingConfig,
    ) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        // NOTE: Destructure to make sure we don’t miss a feature
        // (logfmt has its own timestamp format and a single `location` for file + line)
        let FeatureSet {
            file_names,
            line_numbers: _,
            thread_names,
            thread_ids,
            timer,
            span_events,
            target_display,
            fmt_trace_id,
        } = &config.features;
        let layer = tracing_logfmt::builder()
            .with_location(*file_names)
            .with_thread_names(*thread_names)
            .with_thread_ids(*thread_ids)
            .with_timestamp(!matches!(timer, LogTimer::None))
            .with_span_events(span_events.clone().unwrap_or(FmtSpan::NONE))
            .with_target(*target_display)
            .layer();
        // logfmt is text: `trace_id=...` prefix is a valid logfmt pair
        configure_trace_id(layer, fmt_trace_id.then_some(false), &config.writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use tracing_subscriber::layer::SubscriberExt;

    /// Log one event inside a span and one outside (with `fmt_trace_id` on), return the output lines
    fn log_lines(format: LogFormat) -> Vec<String> {
        let path = std::env::temp_dir().join(format!(
            "init-tracing-opentelemetry-{format:?}-{}.log",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let config = TracingConfig::default()
            .with_format(format)
            .with_file(&path)
            .without_span_events()
            .with_fmt_trace_id(true);
        let provider = SdkTracerProvider::builder().build();
        let registry = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
        let fmt_layer = match config.format {
            LogFormat::Json => JsonLayerBuilder.build_layer(&config),
            #[cfg(feature = "logfmt")]
            LogFormat::Logfmt => LogfmtLayerBuilder.build_layer(&config),
            _ => FullLayerBuilder.build_layer(&config),
        }
        .unwrap();
        tracing::subscriber::with_default(registry.with(fmt_layer), || {
            tracing::info_span!("span").in_scope(|| tracing::info!("inside"));
            tracing::info!("outside");
        });
        let out = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        out.lines().map(ToOwned::to_owned).collect()
    }

    fn assert_text_trace_id_only_inside_span(lines: &[String]) {
        assert_eq!(lines.len(), 2);
        let trace_id = lines[0].strip_prefix("trace_id=").unwrap();
        assert_eq!(trace_id.split(' ').next().unwrap().len(), 32);
        assert!(!lines[1].contains("trace_id="));
    }

    #[test]
    fn json_has_trace_id_only_inside_span() {
        let lines = log_lines(LogFormat::Json);
        assert_eq!(lines.len(), 2);
        let inside: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        let trace_id = inside["trace_id"].as_str().unwrap();
        assert_eq!(trace_id.len(), 32);
        assert_eq!(inside["fields"]["message"], "inside");
        let outside: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
        assert!(outside.get("trace_id").is_none());
    }

    #[test]
    fn text_has_trace_id_prefix_only_inside_span() {
        assert_text_trace_id_only_inside_span(&log_lines(LogFormat::Full));
    }

    #[cfg(feature = "logfmt")]
    #[test]
    fn logfmt_has_trace_id_prefix_only_inside_span() {
        let lines = log_lines(LogFormat::Logfmt);
        assert_text_trace_id_only_inside_span(&lines);
        assert!(lines[0].ends_with(" message=inside"));
    }
}
