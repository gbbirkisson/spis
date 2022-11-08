use std::{net::TcpListener, sync::Arc};

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use state::State;

mod state;

async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // let shared_state = Arc::new(State::load("dev/api/state", "dev/api/images"));

    let server = HttpServer::new(move || {
        App::new()
            // .app_data(shared_state.clone())
            .route("/api/health", web::get().to(health))
        // .route("/api", web::get().to(images))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

// async fn get_images(state: Extension<Arc<State>>) -> impl IntoResponse {
//     let images: Vec<Image> = state.images().values().cloned().collect::<Vec<Image>>();
//     (StatusCode::OK, Json(images))
// }
