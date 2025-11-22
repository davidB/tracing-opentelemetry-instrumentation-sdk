use fake_opentelemetry_collector::{setup_meter_provider, FakeCollectorServer};
use opentelemetry::{global, KeyValue};
use std::time::Duration;
use tracing::debug;

#[tokio::test(flavor = "multi_thread")]
async fn demo_fake_meter_and_collector() {
    debug!("Start the fake collector");
    let mut fake_collector = FakeCollectorServer::start()
        .await
        .expect("fake collector setup and started");

    debug!("Init the 'application' & meter provider");
    let meter_provider = setup_meter_provider(&fake_collector).await;
    global::set_meter_provider(meter_provider.clone());

    debug!("Run the 'application' & send metrics ...");
    let meter = global::meter("test");
    let attributes = &[KeyValue::new("foo", "bar")];

    let gauge = meter
        .f64_gauge("test_gauge")
        .with_description("A test gauge")
        .with_unit("km/s")
        .build();
    gauge.record(123.456, attributes);

    let up_down_counter = meter
        .i64_up_down_counter("test_updown_counter")
        .with_description("A test up-down-counter")
        .with_unit("m/s^2")
        .build();
    up_down_counter.add(-50, attributes);

    let counter = meter
        .u64_counter("test_counter")
        .with_description("A test counter")
        .with_unit("Jigawatts")
        .build();
    counter.add(25, attributes);

    let histogram = meter
        .u64_histogram("test_histogram")
        .with_description("A test histogram")
        .with_unit("ft/in^2")
        .build();
    histogram.record(10, attributes);
    histogram.record(13, attributes);

    debug!("Shutdown the 'application' & meter provider");
    meter_provider.shutdown().expect("no error during shutdown");
    drop(meter_provider);

    debug!("Collect & check the metrics");
    let otel_metrics = fake_collector
        .exported_metrics(1, Duration::from_millis(500))
        .await;

    insta::assert_yaml_snapshot!(otel_metrics, {
        // Validate gauge metric
        "[0].metrics[0].name" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(name) = value.as_str());
            assert_eq!(name, "test_gauge");
            name.to_string()
        }),
        "[0].metrics[0].description" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(desc) = value.as_str());
            assert_eq!(desc, "A test gauge");
            desc.to_string()
        }),
        "[0].metrics[0].unit" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(unit) = value.as_str());
            assert_eq!(unit, "km/s");
            unit.to_string()
        }),
        "[0].metrics[0].data.Gauge.data_points[0].value.AsDouble" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(val) = value.as_f64());
            assert!((val - 123.456).abs() < 0.001);
            format!("{val}")
        }),

        // Validate up-down counter
        "[0].metrics[1].name" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(name) = value.as_str());
            assert_eq!(name, "test_updown_counter");
            name.to_string()
        }),
        "[0].metrics[1].description" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(desc) = value.as_str());
            assert_eq!(desc, "A test up-down-counter");
            desc.to_string()
        }),
        "[0].metrics[1].unit" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(unit) = value.as_str());
            assert_eq!(unit, "m/s^2");
            unit.to_string()
        }),
        "[0].metrics[1].data.Sum.data_points[0].value.AsInt" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(val) = value.as_i64());
            assert_eq!(val, -50);
            format!("{val}")
        }),
        "[0].metrics[1].data.Sum.is_monotonic" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(monotonic) = value.as_bool());
            assert!(!monotonic);
            format!("{monotonic}")
        }),
        "[0].metrics[1].data.Sum.aggregation_temporality" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(temporality) = value.as_u64());
            assert_eq!(temporality, 2); // Cumulative
            format!("{temporality}")
        }),

        // Validate counter
        "[0].metrics[2].name" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(name) = value.as_str());
            assert_eq!(name, "test_counter");
            name.to_string()
        }),
        "[0].metrics[2].description" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(desc) = value.as_str());
            assert_eq!(desc, "A test counter");
            desc.to_string()
        }),
        "[0].metrics[2].unit" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(unit) = value.as_str());
            assert_eq!(unit, "Jigawatts");
            unit.to_string()
        }),
        "[0].metrics[2].data.Sum.data_points[0].value.AsInt" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(val) = value.as_i64());
            assert_eq!(val, 25);
            format!("{val}")
        }),
        "[0].metrics[2].data.Sum.is_monotonic" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(monotonic) = value.as_bool());
            assert!(monotonic);
            format!("{monotonic}")
        }),
        "[0].metrics[2].data.Sum.aggregation_temporality" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(temporality) = value.as_u64());
            assert_eq!(temporality, 2); // Cumulative
            format!("{temporality}")
        }),

        // Validate histogram
        "[0].metrics[3].name" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(name) = value.as_str());
            assert_eq!(name, "test_histogram");
            name.to_string()
        }),
        "[0].metrics[3].description" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(desc) = value.as_str());
            assert_eq!(desc, "A test histogram");
            desc.to_string()
        }),
        "[0].metrics[3].unit" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(unit) = value.as_str());
            assert_eq!(unit, "ft/in^2");
            unit.to_string()
        }),
        "[0].metrics[3].data.Histogram.data_points[0].count" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(count) = value.as_u64());
            assert_eq!(count, 2);
            format!("{count}")
        }),
        "[0].metrics[3].data.Histogram.data_points[0].sum" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(sum) = value.as_u64());
            assert_eq!(sum, 23); // 10 + 13
            format!("{sum}")
        }),
        "[0].metrics[3].data.Histogram.data_points[0].min" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(min) = value.as_u64());
            assert_eq!(min, 10);
            format!("{min}")
        }),
        "[0].metrics[3].data.Histogram.data_points[0].max" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(max) = value.as_u64());
            assert_eq!(max, 13);
            format!("{max}")
        }),
        "[0].metrics[3].data.Histogram.aggregation_temporality" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(temporality) = value.as_u64());
            assert_eq!(temporality, 2); // Cumulative
            format!("{temporality}")
        }),

        // Validate attributes for all metrics
        "[].metrics[].data.**.attributes.foo" => insta::dynamic_redaction(|value, _path| {
            assert2::let_assert!(Some(attr_value) = value.as_str());
            assert!(attr_value.contains("bar"));
            "\"Some(AnyValue { value: Some(StringValue(\\\"bar\\\")) })\""
        }),

        // Redact timestamps
        "[].metrics[].data.**.start_time_unix_nano" => "[timestamp]",
        "[].metrics[].data.**.time_unix_nano" => "[timestamp]",
        "[].metrics[].data.**.exemplars[].time_unix_nano" => "[timestamp]",
    });
}
