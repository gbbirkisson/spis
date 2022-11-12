use std::{net::TcpListener, path::PathBuf};

use spis_server::{db, img, run};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let img_dir = PathBuf::from("dev/api/images");
    let thumb_dir = PathBuf::from("dev/api/state/thumbnails");
    let db_url = PathBuf::from("dev/api/state/spis.db");

    let pool = db::setup_db(db_url).await.unwrap();

    let (tx, mut rx) = channel(32);
    img::image_processor(img_dir, thumb_dir, tx);

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(img) => {
                    db::insert_image(&pool, img).await;
                }
                None => {
                    tracing::info!("None from channel");
                }
            }
        }
    });

    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    let server = run(listener).expect("Failed to create server");
    server.await
}
