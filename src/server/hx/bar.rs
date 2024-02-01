use super::gallery::render;
use super::State;
use actix_web::get;
use actix_web::web::{Path, Query};
use actix_web::Responder;

#[get("/favorite")]
async fn favorite(state: Query<State>) -> actix_web::Result<impl Responder> {
    let mut state = state.into_inner();
    state.filter_favorite = state.filter_favorite.or(Some(false)).map(|b| !b);
    render(state).await
}

#[get("/year/{year}")]
async fn year(state: Query<State>, path: Path<usize>) -> actix_web::Result<impl Responder> {
    let mut state = state.into_inner();
    let year = path.into_inner();
    if state.filter_year == Some(year) {
        state.filter_year = None;
    } else {
        state.filter_year = Some(year);
    }
    state.filter_month = None;
    render(state).await
}

#[get("/month/{month}")]
async fn month(state: Query<State>, path: Path<u8>) -> actix_web::Result<impl Responder> {
    let mut state = state.into_inner();
    let month = path.into_inner();
    assert!(state.filter_year.is_some());
    if state.filter_month == Some(month) {
        state.filter_month = None;
    } else {
        state.filter_month = Some(month);
    }
    render(state).await
}

#[get("/bar/clear")]
async fn clear() -> actix_web::Result<impl Responder> {
    render(State::default()).await
}
