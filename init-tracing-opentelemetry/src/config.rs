//! Flexible tracing configuration with builder pattern.
//!
//! Provides [`TracingConfig`] for configurable tracing setup with format options,
//! output destinations, level filtering, and OpenTelemetry integration.
//!
//! # Example
//! ```no_run
//! use init_tracing_opentelemetry::TracingConfig;
//!
//! // Use preset with global subscriber (default)
//! let _guard = TracingConfig::development().init_subscriber()?;
//!
//! // Custom configuration with global subscriber
//! let _guard = TracingConfig::default()
//!     .with_json_format()
//!     .with_stderr()
//!     .with_log_directives("debug")
//!     .init_subscriber()?;
//!
//! // Non-global subscriber (thread-local)
//! let guard = TracingConfig::development()
//!     .with_global_subscriber(false)
//!     .init_subscriber()?;
//! // Guard must be kept alive for subscriber to remain active
//! assert!(guard.is_non_global());
//!
//! // Without OpenTelemetry (just logging)
//! let guard = TracingConfig::minimal()
//!     .with_otel(false)
//!     .init_subscriber()?;
//! // Works fine - guard.otel_guard is None
//! assert!(!guard.has_otel());
//! assert!(guard.otel_guard.is_none());
//!
//! // Direct field access is also possible
//! if let Some(otel_guard) = &guard.otel_guard {
//!     // Use otel_guard...
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::path::{Path, PathBuf};

use tracing::{info, level_filters::LevelFilter, Subscriber};
use tracing_subscriber::{
    filter::EnvFilter, fmt::format::FmtSpan, layer::SubscriberExt, registry::LookupSpan, Layer,
    Registry,
};

#[cfg(feature = "logfmt")]
use crate::formats::LogfmtLayerBuilder;
use crate::formats::{
    CompactLayerBuilder, FullLayerBuilder, JsonLayerBuilder, LayerBuilder, PrettyLayerBuilder,
};

use crate::tracing_subscriber_ext::regiter_otel_layers;
use crate::{otlp::OtelGuard, resource::DetectResource, Error};

/// Combined guard that handles both `OtelGuard` and optional `DefaultGuard`
///
/// This struct holds the various guards needed to maintain the tracing subscriber.
/// - `otel_guard`: OpenTelemetry guard for flushing traces/metrics on drop (None when OTEL disabled)
/// - `default_guard`: Subscriber default guard for non-global subscribers (None when using global)
#[must_use = "Recommend holding with 'let _guard = ' pattern to ensure final traces/log/metrics are sent to the server and subscriber is maintained"]
pub struct Guard {
    /// OpenTelemetry guard for proper cleanup (None when OTEL is disabled)
    pub otel_guard: Option<OtelGuard>,
    /// Default subscriber guard for non-global mode (None when using global subscriber)
    pub default_guard: Option<tracing::subscriber::DefaultGuard>,
    // Easy to add in the future:
    // pub log_guard: Option<LogGuard>,
    // pub metrics_guard: Option<MetricsGuard>,
}

impl Guard {
    /// Create a new Guard for global subscriber mode
    pub fn global(otel_guard: Option<OtelGuard>) -> Self {
        Self {
            otel_guard,
            default_guard: None,
        }
    }

    /// Create a new Guard for non-global subscriber mode
    pub fn non_global(
        otel_guard: Option<OtelGuard>,
        default_guard: tracing::subscriber::DefaultGuard,
    ) -> Self {
        Self {
            otel_guard,
            default_guard: Some(default_guard),
        }
    }

    /// Get a reference to the underlying `OtelGuard` if present
    #[must_use]
    pub fn otel_guard(&self) -> Option<&OtelGuard> {
        self.otel_guard.as_ref()
    }

    /// Check if OpenTelemetry is enabled for this guard
    #[must_use]
    pub fn has_otel(&self) -> bool {
        self.otel_guard.is_some()
    }

    /// Check if this guard is managing a non-global (thread-local) subscriber
    #[must_use]
    pub fn is_non_global(&self) -> bool {
        self.default_guard.is_some()
    }

    /// Check if this guard is for a global subscriber
    #[must_use]
    pub fn is_global(&self) -> bool {
        self.default_guard.is_none()
    }
}

/// Configuration for log output format
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// Pretty formatted output with colors and indentation (development)
    Pretty,
    /// Structured JSON output (production)
    Json,
    /// Single-line output
    Full,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTimer {
    None,
    Time,
    Uptime,
}

impl Default for LogTimer {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            LogTimer::Uptime
        } else {
            LogTimer::Time
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
    /// Include file names in output
    pub file_names: bool,
    /// Include line numbers in output
    pub line_numbers: bool,
    /// Include thread names in output
    pub thread_names: bool,
    /// Include thread IDs in output
    pub thread_ids: bool,
    /// Configure time logging (wall clock, uptime or none)
    pub timer: LogTimer,
    /// Configure span event logging
    pub span_events: Option<FmtSpan>,
    /// Display target information
    pub target_display: bool,
}

impl Default for FeatureSet {
    fn default() -> Self {
        Self {
            file_names: true,
            line_numbers: cfg!(debug_assertions),
            thread_names: cfg!(debug_assertions),
            thread_ids: false,
            timer: LogTimer::default(),
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
#[derive(Debug)]
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
    /// Whether to set the subscriber as global default
    pub global_subscriber: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::default(),
            writer: WriterConfig::default(),
            level_config: LevelConfig::default(),
            features: FeatureSet::default(),
            otel_config: OtelConfig::default(),
            global_subscriber: true,
        }
    }
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

    /// Use full formatted output
    #[must_use]
    pub fn with_full_format(self) -> Self {
        self.with_format(LogFormat::Full)
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

    /// Enable or disable file names in output
    #[must_use]
    pub fn with_file_names(mut self, enabled: bool) -> Self {
        self.features.file_names = enabled;
        self
    }

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

    /// Enable or disable thread IDs in output
    #[must_use]
    pub fn with_thread_ids(mut self, enabled: bool) -> Self {
        self.features.thread_ids = enabled;
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
    #[deprecated = "Use `TracingConfig::with_timer` instead"]
    pub fn with_uptime_timer(mut self, enabled: bool) -> Self {
        self.features.timer = if enabled {
            LogTimer::Uptime
        } else {
            LogTimer::Time
        };
        self
    }

    /// Configure time logging (wall clock, uptime or none)
    #[must_use]
    pub fn with_timer(mut self, timer: LogTimer) -> Self {
        self.features.timer = timer;
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

    /// Set whether to initialize the subscriber as global default
    ///
    /// When `global` is true (default), the subscriber is set as the global default.
    /// When false, the subscriber is set as thread-local default and the returned
    /// Guard must be kept alive to maintain the subscriber.
    #[must_use]
    pub fn with_global_subscriber(mut self, global: bool) -> Self {
        self.global_subscriber = global;
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
            LogFormat::Full => FullLayerBuilder.build_layer(self),
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

    /// Initialize the tracing subscriber with this configuration
    ///
    /// If `global_subscriber` is true, sets the subscriber as the global default.
    /// If false, returns a Guard that maintains the subscriber as the thread-local default.
    ///
    /// When OpenTelemetry is disabled, the Guard will contain `None` for the `OtelGuard`.
    pub fn init_subscriber(self) -> Result<Guard, Error> {
        self.init_subscriber_ext(Self::transform_identity)
    }

    fn transform_identity(s: Registry) -> Registry {
        s
    }

    /// `transform` parameter allow to customize the registry/subscriber before
    /// the setup of opentelemetry, log, logfilter.
    /// ```text
    /// let guard = TracingConfig::default()
    ///    .with_json_format()
    ///    .with_stderr()
    ///    .init_subscriber_ext(|subscriber| subscriber.with(my_layer))?;
    /// ```
    pub fn init_subscriber_ext<F, SOut>(self, transform: F) -> Result<Guard, Error>
    where
        SOut: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
        F: FnOnce(Registry) -> SOut,
    {
        // Setup a temporary subscriber for initialization logging
        let temp_subscriber = tracing_subscriber::registry()
            .with(self.build_layer()?)
            .with(self.build_filter_layer()?);
        let _guard = tracing::subscriber::set_default(temp_subscriber);
        info!("init logging & tracing");

        // Build the final subscriber based on OTEL configuration
        if self.otel_config.enabled {
            let subscriber = transform(tracing_subscriber::registry());
            let (subscriber, otel_guard) = regiter_otel_layers(subscriber)?;
            let subscriber = subscriber
                .with(self.build_layer()?)
                .with(self.build_filter_layer()?);

            if self.global_subscriber {
                tracing::subscriber::set_global_default(subscriber)?;
                Ok(Guard::global(Some(otel_guard)))
            } else {
                let default_guard = tracing::subscriber::set_default(subscriber);
                Ok(Guard::non_global(Some(otel_guard), default_guard))
            }
        } else {
            info!("OpenTelemetry disabled - proceeding without OTEL layers");
            let subscriber = transform(tracing_subscriber::registry())
                .with(self.build_layer()?)
                .with(self.build_filter_layer()?);

            if self.global_subscriber {
                tracing::subscriber::set_global_default(subscriber)?;
                Ok(Guard::global(None))
            } else {
                let default_guard = tracing::subscriber::set_default(subscriber);
                Ok(Guard::non_global(None, default_guard))
            }
        }
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
    /// - non global registration (of subscriber)
    #[must_use]
    pub fn testing() -> Self {
        Self::default()
            .with_compact_format()
            .with_stderr()
            .with_line_numbers(false)
            .with_thread_names(false)
            .without_span_events()
            .with_otel(false)
            .with_global_subscriber(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_subscriber_true_returns_global_guard() {
        let config = TracingConfig::minimal()
            .with_global_subscriber(true)
            .with_otel(false); // Disable for simple test

        // This would actually initialize the subscriber, so we'll just test that
        // the config has the right value
        assert!(config.global_subscriber);
    }

    #[test]
    fn test_global_subscriber_false_sets_config() {
        let config = TracingConfig::minimal()
            .with_global_subscriber(false)
            .with_otel(false); // Disable for simple test

        assert!(!config.global_subscriber);
    }

    #[test]
    fn test_default_global_subscriber_is_true() {
        let config = TracingConfig::default();
        assert!(config.global_subscriber);
    }

    #[test]
    fn test_init_subscriber_without_otel_succeeds() {
        // Test that initialization succeeds when OTEL is disabled
        let guard = TracingConfig::minimal()
            .with_otel(false)
            .with_global_subscriber(false) // Use non-global to avoid affecting other tests
            .init_subscriber();

        assert!(guard.is_ok());
        let guard = guard.unwrap();

        // Verify that the guard indicates no OTEL
        assert!(!guard.has_otel());
        assert!(guard.otel_guard().is_none());
    }

    #[test]
    fn test_init_subscriber_with_otel_disabled_global() {
        // Test global subscriber mode with OTEL disabled
        let guard = TracingConfig::minimal()
            .with_otel(false)
            .with_global_subscriber(true)
            .init_subscriber();

        assert!(guard.is_ok());
        let guard = guard.unwrap();

        // Should be global mode with no OTEL
        assert!(guard.is_global());
        assert!(!guard.has_otel());
        assert!(guard.otel_guard().is_none());
    }

    #[test]
    fn test_init_subscriber_with_otel_disabled_non_global() {
        // Test non-global subscriber mode with OTEL disabled
        let guard = TracingConfig::minimal()
            .with_otel(false)
            .with_global_subscriber(false)
            .init_subscriber();

        assert!(guard.is_ok());
        let guard = guard.unwrap();

        // Should be non-global mode with no OTEL
        assert!(guard.is_non_global());
        assert!(!guard.has_otel());
        assert!(guard.otel_guard().is_none());
    }

    #[test]
    fn test_guard_helper_methods() {
        // Test the Guard helper methods work correctly with None values
        let guard_global_none = Guard::global(None);
        assert!(!guard_global_none.has_otel());
        assert!(guard_global_none.otel_guard().is_none());
        assert!(guard_global_none.is_global());
        assert!(!guard_global_none.is_non_global());
        assert!(guard_global_none.default_guard.is_none());

        // We can't easily create a DefaultGuard for testing, but we can test the constructor
        // Note: We can't actually create a DefaultGuard without setting up a real subscriber,
        // so we'll just test the struct design is sound
    }

    #[test]
    fn test_guard_struct_direct_field_access() {
        // Test that we can directly access fields, which is a benefit of the struct design
        let guard = Guard::global(None);

        // Direct field access is now possible
        assert!(guard.otel_guard.is_none());
        assert!(guard.default_guard.is_none());

        // Helper methods still work
        assert!(!guard.has_otel());
        assert!(guard.is_global());
    }

    #[test]
    fn test_guard_struct_extensibility() {
        // This test demonstrates how the struct design makes it easier to extend
        // We can easily add more optional guards in the future without breaking existing code
        let guard = Guard {
            otel_guard: None,
            default_guard: None,
            // Future: log_guard: None, metrics_guard: None, etc.
        };

        assert!(guard.is_global());
        assert!(!guard.has_otel());
    }

    #[tokio::test]
    async fn test_init_with_transform() {
        use std::time::Duration;
        use tokio_blocked::TokioBlockedLayer;
        let blocked =
            TokioBlockedLayer::new().with_warn_busy_single_poll(Some(Duration::from_micros(150)));

        let guard = TracingConfig::default()
            .with_json_format()
            .with_stderr()
            .with_log_directives("debug")
            .with_global_subscriber(false)
            .init_subscriber_ext(|subscriber| subscriber.with(blocked))
            .unwrap();

        assert!(!guard.is_global());
        assert!(guard.has_otel());
    }
}
