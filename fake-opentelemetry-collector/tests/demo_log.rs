use std::time::Duration;

use fake_opentelemetry_collector::{setup_logger_provider, FakeCollectorServer};
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider, Severity};
use tracing::debug;

#[tokio::test(flavor = "multi_thread")]
async fn demo_fake_logger_and_collector() {
    debug!("Start the fake collector");
    let mut fake_collector = FakeCollectorServer::start()
        .await
        .expect("fake collector setup and started");

    debug!("Init the 'application' & logger provider");
    let logger_provider = setup_logger_provider(&fake_collector).await;
    let logger = logger_provider.logger("test");

    debug!("Run the 'application' & send log ...");
    let mut record = logger.create_log_record();
    record.set_body("This is information".into());
    record.set_severity_number(Severity::Info);
    record.set_severity_text("info");
    logger.emit(record);

    debug!("Shutdown the 'application' & logger provider");
    let _ = logger_provider.force_flush();
    logger_provider
        .shutdown()
        .expect("no error during shutdown");
    drop(logger_provider);

    debug!("Collect & check the logs");
    let otel_logs = fake_collector
        .exported_logs(1, Duration::from_millis(500))
        .await;

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
