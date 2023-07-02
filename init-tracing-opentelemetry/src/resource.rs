use opentelemetry::sdk::{
    resource::{OsResourceDetector, ResourceDetector},
    Resource,
};
use opentelemetry_semantic_conventions as semcov;
use std::time::Duration;

/// To log detected value set environement variable `RUST_LOG="...,otel::setup::resource=debug"`
/// ```rust
/// use init_tracing_opentelemetry::resource::DetectResource;
/// # fn main() {
/// let otel_rsrc = DetectResource::default()
///     .with_fallback_service_name(env!("CARGO_PKG_NAME"))
///     .with_fallback_service_version(env!("CARGO_PKG_VERSION"))
///     .build();
/// # }
///
/// ```
#[derive(Debug, Default)]
pub struct DetectResource {
    fallback_service_name: Option<&'static str>,
    fallback_service_version: Option<&'static str>,
}

impl DetectResource {
    /// `service.name` is first extracted from environment variables
    /// (in this order) `OTEL_SERVICE_NAME`, `SERVICE_NAME`, `APP_NAME`.
    /// But a default value can be provided with this method.
    #[must_use]
    pub fn with_fallback_service_name(mut self, fallback_service_name: &'static str) -> Self {
        self.fallback_service_name = Some(fallback_service_name);
        self
    }

    /// `service.name` is first extracted from environment variables
    /// (in this order) `SERVICE_VERSION`, `APP_VERSION`.
    /// But a default value can be provided with this method.
    #[must_use]
    pub fn with_fallback_service_version(mut self, fallback_service_version: &'static str) -> Self {
        self.fallback_service_version = Some(fallback_service_version);
        self
    }

    #[must_use]
    pub fn build(mut self) -> Resource {
        let base = Resource::default();
        let fallback = Resource::from_detectors(
            Duration::from_secs(0),
            vec![
                Box::new(ServiceInfoDetector {
                    fallback_service_name: self.fallback_service_name.take(),
                    fallback_service_version: self.fallback_service_version.take(),
                }),
                Box::new(OsResourceDetector),
                //Box::new(ProcessResourceDetector),
            ],
        );
        let rsrc = base.merge(&fallback); // base has lower priority
        debug_resource(&rsrc);
        rsrc
    }
}

pub fn debug_resource(rsrc: &Resource) {
    rsrc.iter().for_each(
        |kv| tracing::debug!(target: "otel::setup::resource", key = %kv.0, value = %kv.1),
    );
}

#[derive(Debug)]
pub struct ServiceInfoDetector {
    fallback_service_name: Option<&'static str>,
    fallback_service_version: Option<&'static str>,
}

impl ResourceDetector for ServiceInfoDetector {
    fn detect(&self, _timeout: Duration) -> Resource {
        let service_name = std::env::var("OTEL_SERVICE_NAME")
            .or_else(|_| std::env::var("SERVICE_NAME"))
            .or_else(|_| std::env::var("APP_NAME"))
            .ok()
            .or_else(|| {
                self.fallback_service_name
                    .map(std::string::ToString::to_string)
            })
            .map(|v| semcov::resource::SERVICE_NAME.string(v));
        let service_version = std::env::var("SERVICE_VERSION")
            .or_else(|_| std::env::var("APP_VERSION"))
            .ok()
            .or_else(|| {
                self.fallback_service_version
                    .map(std::string::ToString::to_string)
            })
            .map(|v| semcov::resource::SERVICE_VERSION.string(v));
        Resource::new(vec![service_name, service_version].into_iter().flatten())
    }
}
