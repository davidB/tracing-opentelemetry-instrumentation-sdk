# tracing-opentelemetry-instrumentation-sdk

Provide a set of helpers to build [OpenTelemetry] instrumentation based on [`tracing`] crate, and following the [OpenTelemetry Trace Semantic Conventions](https://github.com/open-telemetry/opentelemetry-specification/tree/v1.22.0/specification/trace/semantic_conventions).

PS: Contributions are welcome (bug report, improvements, features, ...)

Instrumentation on the caller side of a call is  composed of steps:

- start a span with all the attributes (some set to `Empty`)
- inject into the call (via header) the propagation data (if supported)
- do the call
- update attributes of the span with response (status,...)

Instrumentation on the callee side of a call is  composed of steps:

- extract info propagated info (from header) (if supported) an create an OpenTelemetry Context
- start a span with all the attributes (some set to `Empty`)
- attach the context as parent on the span
- do the processing
- update attributes of the span with response (status,...)

The crates provide helper (or inspiration) to extract/inject context info, start & update span and retrieve context or trace_id during processing (eg to inject trace_id into log, error message,...).

```rust
  let trace_id = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
  //json!({ "error" :  "xxxxxx", "trace_id": trace_id})
```

The helpers could be used as is or into middleware build on it (eg: [`axum-tracing-opentelemetry`], [`tonic-tracing-opentelemetry`] are middlewares build on top of the helpers provide for `http` (feature & crate))

## Notes

- [`tracing-opentelemetry`] extends [`tracing`] to interoperate with [OpenTelemetry]. But with some constraints:
  - Creation of the OpenTelemetry's span is done when the tracing span is closed. So do not try to interact with OpenTelemetry Span (or SpanBuilder) from inside the tracing span.
  - The OpenTelemetry parent Context (and trace_id) is created on `NEW` span or inherited from parent span. The parent context can be overwritten after creation, but until then the `trace_id` is the one from `NEW`, So tracing's log could report none or not-yet set trace_id on event `NEW` and the following until update.
  - To define kind, name,... of OpenTelemetry's span from tracing's span used special record's name: `otel.name`, `otel.kind`, ...
  - Record in a [`tracing`]'s Span should be defined at creation time. So some field are created with value `tracing::field::Empty` to then being updated.
- Create trace with target `otel::tracing` (and level `trace`), to have a common way to enable / to disable

## Instrumentations Tips

Until every crates are instrumented

Use `tracing::instrumented` (no propagation & no update on response)

```txt
// basic handmade span far to be compliant with
//[opentelemetry-specification/.../database.md](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.22.0/specification/trace/semantic_conventions/database.md)
fn make_otel_span(db_operation: &str) -> tracing::Span {
    // NO parsing of statement to extract information, not recommended by Specification and time-consuming
    // warning: providing the statement could leek information
    tracing::trace_span!(
        target: tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET,
        "DB request",
        db.system = "postgresql",
        // db.statement = stmt,
        db.operation = db_operation,
        otel.name = db_operation, // should be <db.operation> <db.name>.<db.sql.table>,
        otel.kind = "CLIENT",
        otel.status_code = tracing::field::Empty,
    )
}


      // Insert or update
        sqlx::query!(
                "INSERT INTO ...",
                id,
                sub_key,
                result,
            )
            .execute(&*self.pool)
            .instrument(make_otel_span("INSERT"))
            .await
            .map_err(...)?;
```

## Related crates

- [`init-tracing-opentelemetry`] to initialize [`tracing`] & [OpenTelemetry]
- [`axum-tracing-opentelemetry`] middlewares for axum based on [`tracing-opentelemetry-instrumentation-sdk`]
- [`tonic-tracing-opentelemetry`] middlewares for tonic based on [`tracing-opentelemetry-instrumentation-sdk`]

[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[OpenTelemetry]: https://crates.io/crates/opentelemetry
[`tracing`]: https://crates.io/crates/tracing
[`axum-tracing-opentelemetry`]: https://crates.io/crates/axum-tracing-opentelemetry
[`init-tracing-opentelemetry`]: https://crates.io/crates/init-tracing-opentelemetry
[`tonic-tracing-opentelemetry`]: https://crates.io/crates/tonic-tracing-opentelemetry
[`tracing-opentelemetry-instrumentation-sdk`]: https://crates.io/crates/tracing-opentelemetry-instrumentation-sdk
