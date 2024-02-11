use hello_world::greeter_client::GreeterClient;
use hello_world::{HelloRequest, StatusRequest};
use tonic::transport::Channel;
use tonic::Code;
use tonic_tracing_opentelemetry::middleware::client::OtelGrpcLayer;
use tower::ServiceBuilder;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // very opinionated init of tracing, look as is source to make your own
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()
        .expect("init subscribers");

    // let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?; //Devskim: ignore DS137138
    let channel = ServiceBuilder::new().layer(OtelGrpcLayer).service(channel);

    let mut client = GreeterClient::new(channel);
    {
        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into(),
        });

        let response = client.say_hello(request).await?;

        println!("RESPONSE={:?}", response);
    }
    {
        let request = tonic::Request::new(StatusRequest {
            code: Code::NotFound.into(),
            message: "not found...".into(),
        });

        let response = client.say_status(request).await;

        println!("RESPONSE={:?}", response);
    }
    {
        let request = tonic::Request::new(StatusRequest {
            code: Code::DeadlineExceeded.into(),
            message: "deadline...".into(),
        });

        let response = client.say_status(request).await;

        println!("RESPONSE={:?}", response);
    }

    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
