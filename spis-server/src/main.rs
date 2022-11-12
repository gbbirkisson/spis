use std::{net::TcpListener, path::PathBuf};

use spis_server::{db, img, server};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Setup logging
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Setup paths
    let img_dir = PathBuf::from("dev/api/images");
    let thumb_dir = PathBuf::from("dev/api/state/thumbnails");
    let db_url = PathBuf::from("dev/api/state/spis.db");

    // Setup DB
    let pool = db::setup_db(db_url).await.unwrap();

    // Process images
    let processor_pool = pool.clone();
    tokio::spawn(async move {
        img::process(processor_pool, img_dir, thumb_dir).await;
    });

    // Start server
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    let server = server::run(listener, pool).expect("Failed to create server");
    server.await
}
