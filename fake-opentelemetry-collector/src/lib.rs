mod common;
mod logs;
mod trace;
pub use logs::ExportedLog;
pub use trace::ExportedSpan;

use logs::*;
use trace::*;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use futures::StreamExt;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig};
use opentelemetry_proto::tonic::collector::logs::v1::logs_service_server::LogsServiceServer;
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::TraceServiceServer;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::TcpListenerStream;
use tracing::debug;

pub struct FakeCollectorServer {
    address: SocketAddr,
    req_rx: mpsc::Receiver<ExportedSpan>,
    log_rx: mpsc::Receiver<ExportedLog>,
    handle: tokio::task::JoinHandle<()>,
}

impl FakeCollectorServer {
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

        let (req_tx, req_rx) = mpsc::channel::<ExportedSpan>(64);
        let (log_tx, log_rx) = mpsc::channel::<ExportedLog>(64);
        let trace_service = TraceServiceServer::new(FakeTraceService::new(req_tx));
        let logs_service = LogsServiceServer::new(FakeLogsService::new(log_tx));
        let handle = tokio::task::spawn(async move {
            debug!("start FakeCollectorServer http://{addr}"); //Devskim: ignore DS137138)
            tonic::transport::Server::builder()
                .add_service(trace_service)
                .add_service(logs_service)
                .serve_with_incoming(stream)
                .await
                .expect("Server failed");
            debug!("stop FakeCollectorServer");
        });
        Ok(Self {
            address: addr,
            req_rx,
            log_rx,
            handle,
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn endpoint(&self) -> String {
        format!("http://{}", self.address()) //Devskim: ignore DS137138)
    }

    pub async fn exported_spans(
        &mut self,
        at_least: usize,
        timeout: Duration,
    ) -> Vec<ExportedSpan> {
        recv_many(&mut self.req_rx, at_least, timeout).await
    }

    pub async fn exported_logs(&mut self, at_least: usize, timeout: Duration) -> Vec<ExportedLog> {
        recv_many(&mut self.log_rx, at_least, timeout).await
    }

    pub fn abort(self) {
        self.handle.abort()
    }
}

async fn recv_many<T>(rx: &mut Receiver<T>, at_least: usize, timeout: Duration) -> Vec<T> {
    let deadline = Instant::now();
    let pause = (timeout / 10).min(Duration::from_millis(10));
    while rx.len() < at_least && deadline.elapsed() < timeout {
        tokio::time::sleep(pause).await;
    }
    std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>()
}

pub async fn setup_tracer_provider(
    fake_server: &FakeCollectorServer,
) -> opentelemetry_sdk::trace::TracerProvider {
    // if the environment variable is set (in test or in caller), `with_endpoint` value is ignored
    std::env::remove_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");

    opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(
            SpanExporter::builder()
                .with_tonic()
                .with_endpoint(fake_server.endpoint())
                .build()
                .expect("failed to install tracer"),
            opentelemetry_sdk::runtime::Tokio,
        )
        .build()
}

pub async fn setup_logger_provider(
    fake_server: &FakeCollectorServer,
) -> opentelemetry_sdk::logs::LoggerProvider {
    opentelemetry_sdk::logs::LoggerProvider::builder()
        //Install simple so we don't have to wait for batching in tests
        .with_simple_exporter(
            LogExporter::builder()
                .with_tonic()
                .with_endpoint(fake_server.endpoint())
                .build()
                .expect("failed to install logging"),
        )
        .build()
}
