mod common;
mod logs;
mod trace;
pub use logs::ExportedLog;
pub use trace::ExportedSpan;

use logs::*;
use trace::*;

use std::net::SocketAddr;

use futures::StreamExt;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_proto::tonic::collector::logs::v1::logs_service_server::LogsServiceServer;
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::TraceServiceServer;
use std::sync::mpsc;
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

        let (req_tx, req_rx) = mpsc::sync_channel::<ExportedSpan>(1024);
        let (log_tx, log_rx) = mpsc::sync_channel::<ExportedLog>(1024);
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

    pub fn exported_spans(&self) -> Vec<ExportedSpan> {
        std::iter::from_fn(|| self.req_rx.try_recv().ok()).collect::<Vec<_>>()
    }

    pub fn exported_logs(&self) -> Vec<ExportedLog> {
        std::iter::from_fn(|| self.log_rx.try_recv().ok()).collect::<Vec<_>>()
    }

    pub fn abort(self) {
        self.handle.abort()
    }
}

pub async fn setup_tracer(fake_server: &FakeCollectorServer) -> opentelemetry_sdk::trace::Tracer {
    // if the environment variable is set (in test or in caller), `with_endpoint` value is ignored
    std::env::remove_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT");
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(fake_server.endpoint()),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("failed to install tracer")
}

pub async fn setup_logger(
    fake_server: &FakeCollectorServer,
) -> opentelemetry_sdk::logs::LoggerProvider {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(fake_server.endpoint()),
        )
        .install_simple() //Install simple so we don't have to wait for batching in tests
        .expect("failed to install logging")
}

#[cfg(test)]
mod tests {
    use super::*;

    use opentelemetry::global::shutdown_tracer_provider;
    use opentelemetry::logs::{LogRecord, Logger, LoggerProvider, Severity};
    use opentelemetry::trace::{Span, SpanKind, Tracer};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fake_tracer_and_collector() {
        let fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector setup and started");
        let tracer = setup_tracer(&fake_collector).await;

        debug!("Sending span...");
        let mut span = tracer
            .span_builder("my-test-span")
            .with_kind(SpanKind::Server)
            .start(&tracer);
        span.add_event("my-test-event", vec![]);
        span.end();
        shutdown_tracer_provider();

        let otel_spans = fake_collector.exported_spans();
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

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fake_logger_and_collector() {
        let fake_collector = FakeCollectorServer::start()
            .await
            .expect("fake collector setup and started");

        let logger_provider = setup_logger(&fake_collector).await;
        let logger = logger_provider.logger("test");
        let mut record = logger.create_log_record();
        record.set_body("This is information".into());
        record.set_severity_number(Severity::Info);
        record.set_severity_text("info".into());
        logger.emit(record);

        let otel_logs = fake_collector.exported_logs();
        println!("otel_logs {:?}", otel_logs);

        insta::assert_yaml_snapshot!(otel_logs, {
            "[].trace_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(trace_id) = value.as_str());
                format!("[trace_id:lg{}]", trace_id.len())
            }),
            "[].span_id" => insta::dynamic_redaction(|value, _path| {
                assert2::let_assert!(Some(span_id) = value.as_str());
                format!("[span_id:lg{}]", span_id.len())
            }),
            "[].observed_time_unix_nano" => "[timestamp]",
            "[].severity_number" => 9,
            "[].severity_text" => "info",
            "[].body" => "AnyValue { value: Some(StringValue(\"This is information\")) }",
        });
    }
}
