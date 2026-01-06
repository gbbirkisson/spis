use crate::db;
use crate::server::Config;

use super::render::ServerError;
use super::render::{Response, TemplatedResponse};
use super::Media;
use super::State;
use actix_web::get;
use actix_web::web::Data;
use actix_web::web::Query;
use askama::Template;
use sqlx::Pool;
use sqlx::Sqlite;

#[derive(Template)]
#[template(path = "web/map/map.html")]
struct HxMap<'a> {
    media: &'a Vec<Media>,
}

pub(super) async fn render(pool: &Pool<Sqlite>, config: &Config, state: State) -> Response {
    let media = db::media_with_pos(pool)
        .await
        .map_err(ServerError::DBError)?
        .into_iter()
        .map(|row| (row, &config.pathfinder).into())
        .collect();

    HxMap { media: &media }.render_response()
}

#[get("")]
async fn root(pool: Data<Pool<Sqlite>>, config: Data<Config>, state: Query<State>) -> Response {
    render(&pool, &config, state.into_inner()).await
}
