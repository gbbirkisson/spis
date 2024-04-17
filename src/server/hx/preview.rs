use crate::db;
use crate::server::Config;

use super::render::{Response, ServerError, TemplatedResponse};
use super::Media;
use super::State;
use actix_web::web::Data;
use actix_web::web::{self, Query};
use actix_web::{delete, get, put};
use askama::Template;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "preview/preview.html")]
struct HxRoot {
    archive_confirm: bool,
    archive_allow: bool,
    favorite_allow: bool,
    prev: Option<Media>,
    media: Option<Media>,
    next: Option<Media>,
}

#[get("/{idx}")]
async fn root(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
    state: Query<State>,
    uuid: web::Path<Uuid>,
) -> Response {
    let res = db::media_get(&pool, &*state, db::Order::Desc, &uuid)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive_confirm: false,
        archive_allow: config.archive_allow,
        favorite_allow: config.favorite_allow,
        prev: res.0.map(|m| (m, &config.pathfinder).into()),
        media: res.1.map(|m| (m, &config.pathfinder).into()),
        next: res.2.map(|m| (m, &config.pathfinder).into()),
    }
    .render_response()
}

#[put("/{idx}/favorite/{value}")]
async fn favorite(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
    state: Query<State>,
    path: web::Path<(Uuid, bool)>,
) -> Response {
    let (uuid, value) = path.into_inner();

    db::media_favorite(&pool, &uuid, value)
        .await
        .map_err(ServerError::DBError)?;

    let res = db::media_get(&pool, &*state, db::Order::Desc, &uuid)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive_confirm: false,
        archive_allow: config.archive_allow,
        favorite_allow: config.favorite_allow,
        prev: res.0.map(|m| (m, &config.pathfinder).into()),
        media: res.1.map(|m| (m, &config.pathfinder).into()),
        next: res.2.map(|m| (m, &config.pathfinder).into()),
    }
    .render_response()
}

#[delete("/{idx}")]
async fn archive(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
    state: Query<State>,
    uuid: web::Path<Uuid>,
) -> Response {
    let res = db::media_get(&pool, &*state, db::Order::Desc, &uuid)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive_confirm: true,
        archive_allow: config.archive_allow,
        favorite_allow: config.favorite_allow,
        prev: res.0.map(|m| (m, &config.pathfinder).into()),
        media: res.1.map(|m| (m, &config.pathfinder).into()),
        next: res.2.map(|m| (m, &config.pathfinder).into()),
    }
    .render_response()
}

#[delete("/{idx}/confirm")]
async fn archive_confirm(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
    uuid: web::Path<Uuid>,
) -> Response {
    db::media_archive(&pool, &uuid, true)
        .await
        .map_err(ServerError::DBError)?;

    HxRoot {
        archive_confirm: false,
        archive_allow: config.archive_allow,
        favorite_allow: config.favorite_allow,
        prev: None,
        media: None,
        next: None,
    }
    .render_response()
}
