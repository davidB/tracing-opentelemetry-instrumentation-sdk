# tonic-tracing-opentelemetry

[![crates license](https://img.shields.io/crates/l/tonic-tracing-opentelemetry.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/tonic-tracing-opentelemetry.svg)](https://crates.io/crates/tonic-tracing-opentelemetry)

[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Middlewares and tools to integrate tonic + tracing + opentelemetry for client and server.

> Really early, missing lot of features, help is welcomed.

- Read OpenTelemetry header from the incoming requests
- Start a new trace if no trace is found in the incoming request
- Trace is attached into tracing's span

For examples, you can look at the [examples](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/tree/main/examples/) folder.

Extract of `client.rs`:

```txt
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?; //Devskim: ignore DS137138
    let channel = ServiceBuilder::new()
        .layer(OtelGrpcLayer::default())
        .service(channel);

    let mut client = GreeterClient::new(channel);

    //...

    opentelemetry::global::shutdown_tracer_provider();
```

Extract of `server.rs`:

```txt
    Server::builder()
        // create trace for every request including health_service
        .layer(server::OtelGrpcLayer::default().filter(filters::reject_healthcheck))
        .add_service(health_service)
        .add_service(reflection_service)
        //.add_service(GreeterServer::new(greeter))
        .add_service(GreeterServer::new(greeter))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
```

## TODO

- add test
- add documentation
- add examples
- validate with [[opentelemetry-specification/rpc.md at main · open-telemetry/opentelemetry-specification · GitHub](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/semantic_conventions/rpc.md#grpc)]

## Changelog - History

[CHANGELOG.md](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/main/CHANGELOG.md)
