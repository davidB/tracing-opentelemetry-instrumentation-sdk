# fake-opentelemetry-collector

A Fake (basic) opentelemetry collector, useful to test what is collected opentelemetry

Usage example with [insta](https://crates.io/crates/insta) (snapshot testing)

```rust
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
```
