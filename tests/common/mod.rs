use spis::{
    PathFinder, db,
    media::{ProcessedMedia, ProcessedMediaData, ProcessedMediaType},
    server::{Config, Features, Listener},
};
use sqlx::{Pool, Sqlite};
use std::net::TcpListener;
use std::sync::{Arc, Once};
use tempfile::NamedTempFile;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        dotenv::dotenv().ok();
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();
    });
}

pub async fn setup_db() -> (Pool<Sqlite>, NamedTempFile) {
    let file = NamedTempFile::new().expect("Failed to create temp file");
    let pool = db::setup_db(file.path().to_str().unwrap())
        .await
        .expect("Failed to setup DB");
    (pool, file)
}

pub async fn spawn_server() -> (String, Pool<Sqlite>, NamedTempFile) {
    init_tracing();
    let (pool, db_file) = setup_db().await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let pathfinder = PathFinder::new("", "", "", "");
    let config = Config {
        features: Features {
            archive_allow: true,
            favorite_allow: true,
            slideshow_duration: 10,
        },
        pathfinder,
    };

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        spis::server::run(Listener::Tcp(listener), pool_clone, config)
            .await
            .expect("Server failed");
    });

    (format!("http://127.0.0.1:{port}"), pool, db_file)
}

pub async fn insert_dummy_media(pool: &Pool<Sqlite>, count: i32) -> Vec<uuid::Uuid> {
    let mut uuids = Vec::new();
    for i in 0..count {
        let uuid = uuid::Uuid::new_v4();
        db::media_insert(
            pool,
            ProcessedMedia {
                uuid,
                path: format!("/tmp/media_{}.jpg", i),
                data: Some(ProcessedMediaData {
                    taken_at: chrono::Utc::now(),
                }),
                media_type: ProcessedMediaType::Image,
            },
        )
        .await
        .expect("Failed to insert media");
        uuids.push(uuid);
    }
    uuids
}
