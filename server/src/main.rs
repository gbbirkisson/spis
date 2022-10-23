use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use model::Image;
use std::{net::SocketAddr, sync::Arc};
use walkdir::{DirEntry, WalkDir};

struct ImageServer {
    images: Vec<Image>,
}

impl ImageServer {
    fn new(dir: &str) -> Self {
        let mut images = Vec::with_capacity(100);

        let walk = WalkDir::new(dir).into_iter();
        for entry in walk.filter_map(|e| e.ok()).filter(Self::filter) {
            images.push(Image {
                path: entry.path().to_str().unwrap().to_string().replace(dir, ""),
            });
        }

        Self { images }
    }

    fn filter(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|f| f.ends_with("jpg"))
            .unwrap_or(false)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(ImageServer::new("dev"));

    let app = Router::new()
        .route("/api", get(get_images))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_images(state: Extension<Arc<ImageServer>>) -> impl IntoResponse {
    let images: Vec<Image> = state.images.to_vec();
    (StatusCode::OK, Json(images))
}
