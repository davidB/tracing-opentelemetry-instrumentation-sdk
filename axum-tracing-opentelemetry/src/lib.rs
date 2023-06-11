#![forbid(unsafe_code)]

mod middleware;
mod tools;

pub use self::middleware::response_with_trace_layer;
pub use self::middleware::{opentelemetry_tracing_layer, opentelemetry_tracing_layer_grpc};
pub use self::tools::*;

#[cfg(feature = "tracer")]
#[deprecated(since = "0.9.0", note = "replace by `DetectResource` builder")]
pub use self::tools::resource::make_resource; // for backward compatibility
