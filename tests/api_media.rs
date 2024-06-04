use spis::{
    db,
    media::{ProcessedMedia, ProcessedMediaData, ProcessedMediaType},
    server::{Config, Features, Listener},
    PathFinder,
};
use std::net::TcpListener;
use tempfile::NamedTempFile;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

async fn spawn_server() -> String {
    // Init logging
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Create listener
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let (_, db_file) = NamedTempFile::new()
        .expect("Failed to create file")
        .keep()
        .expect("Failed to keep");

    // Create DB
    let pool = db::setup_db(&db_file.to_str().unwrap())
        .await
        .expect("Failed to create DB");

    // Insert phony record
    db::media_insert(
        &pool,
        ProcessedMedia {
            uuid: uuid::Uuid::new_v4(),
            path: "".to_string(),
            data: Some(ProcessedMediaData {
                taken_at: chrono::Utc::now(),
            }),
            media_type: ProcessedMediaType::Image,
        },
    )
    .await
    .expect("Failed to insert record");

    let pathfinder = PathFinder::new("", "", "", "");

    // Spawn server
    let config = Config {
        features: Features {
            archive_allow: true,
            favorite_allow: true,
        },
        pathfinder,
    };
    let server =
        spis::server::run(Listener::Tcp(listener), pool, config).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    // Return endpoint
    format!("http://127.0.0.1:{}", port)
}

// async fn fetch(client: &reqwest::Client, address: &String) -> Vec<Media> {
//     let response = client
//         .get(&format!("{}/api?page_size=5", &address))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//     assert!(response.status().is_success());
//
//     response
//         .json::<Vec<Media>>()
//         .await
//         .expect("Failed to parse json")
// }

#[tokio::test]
async fn media_works() {
    let _address = spawn_server().await;
    let _client = reqwest::Client::new();

    // let media = fetch(&client, &address).await;
    // assert_eq!(1, media.len());
    //
    // let media = media.get(0).expect("No media fetched");
    // assert_eq!(false, media.favorite);
    // assert_eq!(false, media.archived);
    //
    // let response = client
    //     .post(&format!("{}/api/{}?favorite=true", &address, &media.uuid))
    //     .send()
    //     .await
    //     .expect("Failed to execute request.");
    // assert!(response.status().is_success());
    //
    // let media = fetch(&client, &address).await;
    // assert_eq!(1, media.len());
    //
    // let media = media.get(0).expect("No media fetched");
    // assert_eq!(true, media.favorite);
    // assert_eq!(false, media.archived);
    //
    // let response = client
    //     .post(&format!("{}/api/{}?archive=true", &address, &media.uuid))
    //     .send()
    //     .await
    //     .expect("Failed to execute request.");
    // assert!(response.status().is_success());
    //
    // let media = fetch(&client, &address).await;
    // assert_eq!(0, media.len());
}
