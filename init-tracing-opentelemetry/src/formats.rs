//! Format-specific layer builders for tracing output.
//!
//! Provides implementations for different log formats (Pretty, JSON, Compact, Logfmt)
//! using the strategy pattern with the [`LayerBuilder`] trait.

use tracing::Subscriber;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::{time, uptime, Uptime};
use tracing_subscriber::{registry::LookupSpan, Layer};

use crate::config::{LogTimer, TracingConfig, WriterConfig};
use crate::Error;

/// Trait for building format-specific tracing layers
pub trait LayerBuilder: Send + Sync {
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
    // Configure file names
    layer = layer.with_file(config.features.file_names);

    // Configure line numbers
    layer = layer.with_line_number(config.features.line_numbers);

    // Configure thread names
    layer = layer.with_thread_names(config.features.thread_names);

    // Configure thread IDs
    layer = layer.with_thread_ids(config.features.thread_ids);

    // Configure span events
    let span_events = config
        .features
        .span_events
        .as_ref()
        .map_or(FmtSpan::NONE, ToOwned::to_owned);
    layer = layer.with_span_events(span_events);

    // Configure target display
    layer = layer.with_target(config.features.target_display);

    // Configure timer and writer
    match config.features.timer {
        LogTimer::None => configure_writer(layer.without_time(), config),
        LogTimer::Time => configure_writer(layer.with_timer(time()), config),
        LogTimer::Uptime => configure_writer(layer.with_timer(uptime()), config),
    }
}

fn configure_writer<S, N, L, T, W>(
    layer: fmt::Layer<S, N, fmt::format::Format<L, T>, W>,
    config: &TracingConfig,
) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> fmt::FormatFields<'writer> + Send + Sync + 'static,
    L: Send + Sync + 'static,
    T: Send + Sync + 'static,
    fmt::format::Format<L, T>: fmt::FormatEvent<S, N>,
{
    match &config.writer {
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
        // Note: tracing_logfmt doesn't support the same configuration options
        // as the standard fmt layer, so we have limited configuration ability

        match &config.writer {
            WriterConfig::Stderr => {
                // For stderr, we need to use the builder pattern since layer() doesn't support with_writer
                // However, the current tracing_logfmt version may not support this
                // For now, we'll fall back to the basic layer
                Ok(Box::new(tracing_logfmt::layer()))
            }
            _ => {
                // Default behavior uses stdout
                Ok(Box::new(tracing_logfmt::layer()))
            }
        }
    }
}
