use super::gallery::render;
use super::render::Response;
use super::State;
use crate::PathFinder;
use actix_web::get;
use actix_web::web::{Data, Path, Query};
use sqlx::{Pool, Sqlite};

#[get("/favorite")]
async fn favorite(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
) -> Response {
    let mut state = state.into_inner();
    state.favorite = state.favorite.or(Some(false)).map(|b| !b);
    render(&pool, &pathfinder, state).await
}

#[get("/year/{year}")]
async fn year(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
    path: Path<usize>,
) -> Response {
    let mut state = state.into_inner();
    let year = path.into_inner();
    if state.year == Some(year) {
        state.year = None;
    } else {
        state.year = Some(year);
    }
    state.month = None;
    render(&pool, &pathfinder, state).await
}

#[get("/month/{month}")]
async fn month(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
    path: Path<u8>,
) -> Response {
    let mut state = state.into_inner();
    let month = path.into_inner();
    assert!(state.year.is_some());
    if state.month == Some(month) {
        state.month = None;
    } else {
        state.month = Some(month);
    }
    render(&pool, &pathfinder, state).await
}

#[get("/bar/clear")]
async fn clear(pool: Data<Pool<Sqlite>>, pathfinder: Data<PathFinder>) -> Response {
    render(&pool, &pathfinder, State::default()).await
}
