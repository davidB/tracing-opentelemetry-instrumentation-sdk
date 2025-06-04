use super::grpc;

pub use grpc::update_span_from_response_or_error;

pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    grpc::make_span_from_request(req, opentelemetry::trace::SpanKind::Client)
}
