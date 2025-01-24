use opentelemetry::propagation::{Extractor, Injector};

// copy from crate opentelemetry-http (to not be dependants of on 3rd: http, ...)
pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    /// Set a key and value in the `HeaderMap`. Does nothing if the key or value are not valid inputs.
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                self.0.insert(name, val);
            }
        }
    }
}

pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    /// Get a value for a key from the `HeaderMap`. If the value is not valid ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    /// Collect all the keys from the `HeaderMap`.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(http::HeaderName::as_str)
            .collect::<Vec<_>>()
    }
}
