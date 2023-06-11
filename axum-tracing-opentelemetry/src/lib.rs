#![forbid(unsafe_code)]

mod middleware;

pub use self::middleware::response_with_trace_layer;
pub use self::middleware::{opentelemetry_tracing_layer, opentelemetry_tracing_layer_grpc};
