use assert2::{check, let_assert};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use serde_json::Value;
use std::sync::mpsc::{self, Receiver, SyncSender};

use tracing_subscriber::{
    fmt::{format::FmtSpan, MakeWriter},
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn assert_trace(
    name: &str,
    tracing_events: Vec<Value>,
    otel_spans: Vec<fake_opentelemetry_collector::ExportedSpan>,
    is_trace_id_constant: bool,
) {
    let trace_id_0 = tracing_events
        .get(0)
        .and_then(|v| v.as_object())
        .and_then(|v| v.get("span"))
        .and_then(|v| v.as_object())
        .and_then(|v| v.get("trace_id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned();
    // let trace_id_3 = trace_id_0.clone();
    let trace_id_1 = trace_id_0.clone();
    let trace_id_2 = trace_id_0;
    insta::assert_yaml_snapshot!(name, tracing_events, {
        "[].timestamp" => "[timestamp]",
        "[].fields[\"time.busy\"]" => "[duration]",
        "[].fields[\"time.idle\"]" => "[duration]",
        "[].span.trace_id" => insta::dynamic_redaction(move |value, _path| {
            let_assert!(Some(tracing_trace_id) = value.as_str());
            check!(trace_id_1 == tracing_trace_id);
            if is_trace_id_constant {
                tracing_trace_id.to_string()
            } else {
                format!("[trace_id:lg{}]", tracing_trace_id.len())
            }
        }),
        "[].spans[].trace_id" => insta::dynamic_redaction(move |value, _path| {
            let_assert!(Some(tracing_trace_id) = value.as_str());
            check!(trace_id_2 == tracing_trace_id);
            if is_trace_id_constant {
                tracing_trace_id.to_string()
            } else {
                format!("[trace_id:lg{}]", tracing_trace_id.len())
            }
        }),
    });
    insta::assert_yaml_snapshot!(format!("{}_otel_spans", name), otel_spans, {
        "[].start_time_unix_nano" => "[timestamp]",
        "[].end_time_unix_nano" => "[timestamp]",
        "[].events[].time_unix_nano" => "[timestamp]",
        "[].trace_id" => insta::dynamic_redaction(move |value, _path| {
            let_assert!(Some(otel_trace_id) = value.as_str());
            //FIXME check!(trace_id_3 == otel_trace_id);
            format!("[trace_id:lg{}]", otel_trace_id.len())
        }),
        "[].span_id" => insta::dynamic_redaction(|value, _path| {
            let_assert!(Some(span_id) = value.as_str());
            format!("[span_id:lg{}]", span_id.len())
        }),
        "[].parent_span_id" => insta::dynamic_redaction(|value, _path| {
            let_assert!(Some(span_id) = value.as_str());
            format!("[span_id:lg{}]", span_id.len())
        }),
        "[].links[].trace_id" => insta::dynamic_redaction(|value, _path| {
            let_assert!(Some(otel_trace_id) = value.as_str());
            format!("[trace_id:lg{}]", otel_trace_id.len())
        }),
        "[].links[].span_id" => insta::dynamic_redaction(|value, _path| {
            let_assert!(Some(span_id) = value.as_str());
            format!("[span_id:lg{}]", span_id.len())
        }),
        "[].attributes.busy_ns" => "ignore",
        "[].attributes.idle_ns" => "ignore",
        "[].attributes.trace_id" => "ignore",
        "[].attributes[\"code.lineno\"]" => "ignore",
        "[].attributes[\"code.filepath\"]" => "ignore",
        "[].attributes[\"thread.id\"]" => "ignore",
    });
}

pub struct FakeEnvironment {
    fake_collector: fake_opentelemetry_collector::FakeCollectorServer,
    rx: Receiver<Vec<u8>>,
    _subsciber_guard: tracing::subscriber::DefaultGuard,
}

impl FakeEnvironment {
    pub async fn setup() -> Self {
        //use axum::body::HttpBody as _;
        //use tower::{Service, ServiceExt};
        use tracing_subscriber::layer::SubscriberExt;

        // setup a non Noop OpenTelemetry tracer to have non-empty trace_id
        let fake_collector = fake_opentelemetry_collector::FakeCollectorServer::start()
            .await
            .unwrap();
        let tracer = fake_opentelemetry_collector::setup_tracer(&fake_collector).await;
        //let (tracer, mut req_rx) = fake_opentelemetry_collector::setup_tracer().await;
        opentelemetry_api::global::set_text_map_propagator(TraceContextPropagator::new());
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let (make_writer, rx) = duplex_writer();
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(make_writer)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::try_new("trace").unwrap())
            .with(fmt_layer)
            .with(otel_layer);
        let _subsciber_guard = subscriber.set_default();
        Self {
            fake_collector,
            rx,
            _subsciber_guard,
        }
    }

    pub async fn collect_traces(
        self,
    ) -> (Vec<Value>, Vec<fake_opentelemetry_collector::ExportedSpan>) {
        opentelemetry_api::global::shutdown_tracer_provider();

        let otel_span = self.fake_collector.exported_spans();
        // insta::assert_debug_snapshot!(first_span);
        let tracing_events = std::iter::from_fn(|| self.rx.try_recv().ok())
            .map(|bytes| serde_json::from_slice::<Value>(&bytes).unwrap())
            .collect::<Vec<_>>();
        (tracing_events, otel_span)
    }
}

fn duplex_writer() -> (DuplexWriter, Receiver<Vec<u8>>) {
    let (tx, rx) = mpsc::sync_channel(1024);
    (DuplexWriter { tx }, rx)
}

#[derive(Clone)]
struct DuplexWriter {
    tx: SyncSender<Vec<u8>>,
}

impl<'a> MakeWriter<'a> for DuplexWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl std::io::Write for DuplexWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tx.send(buf.to_vec()).unwrap();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
