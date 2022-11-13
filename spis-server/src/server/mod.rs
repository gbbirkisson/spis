use std::{net::TcpListener, path::PathBuf};

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::db;

#[derive(Deserialize)]
struct ImageParams {
    page_size: i32,
    prev: Option<DateTime<Utc>>,
}

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn images(
    pool: web::Data<Pool<Sqlite>>,
    thumb_dir: web::Data<PathBuf>,
    params: web::Query<ImageParams>,
) -> actix_web::Result<impl Responder> {
    let images = db::image_get(&pool, &thumb_dir, params.page_size, params.prev)
        .await
        .unwrap();
    Ok(web::Json(images))
}

pub fn run(
    listener: TcpListener,
    pool: Pool<Sqlite>,
    thumb_dir: PathBuf,
) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let thumb_dir = web::Data::new(thumb_dir);

    let server = HttpServer::new(move || {
        App::new()
            .route("/api/health", web::get().to(health))
            .route("/api", web::get().to(images))
            .app_data(pool.clone())
            .app_data(thumb_dir.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
