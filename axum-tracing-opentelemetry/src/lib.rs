#![forbid(unsafe_code)]
#[allow(deprecated)]
pub mod middleware;

/// for basic backward compatibility and transition
pub use self::middleware::opentelemetry_tracing_layer;
/// for basic backward compatibility and transition
pub use self::middleware::response_with_trace_layer;
