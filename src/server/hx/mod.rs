use actix_web::get;
use actix_web::web::scope;
use askama::Template;
use chrono::{DateTime, Utc};
use render::{Response, TemplatedResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{PathFinder, db::Filter, db::MediaRow, db::Order};

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
    new_to_old: Option<bool>,
}

fn to_timestamp(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .expect("malformed timestamp")
        .with_timezone(&Utc)
}

impl From<&State> for Filter {
    fn from(value: &State) -> Self {
        let (start, end) = value.year.map_or((None, None), |year| {
            let start = to_timestamp(&format!(
                "{}-{:02}-01T00:00:00-00:00",
                year,
                value.month.unwrap_or(1)
            ));

            let next_year = format!("{}-01-01T00:00:00-00:00", year + 1);
            let end = to_timestamp(&match value.month {
                None => next_year,
                Some(month) => {
                    if month == 12 {
                        next_year
                    } else {
                        format!("{}-{:02}-01T00:00:00-00:00", year, month + 1,)
                    }
                }
            });
            (Some(start), Some(end))
        });

        let favorite = if matches!(value.favorite, Some(true)) {
            Some(true)
        } else {
            None
        };

        Self {
            archived: false,
            favorite,
            taken_after: start,
            taken_before: end,
        }
    }
}

impl From<&State> for Order {
    fn from(value: &State) -> Self {
        match value.new_to_old {
            Some(true) | None => Self::Desc,
            Some(false) => Self::Asc,
        }
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

const fn dev_enabled() -> bool {
    #[cfg(feature = "dev")]
    let dev = true;
    #[cfg(not(feature = "dev"))]
    let dev = false;
    dev
}

#[derive(Template)]
#[template(path = "web/index.html")]
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
                .service(bar::order)
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
