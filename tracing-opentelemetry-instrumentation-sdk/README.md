# tracing-opentelemetry-instrumentation-sdk

Provide a set of helpers to build [OpenTelemetry] instrumentation based on [`tracing`] crate.

PS: Contributions are welcome (bug report, improvements, features, ...)

## Notes

[`tracing-opentelemetry`] extends [`tracing`] to interoperate with [OpenTelemetry]. But with some constraints:

- Creation of the OpenTelemetry's span is done when the tracing span is closed. So do not try to interact with OpenTelemetry Span (or SpanBuilder) from inside the tracing span.
- The OpenTelemetry parent Context (and trace_id) is created on `NEW` span or inherited from parent span. The parent context can be overwritten after creation, but until then the `trace_id` is the one from `NEW`, So tracing's log could report none or not-yet set trace_id on event `NEW` and the following until update.
- To define kind, name,... of OpenTelemetry's span from tracing's span used special record's name: `otel.name`, `otel.kind`, ...

Record in a [`tracing`]'s Span should be defined at creation time. So some field are created with value `tracing::field::Empty` to then being updated.

[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[OpenTelemetry]: https://crates.io/crates/opentelemetry
[`tracing`]: https://crates.io/crates/tracing
