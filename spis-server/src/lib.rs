use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use state::State;

pub mod state;

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

async fn images(state: web::Data<State>) -> actix_web::Result<impl Responder> {
    let images: Vec<spis_model::Image> = state
        .images()
        .values()
        .cloned()
        .collect::<Vec<spis_model::Image>>();
    Ok(web::Json(images))
}

pub fn run(state: State, listener: TcpListener) -> Result<Server, std::io::Error> {
    let state = web::Data::new(state);

    let server = HttpServer::new(move || {
        App::new()
            .route("/api/health", web::get().to(health))
            .route("/api", web::get().to(images))
            .app_data(state.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
