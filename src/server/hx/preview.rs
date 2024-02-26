use std::str::FromStr;

use crate::db;

use super::render::{Response, ServerError, TemplatedResponse};
use super::Media;
use actix_web::web;
use actix_web::web::Data;
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
async fn root(_pool: Data<Pool<Sqlite>>, _uuid: web::Path<Uuid>) -> Response {
    let media = Media {
        uuid: Uuid::from_str("9be1c561-5245-42a3-af22-b0c77136665f").unwrap(),
        url: "http://stufur:1337/assets/media/tota_myndir/2018/20180723_183916.jpg".into(),
        thumbnail: "http://stufur:1337/assets/thumbnails/1601707f-b75e-3640-91e4-0c4331ec7f6e.webp"
            .into(),
        favorite: true,
        video: false,
        taken_at: chrono::offset::Utc::now(),
    };

    HxRoot {
        media: Some(&media),
    }
    .render_response()
}

#[put("/{idx}/favorite")]
async fn favorite(pool: Data<Pool<Sqlite>>, uuid: web::Path<Uuid>) -> Response {
    db::media_favorite(&pool, &uuid, true)
        .await
        .map_err(ServerError::DBError)?;

    let media = Media {
        uuid: Uuid::from_str("9be1c561-5245-42a3-af22-b0c77136665f").unwrap(),
        url: "http://stufur:1337/assets/media/tota_myndir/2018/20180723_183916.jpg".into(),
        thumbnail: "http://stufur:1337/assets/thumbnails/1601707f-b75e-3640-91e4-0c4331ec7f6e.webp"
            .into(),
        favorite: false,
        video: false,
        taken_at: chrono::offset::Utc::now(),
    };

    HxRoot {
        media: Some(&media),
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
