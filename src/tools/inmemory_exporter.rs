use futures::future::BoxFuture;
use opentelemetry::sdk::export::trace::{ExportResult, SpanData, SpanExporter};
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::trace::SpanId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

/// An exporter for jaeger comptible json files containing trace data
#[derive(Debug)]
pub struct InMemoryJsonExporter {
    service_name: String,
    spans: Arc<Vec<serde_json::Value>>,
}

impl InMemoryJsonExporter {
    /// Configure a new jaeger-json exporter
    ///
    /// * `service_name` is used to identify the corresponding service in jaeger
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_owned(),
            spans: Arc::new(vec![]),
        }
    }

    /// Install the exporter using the internal provided runtime
    pub fn install_batch(self) -> Tracer {
        use opentelemetry::trace::TracerProvider;

        let provider_builder =
            opentelemetry::sdk::trace::TracerProvider::builder().with_simple_exporter(self);

        let provider = provider_builder.build();

        let tracer =
            provider.versioned_tracer("opentelemetry", Some(env!("CARGO_PKG_VERSION")), None);
        let _ = opentelemetry::global::set_tracer_provider(provider);

        tracer
    }

    pub fn get_exported(&self) -> Arc<Vec<serde_json::Value>> {
        self.spans.clone()
    }
}

impl SpanExporter for InMemoryJsonExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        let mut trace_map = HashMap::new();

        for span in batch {
            let ctx = &span.span_context;
            trace_map
                .entry(ctx.trace_id())
                .or_insert_with(Vec::new)
                .push(span_data_to_jaeger_json(span));
        }

        let data = trace_map
            .into_iter()
            .map(|(trace_id, spans)| {
                serde_json::json!({
                    "traceID": trace_id.to_string(),
                    "spans": spans,
                    "processes": {
                        "p1": {
                            "serviceName": self.service_name,
                            "tags": []
                        }
                    }
                })
            })
            .collect::<Vec<_>>();

        let json = serde_json::json!({
            "data": data,
        });

        self.spans.push(json);
        Box::pin(std::future::ready(Ok(())))
    }
}

fn span_data_to_jaeger_json(
    span: opentelemetry::sdk::export::trace::SpanData,
) -> serde_json::Value {
    let events = span
        .events
        .iter()
        .map(|e| {
            let mut fields = e
                .attributes
                .iter()
                .map(|a| {
                    let (tpe, value) = opentelemetry_value_to_json(&a.value);
                    serde_json::json!({
                        "key": a.key.as_str(),
                        "type": tpe,
                        "value": value,
                    })
                })
                .collect::<Vec<_>>();
            fields.push(serde_json::json!({
                "key": "event",
                "type": "string",
                "value": e.name,
            }));

            serde_json::json!({
                "timestamp": e.timestamp.duration_since(SystemTime::UNIX_EPOCH).expect("This does not fail").as_micros() as i64,
                "fields": fields,
            })
        })
        .collect::<Vec<_>>();
    let tags = span
        .attributes
        .iter()
        .map(|(key, value)| {
            let (tpe, value) = opentelemetry_value_to_json(value);
            serde_json::json!({
            "key": key.as_str(),
            "type": tpe,
            "value": value,
            })
        })
        .collect::<Vec<_>>();
    let mut references = if span.links.is_empty() {
        None
    } else {
        Some(
            span.links
                .iter()
                .map(|link| {
                    let span_context = &link.span_context;
                    serde_json::json!({
                        "refType": "FOLLOWS_FROM",
                        "traceID": span_context.trace_id().to_string(),
                        "spanID": span_context.span_id().to_string(),
                    })
                })
                .collect::<Vec<_>>(),
        )
    };
    if span.parent_span_id != SpanId::INVALID {
        let val = serde_json::json!({
            "refType": "CHILD_OF",
            "traceID": span.span_context.trace_id().to_string(),
            "spanID": span.parent_span_id.to_string(),
        });
        references.get_or_insert_with(Vec::new).push(val);
    }
    serde_json::json!({
        "traceID": span.span_context.trace_id().to_string(),
        "spanID": span.span_context.span_id().to_string(),
        "startTime": span.start_time.duration_since(SystemTime::UNIX_EPOCH).expect("This does not fail").as_micros() as i64,
        "duration": span.end_time.duration_since(span.start_time).expect("This does not fail").as_micros() as i64,
        "operationName": span.name,
        "tags": tags,
        "logs": events,
        "flags": span.span_context.trace_flags().to_u8(),
        "processID": "p1",
        "warnings": None::<String>,
        "references": references,
    })
}

fn opentelemetry_value_to_json(value: &opentelemetry::Value) -> (&str, serde_json::Value) {
    match value {
        opentelemetry::Value::Bool(b) => ("bool", serde_json::json!(b)),
        opentelemetry::Value::I64(i) => ("int64", serde_json::json!(i)),
        opentelemetry::Value::F64(f) => ("float64", serde_json::json!(f)),
        opentelemetry::Value::String(s) => ("string", serde_json::json!(s.as_str())),
        v @ opentelemetry::Value::Array(_) => ("string", serde_json::json!(v.to_string())),
    }
}
