use std::{net::TcpListener, path::PathBuf};

use spis_server::db::setup_db;
use uuid::Uuid;

async fn spawn_server() -> String {
    // Create listener
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // Create DB
    let db_file = PathBuf::from("/tmp/").join(Uuid::new_v4().to_string());
    let pool = setup_db(db_file).await.expect("Failed to create DB");

    let thumb_dir = PathBuf::from("/tmp");

    // Spawn server
    let server =
        spis_server::server::run(listener, pool, thumb_dir).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    // Return endpoint
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/health", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
