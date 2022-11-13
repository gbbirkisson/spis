use std::{net::TcpListener, path::PathBuf};

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use sqlx::{Pool, Sqlite};

use crate::{db, img::prelude::Thumbnail};

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn images(
    pool: web::Data<Pool<Sqlite>>,
    thumb_dir: web::Data<PathBuf>,
) -> actix_web::Result<impl Responder> {
    let res = db::image_get(&pool, 10, None).await.unwrap();
    let images: Vec<spis_model::Image> = res
        .iter()
        .map(|i| {
            let thumbnail = thumb_dir.get_thumbnail(&i.id).to_str().unwrap().to_string();
            spis_model::Image {
                hash: i.id.to_string(),
                path: i.image.clone(),
                thumbnail,
            }
        })
        .collect();
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
