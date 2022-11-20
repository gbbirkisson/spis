use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use spis_model::Media;
use sqlx::{Pool, Sqlite};

use crate::{
    db::{self, MediaRow},
    SpisCfg,
};

trait MediaConvert {
    fn into(self, config: &SpisCfg) -> Media;
}

impl MediaConvert for MediaRow {
    fn into(self, config: &SpisCfg) -> Media {
        Media {
            uuid: self.id.to_string(),
            taken_at: self.taken_at,
            location: config.api_media_location(&self.media),
            thumbnail: config.api_thumbnail(&self.id),
        }
    }
}

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn get_media(
    pool: web::Data<Pool<Sqlite>>,
    config: web::Data<SpisCfg>,
    params: web::Query<spis_model::MediaSearchParams>,
) -> actix_web::Result<impl Responder> {
    let media: Vec<Media> = db::media_get(&pool, params.page_size as i32, params.taken_after)
        .await
        .unwrap()
        .into_iter()
        .map(|i| MediaConvert::into(i, &config))
        .collect();

    Ok(web::Json(media))
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
            .route("/api", web::get().to(get_media))
            .app_data(pool.clone())
            .app_data(config.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
