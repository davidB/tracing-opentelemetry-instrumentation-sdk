use generated::greeter_server::{Greeter, GreeterServer};
use generated::{HelloReply, HelloRequest, StatusRequest};
use tonic::Code;
use tonic::{transport::Server, Request, Response, Status};
use tonic_tracing_opentelemetry::middleware::{filters, server};

pub mod generated {
    //tonic::include_proto!("helloworld");
    include!("generated/helloworld.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        //tonic::include_file_descriptor_set!("helloworld_descriptor");
        include_bytes!("generated/helloworld_descriptor.bin");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    #[tracing::instrument(skip(self, request))]
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let trace_id = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
        tracing::info!(
            "Got a request from {:?} ({:?})",
            request.remote_addr(),
            trace_id
        );

        let reply = generated::HelloReply {
            message: format!("Hello {}! ({:?})", request.into_inner().name, trace_id),
        };
        Ok(Response::new(reply))
    }

    #[tracing::instrument(skip(self, request))]
    async fn say_status(&self, request: Request<StatusRequest>) -> Result<Response<()>, Status> {
        let trace_id = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
        let request = request.into_inner();
        tracing::info!("ask to return status : {} ({:?})", request.code, trace_id);
        Err(Status::new(Code::from(request.code), request.message))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // very opinionated init of tracing, look as is source to make your own
    let _guard = init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()
        .expect("init subscribers");

    let addr = "0.0.0.0:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    let (_, health_service) = tonic_health::server::health_reporter();
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(generated::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        // create trace for every request including health_service
        .layer(server::OtelGrpcLayer::default().filter(filters::reject_healthcheck))
        .add_service(health_service)
        .add_service(reflection_service)
        //.add_service(GreeterServer::new(greeter))
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
