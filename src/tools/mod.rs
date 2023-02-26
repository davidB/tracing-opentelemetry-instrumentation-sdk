use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
#[cfg(feature = "tracer")]
use opentelemetry::sdk::trace::Tracer;
#[cfg(feature = "tracer")]
use opentelemetry::sdk::Resource;
use opentelemetry::trace::TraceError;

#[cfg(feature = "jaeger")]
pub mod jaeger;
#[cfg(feature = "otlp")]
pub mod otlp;
#[cfg(feature = "tracer")]
pub mod resource;
#[cfg(feature = "tracer")]
pub mod stdio;
#[cfg(feature = "tracing_subscriber_ext")]
pub mod tracing_subscriber_ext;

#[cfg(feature = "tracer")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectorKind {
    #[cfg(feature = "otlp")]
    Otlp,
    #[cfg(feature = "jaeger")]
    Jaeger,
    Stdout,
    Stderr,
    NoWrite,
}

#[cfg(feature = "tracer")]
#[deprecated(
    since = "0.10.0",
    note = "call `init_tracer` from sub sub package directly"
)]
pub fn init_tracer(kind: CollectorKind, resource: Resource) -> Result<Tracer, TraceError> {
    match kind {
        CollectorKind::Stdout => stdio::init_tracer(resource, stdio::identity, std::io::stdout()),
        CollectorKind::Stderr => stdio::init_tracer(resource, stdio::identity, std::io::stderr()),
        CollectorKind::NoWrite => {
            stdio::init_tracer(resource, stdio::identity, stdio::WriteNoWhere::default())
        }
        #[cfg(feature = "otlp")]
        CollectorKind::Otlp => {
            // if let Some(url) = std::env::var_os("OTEL_COLLECTOR_URL")
            // "http://localhost:14499/otlp/v1/traces"
            // let collector_url = url.to_str().ok_or(TraceError::Other(
            //     anyhow!("failed to parse OTEL_COLLECTOR_URL").into(),
            // ))?;
            otlp::init_tracer(resource, otlp::identity)
        }
        #[cfg(feature = "jaeger")]
        CollectorKind::Jaeger => {
            // Or "OTEL_EXPORTER_JAEGER_ENDPOINT"
            // or now variable
            jaeger::init_tracer(resource, jaeger::identity)
        }
    }
}

/// Configure the global propagator based on content of the env variable [OTEL_PROPAGATORS](https://opentelemetry.io/docs/concepts/sdk-configuration/general-sdk-configuration/#otel_propagators)
/// Specifies Propagators to be used in a comma-separated list.
/// Default value: `"tracecontext,baggage"`
/// Example: `export OTEL_PROPAGATORS="b3"`
/// Accepted values for `OTEL_PROPAGATORS` are:
///
/// - "tracecontext": W3C Trace Context
/// - "baggage": W3C Baggage
/// - "b3": B3 Single (require feature "zipkin")
/// - "b3multi": B3 Multi (require feature "zipkin")
/// - "jaeger": Jaeger (require feature "jaeger")
/// - "xray": AWS X-Ray (require feature "xray")
/// - "ottrace": OT Trace (third party) (not supported)
/// - "none": No automatically configured propagator.
pub fn init_propagator() -> Result<(), TraceError> {
    let value_from_env =
        std::env::var("OTEL_PROPAGATORS").unwrap_or_else(|_| "tracecontext,baggage".to_string());
    let propagators: Vec<(Box<dyn TextMapPropagator + Send + Sync>, String)> = value_from_env
        .split(',')
        .map(|s| {
            let name = s.trim().to_lowercase();
            propagator_from_string(&name).map(|o| o.map(|b| (b, name)))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    if !propagators.is_empty() {
        let (propagators_impl, propagators_name): (Vec<_>, Vec<_>) =
            propagators.into_iter().unzip();
        tracing::debug!(target: "otel::setup", OTEL_PROPAGATORS = propagators_name.join(","));
        let composite_propagator = TextMapCompositePropagator::new(propagators_impl);
        opentelemetry::global::set_text_map_propagator(composite_propagator);
    }
    Ok(())
}

#[allow(clippy::box_default)]
fn propagator_from_string(
    v: &str,
) -> Result<Option<Box<dyn TextMapPropagator + Send + Sync>>, TraceError> {
    match v {
        "tracecontext" => Ok(Some(Box::new(TraceContextPropagator::new()))),
        "baggage" => Ok(Some(Box::new(BaggagePropagator::new()))),
        #[cfg(feature = "zipkin")]
        "b3" => Ok(Some(Box::new(
            opentelemetry_zipkin::Propagator::with_encoding(
                opentelemetry_zipkin::B3Encoding::SingleHeader,
            ),
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'b3', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "zipkin")]
        "b3multi" => Ok(Some(Box::new(
            opentelemetry_zipkin::Propagator::with_encoding(
                opentelemetry_zipkin::B3Encoding::MultipleHeader,
            ),
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3multi" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'b3multi', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "jaeger")]
        "jaeger" => Ok(Some(Box::new(
            opentelemetry_jaeger::Propagator::default()
        ))),
        #[cfg(not(feature = "jaeger"))]
        "jaeger" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'jaeger', try to enable compile feature 'jaeger'"
        )),
        #[cfg(feature = "xray")]
        "xray" => Ok(Some(Box::new(
            opentelemetry_aws::trace::XrayPropagator::default(),
        ))),
        #[cfg(not(feature = "xray"))]
        "xray" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'xray', try to enable compile feature 'xray'"
        )),
        "none" => Ok(None),
        unknown => Err(TraceError::from(format!(
            "unsupported propagators form env OTEL_PROPAGATORS: '{unknown}'"
        ))),
    }
}

/// Search the current opentelemetry trace id into the Context from the current tracing'span.
/// This function can be used to report the trace id into the error message send back to user.
///
/// ```rust
/// let trace_id = axum_tracing_opentelemetry::find_current_trace_id();
/// // json!({ "error" :  "xxxxxx", "trace_id": trace_id})
///
/// ```
pub fn find_current_trace_id() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    // let context = opentelemetry::Context::current();
    // OpenTelemetry Context is propagation inside code is done via tracing crate
    let context = tracing::Span::current().context();
    let span = context.span();
    let span_context = span.span_context();
    span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string())
}

#[cfg(test)]
#[cfg(feature = "tracer")]
mod tests {
    use assert2::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        BoxError, Router,
    };
    use tower::Service; // for `call`
    use tower::ServiceExt; // for `oneshot` and `ready`
    use tracing::subscriber::DefaultGuard;

    fn init_tracing() -> Result<DefaultGuard, BoxError> {
        use tracing_subscriber::filter::EnvFilter;
        use tracing_subscriber::fmt::format::FmtSpan;
        use tracing_subscriber::layer::SubscriberExt;

        let subscriber = tracing_subscriber::registry();

        // register opentelemetry tracer layer
        let otel_layer = {
            use crate::{
                init_propagator,
                resource::DetectResource,
                //otlp,
                stdio,
            };
            let otel_rsrc = DetectResource::default().build();
            // let otel_tracer =
            //     otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");
            let otel_tracer =
                stdio::init_tracer(otel_rsrc, stdio::identity, stdio::WriteNoWhere::default())?;
            init_propagator()?;
            tracing_opentelemetry::layer().with_tracer(otel_tracer)
        };
        let subscriber = subscriber.with(otel_layer);

        // filter what is output on log (fmt), but not what is send to trace (opentelemetry collector)
        // std::env::set_var("RUST_LOG", "info,kube=trace");
        std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG")
                .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
                .unwrap_or_else(|_| "info".to_string()),
        );
        let subscriber = subscriber.with(EnvFilter::from_default_env());

        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_timer(tracing_subscriber::fmt::time::uptime());
        let subscriber = subscriber.with(fmt_layer);

        Ok(tracing::subscriber::set_default(subscriber))
    }

    /// Having a function that produces our app makes it easy to call it from tests
    /// without having to create an HTTP server.
    #[allow(dead_code)]
    fn app() -> (Router, DefaultGuard) {
        let guard = init_tracing().unwrap();

        let router = Router::new()
            .route(
                "/",
                get(|| async { crate::find_current_trace_id().unwrap_or_default() }),
            )
            // include trace context as header into the response
            .layer(crate::response_with_trace_layer())
            .layer(crate::opentelemetry_tracing_layer());
        (router, guard)
    }

    #[tokio::test]
    async fn trace_id_propagate_into_response() {
        if cfg!(feature = "zipkin") {
            std::env::set_var("OTEL_PROPAGATORS", "tracecontext,b3multi");
        } else {
            std::env::set_var("OTEL_PROPAGATORS", "tracecontext");
        }
        let (mut app, _guard) = app();

        let request = Request::builder()
            .uri("/")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let response = app.ready().await.unwrap().call(request).await.unwrap();
        check!(response.status() == StatusCode::OK);

        let trace_id_b3 = String::from_utf8(
            response
                .headers()
                .get("X-B3-TraceId")
                .map(|h| h.as_bytes().to_vec())
                .unwrap_or_default(),
        )
        .unwrap();
        let trace_id_context = String::from_utf8(
            response
                .headers()
                .get("traceparent")
                .map(|h| h.as_bytes().to_vec())
                .unwrap_or_default(),
        )
        .unwrap();
        let trace_id_body = String::from_utf8(
            hyper::body::to_bytes(response.into_body())
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();

        check!(trace_id_context.contains(&trace_id_body));
        if cfg!(feature = "zipkin") {
            check!(trace_id_b3 == trace_id_body, "B3 should be as trace Id");
        } else {
            check!(trace_id_b3.is_empty(), "B3 should be empty");
        }
    }

    #[test]
    fn init_tracing_failed_on_invalid_propagator() {
        let_assert!(Err(_) = super::propagator_from_string("xxxxxx"));

        // std::env::set_var("OTEL_PROPAGATORS", "xxxxxx");
        // dbg!(std::env::var("OTEL_PROPAGATORS"));
        // let_assert!(Err(_) = init_tracing());
    }
}
