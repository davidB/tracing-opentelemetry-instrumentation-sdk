#[tracing::instrument]
async fn log() {
    tracing::error!("This is ground control to Major Tom");
    tracing::warn!("Houston, we have a problem");
    tracing::info!("We have contact");
    tracing::debug!("Roger, copy that");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
}

#[tracing::instrument]
async fn calc(a: i32, b: i32) {
    let result = a + b;
    tracing::info!(result, "calculated result");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up tracing
    let _guard = init_tracing_opentelemetry::TracingConfig::production().init_subscriber()?;

    log().await;
    calc(1, 2).await;

    Ok(())
}
