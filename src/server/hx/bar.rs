use super::gallery::render;
use super::ServerError;
use super::State;
use actix_web::web::{Data, Path, Query};
use actix_web::{get, HttpResponse};
use sqlx::{Pool, Sqlite};

#[get("/favorite")]
async fn favorite(
    pool: Data<Pool<Sqlite>>,
    state: Query<State>,
) -> Result<HttpResponse, ServerError> {
    let mut state = state.into_inner();
    state.favorite = state.favorite.or(Some(false)).map(|b| !b);
    render(&pool, state).await
}

#[get("/year/{year}")]
async fn year(
    pool: Data<Pool<Sqlite>>,
    state: Query<State>,
    path: Path<usize>,
) -> Result<HttpResponse, ServerError> {
    let mut state = state.into_inner();
    let year = path.into_inner();
    if state.year == Some(year) {
        state.year = None;
    } else {
        state.year = Some(year);
    }
    state.month = None;
    render(&pool, state).await
}

#[get("/month/{month}")]
async fn month(
    pool: Data<Pool<Sqlite>>,
    state: Query<State>,
    path: Path<u8>,
) -> Result<HttpResponse, ServerError> {
    let mut state = state.into_inner();
    let month = path.into_inner();
    assert!(state.year.is_some());
    if state.month == Some(month) {
        state.month = None;
    } else {
        state.month = Some(month);
    }
    render(&pool, state).await
}

#[get("/bar/clear")]
async fn clear(pool: Data<Pool<Sqlite>>) -> Result<HttpResponse, ServerError> {
    render(&pool, State::default()).await
}
