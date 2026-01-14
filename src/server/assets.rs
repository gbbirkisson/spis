use crate::server::AppState;
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use std::path::Path;

static ASSETS: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets");

async fn serve_asset(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    let Some(file) = ASSETS.get_file(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mime_type = match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("json") => "application/json",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("css") => "text/css",
        Some("ttf") => "font/ttf",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    };

    Response::builder()
        .header(header::CONTENT_TYPE, HeaderValue::from_static(mime_type))
        .body(Body::from(file.contents()))
        .expect("Failed to build response")
}

pub async fn serve_sw() -> impl IntoResponse {
    let Some(file) = ASSETS.get_file("sw.js") else {
        return StatusCode::NOT_FOUND.into_response();
    };

    Response::builder()
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/javascript"),
        )
        .body(Body::from(file.contents()))
        .expect("Failed to build response")
}

pub fn create_router() -> Router<AppState> {
    Router::new().route("/{*path}", get(serve_asset))
}
