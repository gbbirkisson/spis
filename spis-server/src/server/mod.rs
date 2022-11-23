use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use eyre::{eyre, Result};
use spis_model::Media;
use sqlx::{Pool, Sqlite};

use crate::{
    db::{self, MediaRow},
    SpisCfg,
};

#[cfg(feature = "release")]
static GUI: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../spis-gui/dist");

#[cfg(feature = "release")]
fn find_gui_file(name: &str) -> &include_dir::File {
    GUI.find(name)
        .unwrap_or_else(|_| panic!("Could not find {}", name))
        .next()
        .unwrap_or_else(|| panic!("Iterator has not file: {}", name))
        .as_file()
        .unwrap_or_else(|| panic!("Could not convert to file: {}", name))
}

#[cfg(feature = "release")]
fn create_gui_route(content_type: &str, file: &'static include_dir::File) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(content_type)
        .body(file.contents())
}

trait MediaConvert {
    fn into(self, config: &SpisCfg) -> Media;
}

impl MediaConvert for MediaRow {
    fn into(self, config: &SpisCfg) -> Media {
        Media {
            uuid: self.id.to_string(),
            taken_at: self.taken_at,
            location: config.api_media_location(&self.path),
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

pub enum Listener {
    Tcp(TcpListener),
    Socket(String),
}

pub fn run(listener: Listener, pool: Pool<Sqlite>, config: SpisCfg) -> Result<Server> {
    let pool = web::Data::new(pool);
    let config = web::Data::new(config);

    let server = HttpServer::new(move || {
        #[allow(unused_mut)]
        let mut app = App::new();

        #[cfg(feature = "release")]
        {
            let html_file = find_gui_file("*html");
            app = app.route(
                "/",
                web::get().to(move || async move { create_gui_route("text/html", html_file) }),
            );

            let js_file = find_gui_file("*js");
            let js_path_name = format!("/{}", js_file.path().to_str().unwrap());
            app = app.route(
                &js_path_name,
                web::get()
                    .to(move || async move { create_gui_route("application/javascript", js_file) }),
            );

            let wasm_file = find_gui_file("*wasm");
            let wasm_path_name = format!("/{}", wasm_file.path().to_str().unwrap());
            app = app.route(
                &wasm_path_name,
                web::get()
                    .to(move || async move { create_gui_route("application/wasm", wasm_file) }),
            );

            let icon_file = find_gui_file("*png");
            app = app.route(
                "/favicon.png",
                web::get().to(move || async move { create_gui_route("image/png", icon_file) }),
            );

            let manifest_file = find_gui_file("*json");
            app = app.route(
                "/manifest.json",
                web::get()
                    .to(move || async move { create_gui_route("application/json", manifest_file) }),
            );
        };
        app.route("/api/health", web::get().to(health))
            .route("/api", web::get().to(get_media))
            .app_data(pool.clone())
            .app_data(config.clone())
    });

    let server = match listener {
        Listener::Tcp(listener) => server.listen(listener)?,
        Listener::Socket(file) => {
            if cfg!(not(unix)) {
                return Err(eyre!("You can only use unix sockets on unix!"));
            }
            server.bind_uds(file)?
        }
    }
    .run();

    Ok(server)
}
