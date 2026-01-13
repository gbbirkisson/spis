use super::GalleryState;
use super::gallery::render;
use super::render::{RenderResult, ServerError, TemplatedResponse};
use crate::db;
use crate::server::AppState;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::{Router, routing::get};
use serde::Deserialize;

async fn favorite(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let mut state = state;
    state.favorite = state.favorite.or(Some(false)).map(|b| !b);
    render(&app_state, state).await
}

async fn year(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Path(year): Path<usize>,
) -> RenderResult {
    let mut state = state;
    if state.year == Some(year) {
        state.year = None;
    } else {
        state.year = Some(year);
    }
    state.month = None;
    render(&app_state, state).await
}

async fn month(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Path(month): Path<u8>,
) -> RenderResult {
    let mut state = state;
    assert!(state.year.is_some());
    if state.month == Some(month) {
        state.month = None;
    } else {
        state.month = Some(month);
    }
    render(&app_state, state).await
}

async fn order(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let mut state = state;
    state.new_to_old = state.new_to_old.or(Some(true)).map(|b| !b);
    render(&app_state, state).await
}

#[derive(Deserialize, Debug, Clone)]
pub(super) struct CollectionPick {
    pick: Option<String>,
}

async fn collection(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Query(pick): Query<CollectionPick>,
) -> RenderResult {
    let collection = if let Some(pick) = pick.pick {
        Some(pick)
    } else if state.collection.is_none() {
        Some("/".to_string())
    } else {
        None
    };
    let state = GalleryState {
        new_to_old: state.new_to_old,
        collection,
        ..Default::default()
    };
    render(&app_state, state).await
}

#[derive(Deserialize, Debug, Clone)]
pub(super) struct SearchQuery {
    query: String,
}

#[derive(Template)]
#[template(path = "web/bar/search.html")]
struct HxSearchResults {
    results: Vec<String>,
}

async fn search(
    State(app_state): State<AppState>,
    Query(search): Query<SearchQuery>,
) -> RenderResult {
    let results =
        db::collections_search(&app_state.pool, &app_state.config.root_path, &search.query)
            .await
            .map_err(ServerError::DBError)?;
    HxSearchResults { results }.render_response()
}

async fn clear(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    let state = GalleryState {
        collection: state.collection,
        ..Default::default()
    };
    render(&app_state, state).await
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/favorite", get(favorite))
        .route("/year/{year}", get(year))
        .route("/month/{month}", get(month))
        .route("/order", get(order))
        .route("/collection", get(collection))
        .route("/clear", get(clear))
        .route("/search", get(search))
}
