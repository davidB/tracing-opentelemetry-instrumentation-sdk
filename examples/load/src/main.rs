use std::time::Instant;

use memory_stats::memory_stats;
use tracing::field::Empty;
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    let mut stats = memory_stats();
    if stats.is_none() {
        eprintln!("Couldn't get the current memory usage :(");
        return Ok(());
    }
    let start = Instant::now();
    loop {
        let prev_stats = stats;
        stats = memory_stats();
        if stats != prev_stats {
            println!(
                "{}s Current memory usage: {:?}",
                start.elapsed().as_secs(),
                stats
            );
        }
        for _i in 1..10000 {
            let _span = tracing::info_span!(
                target: TRACING_TARGET,
                "Load",
                http.request.method = "GET",
                http.route = Empty,
                network.protocol.version = "1.1",
                http.client.address = Empty,
                http.response.status_code = Empty,
                otel.kind = "Sever",
                otel.status_code = Empty,
                trace_id = Empty,
                request_id = Empty,
                exception.message = Empty,
                //"span.type" = SpanType::Web.to_string(),
            )
            .entered();
            //eprintln!("trace_id: {:?}", tracing_opentelemetry_instrumentation_sdk::find_current_trace_id());
        }
    }
}
