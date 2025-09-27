//! Flexible tracing configuration with builder pattern.
//!
//! Provides [`TracingConfig`] for configurable tracing setup with format options,
//! output destinations, level filtering, and OpenTelemetry integration.
//!
//! # Example
//! ```no_run
//! use init_tracing_opentelemetry::TracingConfig;
//!
//! // Use preset
//! let _guard = TracingConfig::development().init_subscriber()?;
//!
//! // Custom configuration
//! let _guard = TracingConfig::default()
//!     .with_json_format()
//!     .with_stderr()
//!     .with_log_directives("debug")
//!     .init_subscriber()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::path::{Path, PathBuf};

use tracing::{info, level_filters::LevelFilter, Subscriber};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, registry::LookupSpan, Layer};

#[cfg(feature = "logfmt")]
use crate::formats::LogfmtLayerBuilder;
use crate::formats::{CompactLayerBuilder, JsonLayerBuilder, LayerBuilder, PrettyLayerBuilder};

use crate::tracing_subscriber_ext::regiter_otel_layers;
use crate::{otlp::OtelGuard, resource::DetectResource, Error};

/// Configuration for log output format
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// Pretty formatted output with colors and indentation (development)
    Pretty,
    /// Structured JSON output (production)
    Json,
    /// Single-line compact output
    Compact,
    /// Key=value logfmt format
    #[cfg(feature = "logfmt")]
    Logfmt,
}

impl Default for LogFormat {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            LogFormat::Pretty
        } else {
            LogFormat::Json
        }
    }
}

/// Configuration for log output destination
#[derive(Debug, Clone, Default)]
pub enum WriterConfig {
    /// Write to stdout
    #[default]
    Stdout,
    /// Write to stderr
    Stderr,
    /// Write to a file
    File(PathBuf),
}

/// Configuration for log level filtering
#[derive(Debug, Clone)]
pub struct LevelConfig {
    /// Log directives string (takes precedence over env vars)
    pub directives: String,
    /// Environment variable fallbacks (checked in order)
    pub env_fallbacks: Vec<String>,
    /// Default level when no directives or env vars are set
    pub default_level: LevelFilter,
    /// OpenTelemetry tracing level
    pub otel_trace_level: LevelFilter,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            directives: String::new(),
            env_fallbacks: vec!["RUST_LOG".to_string(), "OTEL_LOG_LEVEL".to_string()],
            default_level: LevelFilter::INFO,
            otel_trace_level: LevelFilter::TRACE,
        }
    }
}

/// Configuration for optional tracing features
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FeatureSet {
    /// Include line numbers in output
    pub line_numbers: bool,
    /// Include thread names in output
    pub thread_names: bool,
    /// Use uptime timer instead of wall clock
    pub uptime_timer: bool,
    /// Configure span event logging
    pub span_events: Option<FmtSpan>,
    /// Display target information
    pub target_display: bool,
}

impl Default for FeatureSet {
    fn default() -> Self {
        Self {
            line_numbers: cfg!(debug_assertions),
            thread_names: cfg!(debug_assertions),
            uptime_timer: true,
            span_events: if cfg!(debug_assertions) {
                Some(FmtSpan::NEW | FmtSpan::CLOSE)
            } else {
                None
            },
            target_display: true,
        }
    }
}

/// Configuration for OpenTelemetry integration
#[derive(Debug)]
pub struct OtelConfig {
    /// Enable OpenTelemetry tracing
    pub enabled: bool,
    /// Resource configuration for OTEL
    pub resource_config: Option<DetectResource>,
    /// Enable metrics collection
    pub metrics_enabled: bool,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            resource_config: None,
            metrics_enabled: cfg!(feature = "metrics"),
        }
    }
}

/// Main configuration builder for tracing setup
/// Default create a new tracing configuration with sensible defaults
#[derive(Debug, Default)]
pub struct TracingConfig {
    /// Output format configuration
    pub format: LogFormat,
    /// Output destination configuration
    pub writer: WriterConfig,
    /// Level filtering configuration
    pub level_config: LevelConfig,
    /// Optional features configuration
    pub features: FeatureSet,
    /// OpenTelemetry configuration
    pub otel_config: OtelConfig,
}

impl TracingConfig {
    // === Format Configuration ===

    /// Set the log format
    #[must_use]
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Use pretty formatted output (development style)
    #[must_use]
    pub fn with_pretty_format(self) -> Self {
        self.with_format(LogFormat::Pretty)
    }

    /// Use JSON formatted output (production style)
    #[must_use]
    pub fn with_json_format(self) -> Self {
        self.with_format(LogFormat::Json)
    }

    /// Use compact formatted output
    #[must_use]
    pub fn with_compact_format(self) -> Self {
        self.with_format(LogFormat::Compact)
    }

    /// Use logfmt formatted output (requires 'logfmt' feature)
    #[must_use]
    #[cfg(feature = "logfmt")]
    pub fn with_logfmt_format(self) -> Self {
        self.with_format(LogFormat::Logfmt)
    }

    // === Writer Configuration ===

    /// Set the output writer
    #[must_use]
    pub fn with_writer(mut self, writer: WriterConfig) -> Self {
        self.writer = writer;
        self
    }

    /// Write logs to stdout
    #[must_use]
    pub fn with_stdout(self) -> Self {
        self.with_writer(WriterConfig::Stdout)
    }

    /// Write logs to stderr
    #[must_use]
    pub fn with_stderr(self) -> Self {
        self.with_writer(WriterConfig::Stderr)
    }

    /// Write logs to a file
    #[must_use]
    pub fn with_file<P: AsRef<Path>>(self, path: P) -> Self {
        self.with_writer(WriterConfig::File(path.as_ref().to_path_buf()))
    }

    // === Level Configuration ===

    /// Set log directives (takes precedence over environment variables),
    /// for example if you want to set it from cli arguments (verbosity)
    #[must_use]
    pub fn with_log_directives(mut self, directives: impl Into<String>) -> Self {
        self.level_config.directives = directives.into();
        self
    }

    /// Set the default log level
    #[must_use]
    pub fn with_default_level(mut self, level: LevelFilter) -> Self {
        self.level_config.default_level = level;
        self
    }

    /// Add an environment variable fallback for log configuration
    #[must_use]
    pub fn with_env_fallback(mut self, env_var: impl Into<String>) -> Self {
        self.level_config.env_fallbacks.push(env_var.into());
        self
    }

    /// Set the OpenTelemetry trace level
    #[must_use]
    pub fn with_otel_trace_level(mut self, level: LevelFilter) -> Self {
        self.level_config.otel_trace_level = level;
        self
    }

    // === Feature Configuration ===

    /// Enable or disable line numbers in output
    #[must_use]
    pub fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.features.line_numbers = enabled;
        self
    }

    /// Enable or disable thread names in output
    #[must_use]
    pub fn with_thread_names(mut self, enabled: bool) -> Self {
        self.features.thread_names = enabled;
        self
    }

    /// Configure span event logging
    #[must_use]
    pub fn with_span_events(mut self, events: FmtSpan) -> Self {
        self.features.span_events = Some(events);
        self
    }

    /// Disable span event logging
    #[must_use]
    pub fn without_span_events(mut self) -> Self {
        self.features.span_events = None;
        self
    }

    /// Enable or disable uptime timer (vs wall clock)
    #[must_use]
    pub fn with_uptime_timer(mut self, enabled: bool) -> Self {
        self.features.uptime_timer = enabled;
        self
    }

    /// Enable or disable target display
    #[must_use]
    pub fn with_target_display(mut self, enabled: bool) -> Self {
        self.features.target_display = enabled;
        self
    }

    // === OpenTelemetry Configuration ===

    /// Enable or disable OpenTelemetry tracing
    #[must_use]
    pub fn with_otel(mut self, enabled: bool) -> Self {
        self.otel_config.enabled = enabled;
        self
    }

    /// Enable or disable metrics collection
    #[must_use]
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.otel_config.metrics_enabled = enabled;
        self
    }

    /// Set resource configuration for OpenTelemetry
    #[must_use]
    pub fn with_resource_config(mut self, config: DetectResource) -> Self {
        self.otel_config.resource_config = Some(config);
        self
    }

    // === Build Methods ===

    /// Build a tracing layer with the current configuration
    pub fn build_layer<S>(&self) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, Error>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        match &self.format {
            LogFormat::Pretty => PrettyLayerBuilder.build_layer(self),
            LogFormat::Json => JsonLayerBuilder.build_layer(self),
            LogFormat::Compact => CompactLayerBuilder.build_layer(self),
            #[cfg(feature = "logfmt")]
            LogFormat::Logfmt => LogfmtLayerBuilder.build_layer(self),
        }
    }

    /// Build a level filter layer with the current configuration
    pub fn build_filter_layer(&self) -> Result<EnvFilter, Error> {
        // Use existing function but with our configuration
        let dirs = if self.level_config.directives.is_empty() {
            // Try environment variables in order
            self.level_config
                .env_fallbacks
                .iter()
                .find_map(|var| std::env::var(var).ok())
                .unwrap_or_else(|| self.level_config.default_level.to_string().to_lowercase())
        } else {
            self.level_config.directives.clone()
        };

        let directive_to_allow_otel_trace = format!(
            "otel::tracing={}",
            self.level_config
                .otel_trace_level
                .to_string()
                .to_lowercase()
        )
        .parse()?;

        Ok(EnvFilter::builder()
            .with_default_directive(self.level_config.default_level.into())
            .parse_lossy(dirs)
            .add_directive(directive_to_allow_otel_trace))
    }

    /// Initialize the global tracing subscriber with this configuration
    pub fn init_subscriber(self) -> Result<OtelGuard, Error> {
        // Setup a temporary subscriber for initialization logging
        let temp_subscriber = tracing_subscriber::registry()
            .with(self.build_filter_layer()?)
            .with(self.build_layer()?);
        let _guard = tracing::subscriber::set_default(temp_subscriber);
        info!("init logging & tracing");

        // Build the final subscriber
        let subscriber = tracing_subscriber::registry();
        let (subscriber, otel_guard) = if self.otel_config.enabled {
            regiter_otel_layers(subscriber)?
        } else {
            // Create a dummy OtelGuard for the case when OTEL is disabled
            // This will require modifying OtelGuard to handle this case
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "OpenTelemetry disabled - OtelGuard creation not yet supported",
            )
            .into());
        };

        let subscriber = subscriber
            .with(self.build_filter_layer()?)
            .with(self.build_layer()?);

        tracing::subscriber::set_global_default(subscriber)?;
        Ok(otel_guard)
    }

    // === Preset Configurations ===

    /// Configuration preset for development environments
    /// - Pretty formatting with colors
    /// - Output to stderr
    /// - Line numbers and thread names enabled
    /// - Span events for NEW and CLOSE
    /// - Full OpenTelemetry integration
    #[must_use]
    pub fn development() -> Self {
        Self::default()
            .with_pretty_format()
            .with_stderr()
            .with_line_numbers(true)
            .with_thread_names(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_otel(true)
    }

    /// Configuration preset for production environments
    /// - JSON formatting for structured logging
    /// - Output to stdout
    /// - Minimal metadata (no line numbers or thread names)
    /// - No span events to reduce verbosity
    /// - Full OpenTelemetry integration
    #[must_use]
    pub fn production() -> Self {
        Self::default()
            .with_json_format()
            .with_stdout()
            .with_line_numbers(false)
            .with_thread_names(false)
            .without_span_events()
            .with_otel(true)
    }

    /// Configuration preset for debugging
    /// - Pretty formatting with full verbosity
    /// - Output to stderr
    /// - All metadata enabled
    /// - Full span events
    /// - Debug level logging
    /// - Full OpenTelemetry integration
    #[must_use]
    pub fn debug() -> Self {
        Self::development()
            .with_log_directives("debug")
            .with_span_events(FmtSpan::FULL)
            .with_target_display(true)
    }

    /// Configuration preset for minimal logging
    /// - Compact formatting
    /// - Output to stdout
    /// - No metadata or extra features
    /// - OpenTelemetry disabled for minimal overhead
    #[must_use]
    pub fn minimal() -> Self {
        Self::default()
            .with_compact_format()
            .with_stdout()
            .with_line_numbers(false)
            .with_thread_names(false)
            .without_span_events()
            .with_target_display(false)
            .with_otel(false)
    }

    /// Configuration preset for testing environments
    /// - Compact formatting for less noise
    /// - Output to stderr to separate from test output
    /// - Basic metadata
    /// - OpenTelemetry disabled for speed
    #[must_use]
    pub fn testing() -> Self {
        Self::default()
            .with_compact_format()
            .with_stderr()
            .with_line_numbers(false)
            .with_thread_names(false)
            .without_span_events()
            .with_otel(false)
    }
}
