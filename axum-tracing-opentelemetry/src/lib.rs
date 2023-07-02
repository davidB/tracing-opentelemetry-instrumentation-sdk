//#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

#[allow(deprecated)]
pub mod middleware;

/// for basic backward compatibility and transition
#[allow(deprecated)]
pub use self::middleware::opentelemetry_tracing_layer;
/// for basic backward compatibility and transition
#[allow(deprecated)]
pub use self::middleware::response_with_trace_layer;
