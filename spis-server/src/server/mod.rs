use self::convert::MediaConverter;
use crate::db;
use actix_web::{dev::Server, web, App, HttpServer, Responder, ResponseError};
use color_eyre::{
    eyre::{eyre, Context},
    Report, Result,
};
use derive_more::{Display, Error};
use spis_model::Media;
use sqlx::{Pool, Sqlite};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub mod convert;

#[cfg(feature = "release")]
static GUI: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../spis-gui/dist");

#[derive(Debug, Display, Error)]
#[display(fmt = "{msg} {cause}")]
struct ServerError {
    msg: &'static str,
    cause: Report,
}

impl ResponseError for ServerError {}

#[cfg(feature = "release")]
fn find_gui_files(name: &str) -> Vec<&include_dir::File> {
    GUI.find(name)
        .unwrap_or_else(|_| panic!("Could not find {}", name))
        .map(|f| {
            f.as_file()
                .unwrap_or_else(|| panic!("Could not convert to file: {}", name))
        })
        .collect()
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
        i32::try_from(params.page_size)
            .wrap_err("cast failure")
            .map_err(|e| ServerError {
                msg: "failed to cast page size",
                cause: e,
            })?,
        params.archived.unwrap_or(false),
        params.favorite,
        params.taken_after,
        params.taken_before,
    )
    .await
    .map_err(|e| ServerError {
        msg: "failed to list media",
        cause: e,
    })?
    .into_iter()
    .map(|m| converter.convert(&m))
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
        change = change
            || db::media_archive(&pool, uuid, archive)
                .await
                .map_err(|e| ServerError {
                    msg: "failed to archive media",
                    cause: e,
                })?;
    }
    if let Some(favorite) = params.favorite {
        change = change
            || db::media_favorite(&pool, uuid, favorite)
                .await
                .map_err(|e| ServerError {
                    msg: "failed to favorite media",
                    cause: e,
                })?;
    }

    Ok(web::Json(change))
}

pub enum Listener {
    Tcp(TcpListener),
    Socket(String),
}

#[allow(clippy::missing_errors_doc)]
pub fn run(listener: Listener, pool: Pool<Sqlite>, converter: MediaConverter) -> Result<Server> {
    let pool = web::Data::new(pool);
    let converter = web::Data::new(converter);

    let server = HttpServer::new(move || {
        #[allow(unused_mut)]
        let mut app = App::new().wrap(TracingLogger::default());
        #[cfg(feature = "release")]
        {
            let files = vec![
                ("*.html", "text/html"),
                ("*.json", "application/json"),
                ("*.js", "application/javascript"),
                ("*.wasm", "application/wasm"),
                ("*.png", "image/png"),
                ("*.css", "text/css"),
                ("*.ttf", "font/ttf"),
                ("*.woff2", "font/woff2"),
            ];

            for (file_regex, content_type) in files {
                for file in find_gui_files(file_regex) {
                    let mut file_path = format!("/{}", file.path().to_str().unwrap());
                    if file_path == "/index.html" {
                        file_path = "/".to_string();
                    }
                    app = app.route(
                        &file_path,
                        web::get().to(move || async move { create_gui_route(content_type, file) }),
                    );
                }
            }
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
