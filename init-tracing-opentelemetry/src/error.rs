/// Errors returned when initializing or configuring tracing.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SetGlobalDefaultError(#[from] tracing::subscriber::SetGlobalDefaultError),

    #[error(transparent)]
    OTelSdkError(#[from] opentelemetry_sdk::error::OTelSdkError),

    #[error("Setup error: {0}")]
    SetupError(String),

    #[cfg(feature = "otlp")]
    #[error(transparent)]
    ExporterBuildError(#[from] opentelemetry_otlp::ExporterBuildError),

    #[cfg(feature = "tracing_subscriber_ext")]
    #[error(transparent)]
    FilterParseError(#[from] tracing_subscriber::filter::ParseError),
}
