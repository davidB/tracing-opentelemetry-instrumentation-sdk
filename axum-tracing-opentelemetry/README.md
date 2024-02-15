# axum-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/axum-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/axum-tracing-opentelemetry.svg)](https://crates.io/crates/axum-tracing-opentelemetry)

[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Middlewares to integrate axum + tracing + opentelemetry.

- Read OpenTelemetry header from incoming request
- Start a new trace if no trace found in the incoming request
- Trace is attached into tracing'span
- OpenTelemetry Span is created on close of the tracing's span (behavior from [tracing-opentelemetry])

For examples, you can look at the [examples](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/tree/main/examples/) folder.

```txt
//...
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;

#[tokio::main]
async fn main() -> Result<(), axum::BoxError> {
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    let app = app();
    // run it
    let addr = &"0.0.0.0:3000".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service())
        //FIXME .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    Router::new()
        .route("/", get(index)) // request processed inside span
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .route("/health", get(health)) // request processed without span / trace
}

async fn shutdown_signal() {
    //...
    opentelemetry::global::shutdown_tracer_provider();
}
```

For more info about how to initialize, you can look at crate [`init-tracing-opentelemetry`] or [`tracing-opentelemetry`].

## Changelog - History

[CHANGELOG.md](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/main/CHANGELOG.md)

[`tracing-opentelemetry`]: https://crates.io/crates/tracing-opentelemetry
[`init-tracing-opentelemetry`]: https://crates.io/crates/init-tracing-opentelemetry
