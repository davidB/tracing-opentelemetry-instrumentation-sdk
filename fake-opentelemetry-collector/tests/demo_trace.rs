use std::time::Duration;

use fake_opentelemetry_collector::{setup_tracer_provider, FakeCollectorServer};
use opentelemetry::trace::TracerProvider;
use opentelemetry::trace::{Span, SpanKind, Tracer};
use tracing::debug;

#[tokio::test(flavor = "multi_thread")]
async fn demo_fake_tracer_and_collector() {
    debug!("Start the fake collector");
    let mut fake_collector = FakeCollectorServer::start()
        .await
        .expect("fake collector setup and started");

    debug!("Init the 'application' & tracer provider");
    let tracer_provider = setup_tracer_provider(&fake_collector).await;
    let tracer = tracer_provider.tracer("test");

    debug!("Run the 'application' & sending span...");
    let mut span = tracer
        .span_builder("my-test-span")
        .with_kind(SpanKind::Server)
        .start(&tracer);
    span.add_event("my-test-event", vec![]);
    span.end();

    debug!("Shutdown the 'application' & tracer provider and force flush the spans");
    let _ = tracer_provider.force_flush();
    tracer_provider
        .shutdown()
        .expect("no error during shutdown");
    drop(tracer_provider);

    debug!("Collect & check the spans");
    let otel_spans = fake_collector
        .exported_spans(1, Duration::from_secs(20))
        .await;
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
