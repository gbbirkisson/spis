use super::gallery::render;
use super::render::Response;
use super::State;
use crate::server::Config;
use actix_web::get;
use actix_web::web::{Data, Path, Query};
use sqlx::{Pool, Sqlite};

#[get("/favorite")]
async fn favorite(pool: Data<Pool<Sqlite>>, config: Data<Config>, state: Query<State>) -> Response {
    let mut state = state.into_inner();
    state.favorite = state.favorite.or(Some(false)).map(|b| !b);
    render(&pool, &config, state).await
}

#[get("/year/{year}")]
async fn year(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
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
    render(&pool, &config, state).await
}

#[get("/month/{month}")]
async fn month(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
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
    render(&pool, &config, state).await
}

#[get("/order")]
async fn order(pool: Data<Pool<Sqlite>>, config: Data<Config>, state: Query<State>) -> Response {
    let mut state = state.into_inner();
    state.new_to_old = state.new_to_old.or(Some(true)).map(|b| !b);
    render(&pool, &config, state).await
}

#[get("/clear")]
async fn clear(pool: Data<Pool<Sqlite>>, config: Data<Config>) -> Response {
    render(&pool, &config, State::default()).await
}
