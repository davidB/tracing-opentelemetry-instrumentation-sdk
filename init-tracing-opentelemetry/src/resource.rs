use opentelemetry::KeyValue;
// use opentelemetry_resource_detectors::OsResourceDetector;
use opentelemetry_sdk::{resource::ResourceDetector, Resource};
use opentelemetry_semantic_conventions::resource;

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
#[derive(Debug, Default, Clone)]
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
    pub fn build(&mut self) -> Resource {
        //Box::new(OsResourceDetector), //FIXME enable when available for opentelemetry >= 0.25
        //Box::new(ProcessResourceDetector),
        let rsrc = Resource::builder()
            .with_detector(Box::new(ServiceInfoDetector {
                fallback_service_name: self.fallback_service_name.take(),
                fallback_service_version: self.fallback_service_version.take(),
            }))
            .build();
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
    fn detect(&self) -> Resource {
        let service_name = std::env::var("OTEL_SERVICE_NAME")
            .or_else(|_| std::env::var("SERVICE_NAME"))
            .or_else(|_| std::env::var("APP_NAME"))
            .ok()
            .or_else(|| {
                self.fallback_service_name
                    .map(std::string::ToString::to_string)
            })
            .map(|v| KeyValue::new(resource::SERVICE_NAME, v));
        let service_version = std::env::var("SERVICE_VERSION")
            .or_else(|_| std::env::var("APP_VERSION"))
            .ok()
            .or_else(|| {
                self.fallback_service_version
                    .map(std::string::ToString::to_string)
            })
            .map(|v| KeyValue::new(resource::SERVICE_VERSION, v));
        let mut resource = Resource::builder_empty();
        if let Some(service_name) = service_name {
            resource = resource.with_attribute(service_name);
        }
        if let Some(service_version) = service_version {
            resource = resource.with_attribute(service_version);
        }
        resource.build()
    }
}
