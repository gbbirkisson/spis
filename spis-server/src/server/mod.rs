use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use sqlx::{Pool, Sqlite};

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn images(_: web::Data<Pool<Sqlite>>) -> actix_web::Result<impl Responder> {
    let images: Vec<spis_model::Image> = vec![];
    Ok(web::Json(images))
}

pub fn run(listener: TcpListener, pool: Pool<Sqlite>) -> Result<Server, std::io::Error> {
    let pool = web::Data::new(pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/api/health", web::get().to(health))
            .route("/api", web::get().to(images))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
