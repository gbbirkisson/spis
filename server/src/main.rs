use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router};
use model::Image;
use std::{net::SocketAddr, sync::Arc};

use crate::state::State;

mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(State::load("dev/api/state", "dev/api/images"));

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

async fn get_images(state: Extension<Arc<State>>) -> impl IntoResponse {
    let images: Vec<Image> = state.images().values().cloned().collect::<Vec<Image>>();
    (StatusCode::OK, Json(images))
}
