use crate::{db, PathFinder};

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
struct HxRoot<'a> {
    media: Option<&'a Media>,
}

#[get("/{idx}")]
async fn root(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
    uuid: web::Path<Uuid>,
) -> Response {
    let res = db::media_get(
        &pool,
        db::Filter {
            archived: false,
            favorite: state.favorite,
            taken_after: None,
            taken_before: None,
        },
        db::Order::Desc,
        &uuid,
    )
    .await
    .map_err(ServerError::DBError)?;

    // TODO:
    let media = res.1.map(|m| (m, pathfinder.as_ref()).into());
    HxRoot {
        media: media.as_ref(),
    }
    .render_response()
}

#[put("/{idx}/favorite")]
async fn favorite(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
    uuid: web::Path<Uuid>,
) -> Response {
    db::media_favorite(&pool, &uuid, true)
        .await
        .map_err(ServerError::DBError)?;

    let res = db::media_get(
        &pool,
        db::Filter {
            archived: false,
            favorite: state.favorite,
            taken_after: None,
            taken_before: None,
        },
        db::Order::Desc,
        &uuid,
    )
    .await
    .map_err(ServerError::DBError)?;

    // TODO:
    let media = res.1.map(|m| (m, pathfinder.as_ref()).into());
    HxRoot {
        media: media.as_ref(),
    }
    .render_response()
}

#[delete("/{idx}")]
async fn archive(pool: Data<Pool<Sqlite>>, uuid: web::Path<Uuid>) -> Response {
    db::media_archive(&pool, &uuid, true)
        .await
        .map_err(ServerError::DBError)?;
    HxRoot { media: None }.render_response()
}
