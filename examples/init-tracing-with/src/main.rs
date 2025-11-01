use init_tracing_opentelemetry::TracingConfig;
use tokio_blocked::TokioBlockedLayer;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() {
    let blocked = TokioBlockedLayer::new()
        .with_warn_busy_single_poll(Some(std::time::Duration::from_micros(150)));

    let _guard = TracingConfig::default()
        .with_log_directives("info,tokio::task=trace,tokio::task::waker=warn")
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init_subscriber_ext(|subscriber| subscriber.with(blocked))
        .unwrap();

    info!("will block in 1 secs");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    tokio::task::spawn(async {
        // BAD!
        // This produces a warning log message.
        info!("blocking!");
        std::thread::sleep(std::time::Duration::from_secs(1));
    })
    .await
    .unwrap();

    // sleep().await;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}

// #[tracing::instrument]
// async fn sleep() {
//     tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
// }
