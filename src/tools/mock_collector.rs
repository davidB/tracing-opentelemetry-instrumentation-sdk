//! based on https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/tests/smoke.rs
use futures::StreamExt;
use opentelemetry_api::global::shutdown_tracer_provider;
use opentelemetry_api::trace::{Span, SpanKind, Tracer};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use std::sync::mpsc;
use std::{net::SocketAddr, sync::Mutex};
use tokio_stream::wrappers::TcpListenerStream;

pub type ExportedSpan = opentelemetry_proto::tonic::trace::v1::Span;

struct MockServer {
    tx: Mutex<mpsc::SyncSender<ExportedSpan>>,
}

impl MockServer {
    pub fn new(tx: mpsc::SyncSender<ExportedSpan>) -> Self {
        Self { tx: Mutex::new(tx) }
    }
}

#[tonic::async_trait]
impl TraceService for MockServer {
    async fn export(
        &self,
        request: tonic::Request<ExportTraceServiceRequest>,
    ) -> Result<tonic::Response<ExportTraceServiceResponse>, tonic::Status> {
        println!("Sending request into channel...");
        // assert we have required metadata key
        assert_eq!(
            request.metadata().get("x-header-key"),
            Some(&("header-value".parse().unwrap()))
        );
        request
            .into_inner()
            .resource_spans
            .into_iter()
            .flat_map(|rs| rs.scope_spans)
            .flat_map(|ss| ss.spans)
            //.map(ExportedSpan::from)
            .for_each(|es| {
                self.tx.lock().unwrap().send(es).expect("Channel full");
            });
        Ok(tonic::Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

async fn setup() -> (SocketAddr, mpsc::Receiver<ExportedSpan>) {
    let addr: SocketAddr = "[::1]:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    let addr = listener.local_addr().unwrap();
    let stream = TcpListenerStream::new(listener).map(|s| {
        if let Ok(ref s) = s {
            println!("Got new conn at {}", s.peer_addr().unwrap());
        }
        s
    });

    let (req_tx, req_rx) = mpsc::sync_channel::<ExportedSpan>(1024);
    let service = TraceServiceServer::new(MockServer::new(req_tx));
    tokio::task::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(service)
            .serve_with_incoming(stream)
            .await
            .expect("Server failed");
    });
    (addr, req_rx)
}

pub async fn setup_tracer() -> (
    opentelemetry::sdk::trace::Tracer,
    mpsc::Receiver<ExportedSpan>,
) {
    let (addr, req_rx) = setup().await;

    let mut metadata = tonic::metadata::MetadataMap::new();
    metadata.insert("x-header-key", "header-value".parse().unwrap());
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(format!("http://{}", addr)) //Devskim: ignore DS137138
                .with_metadata(metadata),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("failed to install");
    (tracer, req_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mock_tracer_and_collector() {
        let (tracer, mut req_rx) = setup_tracer().await;

        println!("Sending span...");
        let mut span = tracer
            .span_builder("my-test-span")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.add_event("my-test-event", vec![]);
        span.end();

        shutdown_tracer_provider();

        //let first_span = req_rx.recv().expect("missing export request");
        let otel_spans = std::iter::from_fn(|| req_rx.try_recv().ok()).collect::<Vec<_>>();
        insta::assert_debug_snapshot!(otel_spans);
        //assert_eq!("my-test-span", first_span.name);
    }
}
