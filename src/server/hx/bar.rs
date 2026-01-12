use super::GalleryState;
use super::gallery::render;
use super::render::RenderResult;
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::{Router, routing::get};

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

async fn clear(State(app_state): State<AppState>) -> RenderResult {
    render(&app_state, GalleryState::default()).await
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/favorite", get(favorite))
        .route("/year/{year}", get(year))
        .route("/month/{month}", get(month))
        .route("/order", get(order))
        .route("/clear", get(clear))
}
