use crate::db;
use crate::server::AppState;

use super::GalleryState;
use super::Media;
use super::render::{RenderResult, ServerError, TemplatedResponse};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::{
    Router,
    routing::{get, put},
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "web/preview/preview.html")]
struct HxRoot<'a> {
    archive: bool,
    slideshow: bool,
    features: &'a crate::server::Features,
    prev: Option<Media>,
    media: Option<Media>,
    next: Option<Media>,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub(super) struct PreviewState {
    slideshow: Option<bool>,
    archive: Option<bool>,
}

async fn root(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Query(preview): Query<PreviewState>,
    Path(uuid): Path<Uuid>,
) -> RenderResult {
    let pool = &app_state.pool;
    let config = &app_state.config;
    let res = db::media_get(pool, &state, &state, &uuid)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive: preview.archive.unwrap_or_default(),
        slideshow: preview.slideshow.unwrap_or_default(),
        features: &config.features,
        prev: res.0.map(|m| (m, &config.pathfinder).into()),
        media: res.1.map(|m| (m, &config.pathfinder).into()),
        next: res.2.map(|m| (m, &config.pathfinder).into()),
    }
    .render_response()
}

async fn favorite(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Query(preview): Query<PreviewState>,
    Path((uuid, value)): Path<(Uuid, bool)>,
) -> RenderResult {
    let pool = &app_state.pool;
    let config = &app_state.config;

    db::media_favorite(pool, &uuid, value)
        .await
        .map_err(ServerError::DBError)?;

    let res = db::media_get(pool, &state, &state, &uuid)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive: preview.archive.unwrap_or_default(),
        slideshow: preview.slideshow.unwrap_or_default(),
        features: &config.features,
        prev: res.0.map(|m| (m, &config.pathfinder).into()),
        media: res.1.map(|m| (m, &config.pathfinder).into()),
        next: res.2.map(|m| (m, &config.pathfinder).into()),
    }
    .render_response()
}

async fn archive(State(app_state): State<AppState>, Path(uuid): Path<Uuid>) -> RenderResult {
    let pool = &app_state.pool;
    let config = &app_state.config;

    db::media_archive(pool, &uuid, true)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive: false,
        slideshow: false,
        features: &config.features,
        prev: None,
        media: None,
        next: None,
    }
    .render_response()
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/{idx}", get(root).delete(archive))
        .route("/{idx}/favorite/{value}", put(favorite))
}
