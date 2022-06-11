#![forbid(unsafe_code)]

mod middleware;

pub use self::middleware::opentelemetry_tracing_layer;
