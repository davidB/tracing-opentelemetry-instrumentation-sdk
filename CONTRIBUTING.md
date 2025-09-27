# Contributing to tracing-opentelemetry-instrumentation-sdk

Thank you for your interest in contributing! This document provides guidelines and instructions for setting up your development environment and contributing to the project.

## Development Environment Setup

### Prerequisites

1. **Install mise** (if not already installed):

   ```bash
   # Using the install script
   curl https://mise.run | sh

   # Or using a package manager
   # macOS: brew install mise
   # Arch: pacman -S mise
   # Ubuntu/Debian: see https://mise.jdx.dev/installing-mise.html
   ```

2. **Clone the repository**:

   ```bash
   git clone https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk.git
   cd tracing-opentelemetry-instrumentation-sdk
   ```

3. **Install tools and dependencies**:

   ```bash
   # Install all tools defined in .mise.toml (Rust, protoc, grpcurl, etc.)
   mise install

   # Activate the environment (or use 'mise use' for shell integration)
   mise activate
   ```

### Available Development Tasks

We use `mise` tasks for all development workflows. Here are the main tasks:

#### Code Quality & Formatting

```bash
# Format code
mise run format

# Lint code (clippy + format check)
mise run lint

# Check code with all feature combinations
mise run check

# Run cargo deny security checks
mise run deny
```

#### Testing

```bash
# Run tests (using nextest + doctests)
mise run test

# Test each feature separately (slower but thorough)
mise run test-each-feature
```

#### Tool Installation

These tasks automatically install required tools if not present:

```bash
# Individual tool installation (usually handled automatically by other tasks)
mise run install:cargo-hack
mise run install:cargo-nextest
mise run install:cargo-insta
mise run install:cargo-deny
```

#### Container & Examples

```bash
# Start Jaeger all-in-one for local development
mise run run-jaeger

# Run example applications
mise run run-example-grpc-server     # gRPC server example
mise run run-example-grpc-client     # gRPC client example
mise run run-example-axum-otlp-server # Axum HTTP server
mise run run-example-http-client     # HTTP client test
mise run run-example-load            # Load testing example
```

#### Version Management

```bash
# Set version across all workspace crates
mise run set-version 0.1.0
```

### Development Workflow

1. **Setup environment** (first time only):

   ```bash
   mise install
   ```

2. **Before making changes**:

   ```bash
   # Format and check code
   mise run format
   mise run lint
   mise run check
   ```

3. **While developing**:

   ```bash
   # Run tests frequently
   mise run test

   # Test with examples if relevant
   mise run run-jaeger &  # Start Jaeger in background
   mise run run-example-axum-otlp-server
   ```

4. **Before submitting PR**:
   ```bash
   # Full validation
   mise run format
   mise run lint
   mise run check
   mise run test
   mise run deny
   ```

### Testing with OpenTelemetry

#### Local Jaeger Setup

```bash
# Start Jaeger (runs on various ports including 16686 for UI, 4317/4318 for OTLP)
mise run run-jaeger

# Open Jaeger UI
open http://localhost:16686
```

#### Running Examples

```bash
# Terminal 1: Start Jaeger
mise run run-jaeger

# Terminal 2: Start example server
mise run run-example-axum-otlp-server

# Terminal 3: Send requests and check traces
mise run run-example-http-client
# Then check traces in Jaeger UI at http://localhost:16686
```

### Project Structure

This workspace contains several crates:

- **`init-tracing-opentelemetry/`**: Helpers to initialize tracing + opentelemetry
- **`axum-tracing-opentelemetry/`**: Axum middlewares for tracing integration
- **`tonic-tracing-opentelemetry/`**: Tonic (gRPC) middlewares for tracing
- **`tracing-opentelemetry-instrumentation-sdk/`**: Core instrumentation SDK
- **`fake-opentelemetry-collector/`**: Testing utilities and fake collector
- **`testing-tracing-opentelemetry/`**: Test utilities
- **`examples/`**: Working examples demonstrating usage

### Code Style & Guidelines

- **Formatting**: We use `rustfmt` with default settings
- **Linting**: All clippy warnings must be addressed
- **Testing**: Add tests for new functionality
- **Documentation**: Update documentation for public APIs
- **Examples**: Update examples if adding new features

### Continuous Integration

Our CI pipeline runs:

- `mise run check` - Feature compatibility checks
- `mise run lint` - Code formatting and clippy
- `mise run test` - Full test suite
- `mise run deny` - Security and license checks

Make sure all these pass locally before submitting a PR.

### Common Issues & Solutions

#### Tool Installation Issues

If tools fail to install automatically:

```bash
# Install cargo-binstall manually
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Then retry the task
mise run <task-name>
```

#### Container Runtime

The project supports multiple container runtimes (podman, nerdctl, docker). If you have issues:

```bash
# Make sure one of these is installed and available
which podman || which nerdctl || which docker
```

#### Environment Variables

Key environment variables (already set in `.mise.toml`):

- `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://127.0.0.1:4317"`
- `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL="grpc"`
- `OTEL_TRACES_SAMPLER="always_on"`

### Getting Help

- Check existing [issues](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/issues)
- Review the [examples/](examples/) directory for usage patterns
- Look at test files for API usage examples
- Open a new issue for bugs or feature requests

### Submitting Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-new-feature`
3. Make your changes following the guidelines above
4. Run the full test suite: `mise run lint && mise run test && mise run deny`
5. Commit with descriptive messages
6. Push to your fork and submit a pull request

Thank you for contributing!
