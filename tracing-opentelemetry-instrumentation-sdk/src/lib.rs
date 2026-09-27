//#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]
#![cfg_attr(docs_rs, feature(doc_cfg))]

#[cfg(feature = "http")]
#[cfg_attr(docs_rs, doc(cfg(feature = "http")))]
pub mod http;
mod span_type;

use opentelemetry::Context;

/// tracing's target used by instrumentation library to create span
pub const TRACING_TARGET: &str = "otel::tracing";

#[cfg(not(feature = "tracing_level_info"))]
pub const TRACING_LEVEL: tracing::Level = tracing::Level::TRACE;

#[cfg(feature = "tracing_level_info")]
pub const TRACING_LEVEL: tracing::Level = tracing::Level::INFO;

// const SPAN_NAME_FIELD: &str = "otel.name";
// const SPAN_KIND_FIELD: &str = "otel.kind";
// const SPAN_STATUS_CODE_FIELD: &str = "otel.status_code";
// const SPAN_STATUS_MESSAGE_FIELD: &str = "otel.status_message";

// const FIELD_EXCEPTION_MESSAGE: &str = "exception.message";
// const FIELD_EXCEPTION_STACKTRACE: &str = "exception.stacktrace";
// const HTTP_TARGET: &str = opentelemetry_semantic_conventions::trace::HTTP_TARGET.as_str();

/// Constructs a span for the target `TRACING_TARGET` with the level `TRACING_LEVEL`.
///
/// [Fields] and [attributes] are set using the same syntax as the [`tracing::span!`]
/// macro.
//TODO find a way to use opentelemetry_semantic_conventions::attribute::* as part of the field
#[macro_export]
macro_rules! otel_trace_span {
    (parent: $parent:expr, $name:expr, $($field:tt)*) => {
        tracing::span!(
            target: $crate::TRACING_TARGET,
            parent: $parent,
            $crate::TRACING_LEVEL,
            $name,
            $($field)*
        )
    };
    (parent: $parent:expr, $name:expr) => {
        $crate::otel_trace_span!(parent: $parent, $name,)
    };
    ($name:expr, $($field:tt)*) => {
        tracing::span!(
            target: $crate::TRACING_TARGET,
            $crate::TRACING_LEVEL,
            $name,
            $($field)*
        )
    };
    ($name:expr) => {
        $crate::otel_trace_span!($name,)
    };
}

#[inline]
#[must_use]
pub fn find_current_context() -> Context {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    // OpenTelemetry Context is propagation inside code is done via tracing crate
    context_or_current(tracing::Span::current().context())
}

/// Search the current opentelemetry trace id into the Context from the current tracing'span.
/// This function can be used to report the trace id into the error message send back to user.
///
/// ```rust
/// let trace_id = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
/// // json!({ "error" :  "xxxxxx", "trace_id": trace_id})
///
/// ```
#[inline]
#[must_use]
pub fn find_current_trace_id() -> Option<String> {
    find_trace_id(&find_current_context())
}

#[inline]
#[must_use]
pub fn find_context_from_tracing(span: &tracing::Span) -> Context {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    // OpenTelemetry Context is propagation inside code is done via tracing crate
    context_or_current(span.context())
}

#[inline]
#[must_use]
pub fn find_trace_id_from_tracing(span: &tracing::Span) -> Option<String> {
    find_trace_id(&find_context_from_tracing(span))
}

/// Use the current OpenTelemetry [`Context`] when the span has no valid OpenTelemetry context
/// (span disabled by a filter, no `OpenTelemetryLayer`, ...), so a remote context attached by
/// the server middleware is still propagated (see [`set_parent_or_fallback`]).
fn context_or_current(context: Context) -> Context {
    use opentelemetry::trace::TraceContextExt;
    if context.span().span_context().is_valid() {
        context
    } else {
        Context::current()
    }
}

/// Set `context` as parent of `span`.
///
/// Returns `context` back when it can not be stored into the span (span disabled by a filter,
/// no `OpenTelemetryLayer`, ...). The caller should then attach it as current OpenTelemetry
/// [`Context`] while the request is processed, so child spans and outgoing requests keep the
/// remote `trace_id`, even without exporter.
#[must_use]
pub fn set_parent_or_fallback(span: &tracing::Span, context: Context) -> Option<Context> {
    use tracing_opentelemetry::{OpenTelemetrySpanExt, SetParentError};
    match span.set_parent(context.clone()) {
        Ok(()) => None,
        Err(error @ SetParentError::AlreadyStarted) => {
            tracing::warn!(?error, "can not set parent trace_id to span");
            None
        }
        Err(_) => Some(context),
    }
}

#[inline]
#[must_use]
pub fn find_trace_id(context: &Context) -> Option<String> {
    use opentelemetry::trace::TraceContextExt;

    let span = context.span();
    let span_context = span.span_context();
    span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string())

    // #[cfg(not(any(
    //     feature = "opentelemetry_0_17",
    //     feature = "opentelemetry_0_18",
    //     feature = "opentelemetry_0_19"
    // )))]
    // let trace_id = span.context().span().span_context().trace_id().to_hex();

    // #[cfg(any(
    //     feature = "opentelemetry_0_17",
    //     feature = "opentelemetry_0_18",
    //     feature = "opentelemetry_0_19"
    // ))]
    // let trace_id = {
    //     let id = span.context().span().span_context().trace_id();
    //     format!("{:032x}", id)
    // };
}

#[inline]
#[must_use]
pub fn find_span_id(context: &Context) -> Option<String> {
    use opentelemetry::trace::TraceContextExt;

    let span = context.span();
    let span_context = span.span_context();
    span_context
        .is_valid()
        .then(|| span_context.span_id().to_string())
}

// pub(crate) fn set_otel_parent(parent_context: Context, span: &tracing::Span) {
//     use opentelemetry::trace::TraceContextExt as _;
//     use tracing_opentelemetry::OpenTelemetrySpanExt as _;

//     // let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
//     //     propagator.extract(&RequestHeaderCarrier::new(req.headers()))
//     // });
//     span.set_parent(parent_context);
//     // If we have a remote parent span, this will be the parent's trace identifier.
//     // If not, it will be the newly generated trace identifier with this request as root span.

//     if let Some(trace_id) = find_trace_id_from_tracing(&span) {
//         span.record("trace_id", trace_id);
//     }
// }

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
