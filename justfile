# Format the code and sort dependencies
format:
    cargo fmt
    cargo sort --workspace --grouped

_check_format:
    cargo fmt --all -- --check
    cargo sort --workspace --grouped --check

deny:
    cargo deny check advisories
    cargo deny check bans licenses sources

# Lint the rust code
lint:
    cargo clippy --workspace --all-features --all-targets -- --deny warnings

# Launch tests
test:
    cargo nextest run
    cargo test --doc

run_jaeger:
  podman run --rm --name jaeger \
    -e COLLECTOR_ZIPKIN_HOST_PORT:9411 \
    -e COLLECTOR_OTLP_ENABLED:true \
    -p 6831:6831/udp \
    -p 6832:6832/udp \
    -p 5778:5778 \
    -p 16686:16686 \
    -p 4317:4317 \
    -p 4318:4318 \
    -p 14250:14250 \
    -p 14268:14268 \
    -p 14269:14269 \
    -p 9411:9411 \
    jaegertracing/all-in-one:latest

  # echo "open http://localhost:16686"

run_example_grpc_server:
  cd examples/grpc
  cargo run --bin server

run_example_grpc_client:
  cd examples/grpc
  grpcurl -plaintext 127.0.0.1:50051 list
  # grpcurl -plaintext  -d '{"service": "healthcheck"}' 127.0.0.1:50051 grpc.health.v1.Health/Check
  grpc-health-probe -addr 127.0.0.1:50051
  cargo run --bin client
