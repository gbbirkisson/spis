use crate::db::{self};
use actix_web::{dev::Server, web, App, HttpServer, Responder};
use color_eyre::{eyre::eyre, Result};
use spis_model::Media;
use sqlx::{Pool, Sqlite};
use std::net::TcpListener;

use self::convert::MediaConverter;

pub mod convert;

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
fn create_gui_route(
    content_type: &str,
    file: &'static include_dir::File,
) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(file.contents())
}

async fn media_list(
    pool: web::Data<Pool<Sqlite>>,
    converter: web::Data<MediaConverter>,
    params: web::Query<spis_model::MediaListParams>,
) -> actix_web::Result<impl Responder> {
    let media: Vec<Media> = db::media_get(
        &pool,
        params.page_size as i32,
        params.archived.unwrap_or(false),
        params.taken_after,
        params.taken_before,
    )
    .await
    .unwrap()
    .into_iter()
    .map(|m| converter.convert(m))
    .collect();

    Ok(web::Json(media))
}

async fn media_edit(
    pool: web::Data<Pool<Sqlite>>,
    path: web::Path<uuid::Uuid>,
    params: web::Query<spis_model::MediaEditParams>,
) -> actix_web::Result<impl Responder> {
    let mut change = false;

    let uuid = path.as_ref();
    if let Some(archive) = params.archive {
        change = db::media_archive(&pool, uuid, archive).await.unwrap();
    }

    Ok(web::Json(change))
}

pub enum Listener {
    Tcp(TcpListener),
    Socket(String),
}

pub fn run(listener: Listener, pool: Pool<Sqlite>, converter: MediaConverter) -> Result<Server> {
    let pool = web::Data::new(pool);
    let converter = web::Data::new(converter);

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
        app.route("/api", web::get().to(media_list))
            .route("/api/{uuid}", web::post().to(media_edit))
            .app_data(pool.clone())
            .app_data(converter.clone())
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
