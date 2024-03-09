//#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

pub mod middleware;

// HACK vendor of tracing_opentelemetry_instrumentation_sdk until tonic can support hyper 1, http 1, ...
// TODO reexport tracing_opentelemetry_instrumentation_sdk crate
#[allow(dead_code)]
pub mod tracing_opentelemetry_instrumentation_sdk;
