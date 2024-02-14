use actix_web::get;
use actix_web::web::scope;
use askama::Template;
use chrono::{DateTime, Utc};
use render::{Response, TemplatedResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod bar;
mod gallery;
mod preview;
mod render;

struct Media {
    uuid: Uuid,
    url: String,
    thumbnail: String,
    favorite: bool,
    video: bool,
    taken_at: DateTime<Utc>,
}

#[derive(Deserialize, Default, Debug)]
struct State {
    favorite: Option<bool>,
    year: Option<usize>,
    month: Option<u8>,
}

#[derive(Deserialize, Serialize)]
struct Cursor {
    cursor: DateTime<Utc>,
}

fn dev_enabled() -> bool {
    #[cfg(feature = "dev")]
    let dev = true;
    #[cfg(not(feature = "dev"))]
    let dev = false;
    dev
}

#[derive(Template)]
#[template(path = "index.html")]
struct HxIndex {}

#[get("")]
async fn index() -> Response {
    HxIndex {}.render_response()
}

pub fn create_service(path: &str) -> actix_web::Scope {
    scope(path)
        .service(index)
        .service(
            scope("/gallery")
                .service(gallery::root)
                .service(gallery::more),
        )
        .service(
            scope("/bar")
                .service(bar::favorite)
                .service(bar::year)
                .service(bar::month)
                .service(bar::clear),
        )
        .service(
            scope("/preview")
                .service(preview::root)
                .service(preview::favorite)
                .service(preview::archive),
        )
}
