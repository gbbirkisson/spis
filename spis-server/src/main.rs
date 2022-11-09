use std::{net::TcpListener, path::PathBuf};

use spis_server::{img, run, state::State};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let img_dir = PathBuf::from("dev/api/images");
    let thumb_dir = PathBuf::from("dev/api/state/thumbnails");

    let (tx, mut rx) = channel(32);
    img::image_processor(img_dir, thumb_dir, tx);

    tokio::spawn(async move {
        loop {
            rx.recv().await;
        }
    });

    let state = State::load("dev/api/state", "dev/api/images");
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    let server = run(state, listener).expect("Failed to create server");
    server.await
}
