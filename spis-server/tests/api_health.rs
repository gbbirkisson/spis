use spis_server::{
    db::setup_db,
    server::{convert::MediaConverter, Listener},
};
use std::net::TcpListener;
use tempfile::NamedTempFile;

async fn spawn_server() -> String {
    // Create listener
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // let config = spis_server::SpisCfg::new_testing();

    let db_file = NamedTempFile::new().expect("Failed to create file");

    // Create DB
    let pool = setup_db(&db_file.path().to_str().unwrap())
        .await
        .expect("Failed to create DB");

    let converter = MediaConverter::new("", "", "", "");

    // Spawn server
    let server = spis_server::server::run(Listener::Tcp(listener), pool, converter)
        .expect("Failed to bind address");
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
