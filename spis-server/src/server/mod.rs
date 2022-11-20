use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use spis_model::Image;
use sqlx::{Pool, Sqlite};

use crate::{
    db::{self, ImgRow},
    SpisCfg,
};

trait ImgConvert {
    fn into(self, config: &SpisCfg) -> Image;
}

impl ImgConvert for ImgRow {
    fn into(self, config: &SpisCfg) -> Image {
        Image {
            uuid: self.id.to_string(),
            taken_at: self.taken_at,
            image: config.api_image(&self.image),
            thumbnail: config.api_thumbnail(&self.id),
        }
    }
}

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn images(
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<SpisCfg>,
    params: web::Query<spis_model::ImageSeachParams>,
) -> actix_web::Result<impl Responder> {
    let images: Vec<Image> = db::image_get(&pool, params.page_size as i32, params.taken_after)
        .await
        .unwrap()
        .into_iter()
        .map(|i| ImgConvert::into(i, &config))
        .collect();

    Ok(web::Json(images))
}

pub fn run(
    listener: TcpListener,
    pool: Pool<Sqlite>,
    config: SpisCfg,
) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);
    let config = web::Data::new(config);

    let server = HttpServer::new(move || {
        App::new()
            .route("/api/health", web::get().to(health))
            .route("/api", web::get().to(images))
            .app_data(pool.clone())
            .app_data(config.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
