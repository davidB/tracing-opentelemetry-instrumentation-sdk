_install_cargo-binstall:
    cargo install cargo-binstall

_install_cargo-nextest: _install_cargo-binstall
    cargo binstall cargo-nextest -y

_install_cargo-insta: _install_cargo-binstall
    cargo binstall cargo-insta -y

_install_cargo-release: _install_cargo-binstall
    cargo binstall cargo-release -y

_install_git-cliff: _install_cargo-binstall
    cargo binstall git-cliff -y

# Format the code and sort dependencies
format:
    cargo fmt
    # cargo sort --workspace --grouped

deny:
    cargo deny check advisories
    cargo deny check bans licenses sources

# Lint the rust code
lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-features --all-targets -- --deny warnings

megalinter:
    @just _container run --pull always --rm -it -v "$PWD:/tmp/lint:rw" "megalinter/megalinter:v7"

# Launch tests
tinstall_cst: _install_cargo-nextest _install_cargo-insta
    cargo nextest run
    cargo test --doc

changelog: _install_git-cliff
    git-cliff -o "CHANGELOG.md"
    git add CHANGELOG.md && git commit -m "📝 update CHANGELOG"

release *arguments: _install_cargo-release _install_git-cliff
    cargo release --workspace --execute {{ arguments }}
    # git-cliff could not be used as `pre-release-hook` of cargo-release because it uses tag
    git-cliff -o "CHANGELOG.md"
    git add CHANGELOG.md && git commit -m "📝 update CHANGELOG" && git push

_container *arguments:
    if [ -x "$(command -v podman)" ]; then \
        podman {{ arguments }}; \
    elif [ -x "$(command -v nerdctl)" ]; then \
        nerdctl {{ arguments }}; \
    elif [ -x "$(command -v docker)" ]; then \
        docker {{ arguments }}; \
    else \
        echo "runner not found: podman or nerdctl or docker"; \
        exit 1; \
    fi

run_jaeger:
    @just _container run --rm --name jaeger \
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
        docker.io/jaegertracing/all-in-one:latest

    # echo "open http://localhost:16686"

run_example_grpc_server:
    cd examples/grpc; OTEL_SERVICE_NAME=grpc-server cargo run --bin server

run_example_grpc_client:
    # grpcurl -plaintext  -d '{"service": "healthcheck"}' 127.0.0.1:50051 grpc.health.v1.Health/Check
    grpc-health-probe -addr 127.0.0.1:50051
    grpcurl -plaintext 127.0.0.1:50051 list
    cd examples/grpc; OTEL_SERVICE_NAME=grpc-client cargo run --bin client

run_example_axum-otlp_server:
    cd examples/axum-otlp; OTEL_SERVICE_NAME=axum-otlp-4317 cargo run

run_example_axum-otlp_server_over_http:
    cd examples/axum-otlp; OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4318/v1/traces" OTEL_SERVICE_NAME=axum-otlp-4318 cargo run --features otlp-over-http

run_example_http_server:
    @just run_example_axum-otlp_server

run_example_http_client:
    # curl -i http://127.0.0.1:3003/health
    curl -i http://127.0.0.1:3003/

run_example_load:
    cd examples/load; cargo run --release 2>/dev/null
