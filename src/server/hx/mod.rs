use actix_web::get;
use actix_web::web::scope;
use askama::Template;
use chrono::{DateTime, Utc};
use render::{Response, TemplatedResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::Filter, db::MediaRow, db::Order, PathFinder};

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

impl From<(MediaRow, &PathFinder)> for Media {
    fn from(value: (MediaRow, &PathFinder)) -> Self {
        Self {
            uuid: value.0.id,
            url: value.1.media(&value.0.path),
            thumbnail: value.1.thumbnail(&value.0.id),
            favorite: value.0.favorite,
            video: value.0.media_type == 1,
            taken_at: value.0.taken_at,
        }
    }
}

#[derive(Deserialize, Default, Debug)]
struct State {
    favorite: Option<bool>,
    year: Option<usize>,
    month: Option<u8>,
}

impl From<&State> for Filter {
    fn from(value: &State) -> Self {
        // https://github.com/gbbirkisson/spis/blob/main/spis-gui/src/filters.rs#L92
        Self {
            archived: false,
            favorite: value.favorite,
            taken_after: None,  // TODO:
            taken_before: None, // TODO:
        }
    }
}

impl From<&State> for Order {
    fn from(_value: &State) -> Self {
        Self::Desc // TODO:
    }
}

#[derive(Deserialize, Serialize)]
struct Cursor {
    cursor: DateTime<Utc>,
}

impl From<(&State, &Cursor)> for Filter {
    fn from(value: (&State, &Cursor)) -> Self {
        let mut filter: Self = value.0.into();
        let order: Order = value.0.into();
        match order {
            Order::Asc => filter.taken_after = Some(value.1.cursor),
            Order::Desc => filter.taken_before = Some(value.1.cursor),
        }
        filter
    }
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
                .service(preview::archive)
                .service(preview::archive_confirm),
        )
}
