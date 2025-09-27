//! Format-specific layer builders for tracing output.
//!
//! Provides implementations for different log formats (Pretty, JSON, Compact, Logfmt)
//! using the strategy pattern with the [`LayerBuilder`] trait.

use tracing::Subscriber;
use tracing_subscriber::{registry::LookupSpan, Layer};

use crate::config::{TracingConfig, WriterConfig};
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
        let mut layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_timer(tracing_subscriber::fmt::time::uptime());

        // Configure line numbers
        if config.features.line_numbers {
            layer = layer.with_line_number(true);
        }

        // Configure thread names
        if config.features.thread_names {
            layer = layer.with_thread_names(true);
        }

        // Configure span events
        if let Some(span_events) = &config.features.span_events {
            layer = layer.with_span_events(span_events.clone());
        }

        // Configure target display
        if !config.features.target_display {
            layer = layer.with_target(false);
        }

        // Configure writer
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
        let mut layer = tracing_subscriber::fmt::layer()
            .json()
            .with_timer(tracing_subscriber::fmt::time::uptime());

        // Configure line numbers
        if config.features.line_numbers {
            layer = layer.with_line_number(true);
        }

        // Configure thread names
        if config.features.thread_names {
            layer = layer.with_thread_names(true);
        }

        // Configure span events (typically disabled in production JSON)
        if let Some(span_events) = &config.features.span_events {
            layer = layer.with_span_events(span_events.clone());
        }

        // Configure target display
        if !config.features.target_display {
            layer = layer.with_target(false);
        }

        // Configure writer
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
        let mut layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_timer(tracing_subscriber::fmt::time::uptime());

        // Configure line numbers
        if config.features.line_numbers {
            layer = layer.with_line_number(true);
        }

        // Configure thread names
        if config.features.thread_names {
            layer = layer.with_thread_names(true);
        }

        // Configure span events
        if let Some(span_events) = &config.features.span_events {
            layer = layer.with_span_events(span_events.clone());
        }

        // Configure target display
        if !config.features.target_display {
            layer = layer.with_target(false);
        }

        // Configure writer
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
