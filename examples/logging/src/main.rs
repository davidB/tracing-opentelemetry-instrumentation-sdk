#[tracing::instrument]
async fn calc(a: i32, b: i32) {
    let result = a + b;
    tracing::info!(result, "calculated result");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setting up tracing
    let _guard = init_tracing_opentelemetry::TracingConfig::production().init_subscriber()?;

    calc(1, 2).await;

    Ok(())
}
