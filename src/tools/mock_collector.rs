//! based on https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/tests/smoke.rs
use futures::StreamExt;
//use opentelemetry::{KeyValue, Value};
use opentelemetry_api::global::shutdown_tracer_provider;
use opentelemetry_api::trace::{Span, SpanKind, Tracer};
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use serde::Serialize;
use std::sync::mpsc;
use std::{net::SocketAddr, sync::Mutex};
use tokio_stream::wrappers::TcpListenerStream;
use tracing::debug;

//pub type ExportedSpan = opentelemetry_proto::tonic::trace::v1::Span;

/// opentelemetry_proto::tonic::trace::v1::Span is no compatible with serde::Serialize
/// and to be able to test with insta,... it's needed (Debug is not enough to be able to filter unstable value,...)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
pub struct ExportedSpan {
    pub trace_id: String,
    pub span_id: String,
    pub trace_state: String,
    pub parent_span_id: String,
    pub name: String,
    pub kind: String, //SpanKind,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: Vec<(String, String)>,
    pub dropped_attributes_count: u32,
    pub events: Vec<Event>,
    pub dropped_events_count: u32,
    pub links: Vec<Link>,
    pub dropped_links_count: u32,
    pub status: Option<Status>,
}

impl From<opentelemetry_proto::tonic::trace::v1::Span> for ExportedSpan {
    fn from(value: opentelemetry_proto::tonic::trace::v1::Span) -> Self {
        Self {
            trace_id: hex::encode(&value.trace_id),
            span_id: hex::encode(&value.span_id),
            trace_state: value.trace_state.clone(),
            parent_span_id: hex::encode(&value.parent_span_id),
            name: value.name.clone(),
            kind: value.kind().as_str_name().to_owned(),
            start_time_unix_nano: value.start_time_unix_nano,
            end_time_unix_nano: value.end_time_unix_nano,
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
            events: value.events.iter().map(Event::from).collect(),
            dropped_events_count: value.dropped_events_count,
            links: value.links.iter().map(Link::from).collect(),
            dropped_links_count: value.dropped_links_count,
            status: value.status.map(Status::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
pub struct Status {
    message: String,
    code: String,
}

impl From<opentelemetry_proto::tonic::trace::v1::Status> for Status {
    fn from(value: opentelemetry_proto::tonic::trace::v1::Status) -> Self {
        Self {
            message: value.message.clone(),
            code: value.code().as_str_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
pub struct Link {
    pub trace_id: String,
    pub span_id: String,
    pub trace_state: String,
    pub attributes: Vec<(String, String)>,
    pub dropped_attributes_count: u32,
}

impl From<&opentelemetry_proto::tonic::trace::v1::span::Link> for Link {
    fn from(value: &opentelemetry_proto::tonic::trace::v1::span::Link) -> Self {
        Self {
            trace_id: hex::encode(&value.trace_id),
            span_id: hex::encode(&value.span_id),
            trace_state: value.trace_state.clone(),
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
        }
    }
}

fn cnv_attributes(
    attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue],
) -> Vec<(String, String)> {
    let mut v = attributes
        .iter()
        .map(|kv| (kv.key.to_string(), format!("{:?}", kv.value)))
        .collect::<Vec<(String, String)>>();
    v.sort_by_key(|kv| kv.0.clone());
    v
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
pub struct Event {
    time_unix_nano: u64,
    name: String,
    attributes: Vec<(String, String)>,
    dropped_attributes_count: u32,
}

impl From<&opentelemetry_proto::tonic::trace::v1::span::Event> for Event {
    fn from(value: &opentelemetry_proto::tonic::trace::v1::span::Event) -> Self {
        Self {
            time_unix_nano: value.time_unix_nano,
            name: value.name.clone(),
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
        }
    }
}

struct MockTraceService {
    tx: Mutex<mpsc::SyncSender<ExportedSpan>>,
}

impl MockTraceService {
    pub fn new(tx: mpsc::SyncSender<ExportedSpan>) -> Self {
        Self { tx: Mutex::new(tx) }
    }
}

#[tonic::async_trait]
impl TraceService for MockTraceService {
    async fn export(
        &self,
        request: tonic::Request<ExportTraceServiceRequest>,
    ) -> Result<tonic::Response<ExportTraceServiceResponse>, tonic::Status> {
        debug!("Sending request into channel...");
        request
            .into_inner()
            .resource_spans
            .into_iter()
            .flat_map(|rs| rs.scope_spans)
            .flat_map(|ss| ss.spans)
            .map(ExportedSpan::from)
            .for_each(|es| {
                self.tx.lock().unwrap().send(es).expect("Channel full");
            });
        Ok(tonic::Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

pub struct MockCollectorServer {
    address: SocketAddr,
    req_rx: mpsc::Receiver<ExportedSpan>,
    handle: tokio::task::JoinHandle<()>,
}

impl MockCollectorServer {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let addr = listener.local_addr()?;
        let stream = TcpListenerStream::new(listener).map(|s| {
            if let Ok(ref s) = s {
                debug!("Got new conn at {}", s.peer_addr()?);
            }
            s
        });

        let (req_tx, req_rx) = mpsc::sync_channel::<ExportedSpan>(1024);
        let service = TraceServiceServer::new(MockTraceService::new(req_tx));
        let handle = tokio::task::spawn(async move {
            debug!("start MockCollectorServer http://{addr}"); //Devskim: ignore DS137138)
            tonic::transport::Server::builder()
                .add_service(service)
                .serve_with_incoming(stream)
                // .serve(addr)
                .await
                .expect("Server failed");
            debug!("stop MockCollectorServer");
        });
        Ok(Self {
            address: addr,
            req_rx,
            handle,
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn endpoint(&self) -> String {
        format!("http://{}", self.address()) //Devskim: ignore DS137138)
    }

    pub fn exported_spans(&self) -> Vec<ExportedSpan> {
        std::iter::from_fn(|| self.req_rx.try_recv().ok()).collect::<Vec<_>>()
    }

    pub fn abort(self) {
        self.handle.abort()
    }
}

pub async fn setup_tracer(mock_server: &MockCollectorServer) -> opentelemetry::sdk::trace::Tracer {
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", mock_server.endpoint());
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter().tonic(),
            // if the environment variable is set (in test or in caller), this value is ignored
            // .with_endpoint(mock_server.endpoint()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("failed to install")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_tracer_and_collector() {
        let mock_collector = MockCollectorServer::start()
            .await
            .expect("mock collector setup and started");
        let tracer = setup_tracer(&mock_collector).await;

        debug!("Sending span...");
        let mut span = tracer
            .span_builder("my-test-span")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.add_event("my-test-event", vec![]);
        span.end();

        shutdown_tracer_provider();

        let otel_spans = mock_collector.exported_spans();
        //insta::assert_debug_snapshot!(otel_spans);
        insta::assert_yaml_snapshot!(otel_spans, {
            "[].start_time_unix_nano" => "[timestamp]",
            "[].end_time_unix_nano" => "[timestamp]",
            "[].events[].time_unix_nano" => "[timestamp]",
            "[].trace_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(trace_id) = value.as_str());
                format!("[trace_id:lg{}]", trace_id.len())
            }),
            "[].span_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(span_id) = value.as_str());
                format!("[span_id:lg{}]", span_id.len())
            }),
            "[].links[].trace_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(trace_id) = value.as_str());
                format!("[trace_id:lg{}]", trace_id.len())
            }),
            "[].links[].span_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(span_id) = value.as_str());
                format!("[span_id:lg{}]", span_id.len())
            }),
        });
    }
}
