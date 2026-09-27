//#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

pub mod middleware;
#[cfg(feature = "metrics-prometheus")]
pub mod prometheus_metrics;

// reexport tracing_opentelemetry_instrumentation_sdk crate
pub use tracing_opentelemetry_instrumentation_sdk;
