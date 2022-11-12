use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

pub mod db;
pub mod img;

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn images() -> actix_web::Result<impl Responder> {
    let images: Vec<spis_model::Image> = vec![];
    Ok(web::Json(images))
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .route("/api/health", web::get().to(health))
            .route("/api", web::get().to(images))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
