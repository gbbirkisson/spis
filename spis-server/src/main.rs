use std::net::TcpListener;

use spis_server::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    let server = run(listener).expect("Failed to create server");
    server.await
}
