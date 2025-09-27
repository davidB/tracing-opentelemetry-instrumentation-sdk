use super::grpc;

pub fn make_span_from_request<B>(req: &http::Request<B>) -> tracing::Span {
    grpc::make_span_from_request(req, opentelemetry::trace::SpanKind::Client)
}
