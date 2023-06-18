pub mod client;
pub mod filters;
pub mod server;

fn extract_service_method(path: &str) -> (&str, &str) {
    let mut parts = path.split('/').filter(|x| !x.is_empty());
    let service = parts.next().unwrap_or_default();
    let method = parts.next().unwrap_or_default();
    (service, method)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::*;
    use rstest::*;

    #[rstest]
    #[case("", "", "")]
    #[case("/", "", "")]
    #[case("//", "", "")]
    #[case("/grpc.health.v1.Health/Check", "grpc.health.v1.Health", "Check")]
    fn test_extract_service_method(
        #[case] path: &str,
        #[case] service: &str,
        #[case] method: &str,
    ) {
        check!(extract_service_method(path) == (service, method));
    }
}
