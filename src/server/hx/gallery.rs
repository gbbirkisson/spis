use crate::db;
use crate::server::{Config, Features};
use chrono::Datelike;

use super::Cursor;
use super::Media;
use super::State;
use super::render::ServerError;
use super::render::{Response, TemplatedResponse};
use actix_web::get;
use actix_web::web::Data;
use actix_web::web::Query;
use askama::Template;
use sqlx::Pool;
use sqlx::Sqlite;

const PAGE_SIZE: usize = 200;

mod filters {
    use core::fmt;

    use super::super::{Cursor, Media, gallery::PAGE_SIZE};

    const EMPTY: String = String::new();

    pub fn cursor(media: &[Media], _: &dyn askama::Values) -> ::askama::Result<String> {
        if media.len() < PAGE_SIZE {
            Ok(EMPTY)
        } else {
            Ok(serde_urlencoded::to_string(Cursor {
                cursor: media.last().expect("List should not be empty").taken_at,
            })
            .map_err(|_| fmt::Error)?)
        }
    }
}

enum BarButton {
    Favorite(bool),
    Year(bool, String),
    Month(bool, u8, String),
    Order(bool),
    Clear,
    Empty,
}

#[derive(Template)]
#[template(path = "web/gallery/gallery.html")]
struct HxGallery<'a> {
    bar_buttons: &'a Vec<BarButton>,
    features: &'a Features,
    media: &'a Vec<Media>,
    state: &'a State,
}

pub(super) async fn render(pool: &Pool<Sqlite>, config: &Config, state: State) -> Response {
    let now = chrono::Utc::now();

    #[allow(clippy::cast_sign_loss)]
    let current_year = now.year() as usize;
    #[allow(clippy::cast_possible_truncation)]
    let current_month = now.month() as u8;

    let new_to_old = state.new_to_old.unwrap_or(true);

    // let mut buttons = vec![BarButton::Favorite(state.favorite.unwrap_or(false))];
    let mut buttons = Vec::with_capacity(18);
    // vec![];
    // buttons.push(BarButton::Order(new_to_old));

    if state.year.is_none() {
        for i in (current_year - 14..=current_year).rev() {
            buttons.push(BarButton::Year(false, format!("{i}")));
        }
    } else if let Some(year) = state.year {
        if year == current_year {
            buttons.push(BarButton::Empty);
        } else {
            buttons.push(BarButton::Year(false, format!("{}", year + 1)));
        }

        if new_to_old {
            buttons.push(BarButton::Year(true, format!("{year}")));
        }

        for (month_nr, month_text) in vec![
            (12, "Dec"),
            (11, "Nov"),
            (10, "Oct"),
            (9, "Sep"),
            (8, "Aug"),
            (7, "Jul"),
            (6, "Jun"),
            (5, "May"),
            (4, "Apr"),
            (3, "Mar"),
            (2, "Feb"),
            (1, "Jan"),
        ] {
            if year == current_year && month_nr > current_month {
                buttons.push(BarButton::Empty);
            } else {
                buttons.push(BarButton::Month(
                    Some(month_nr) == state.month,
                    month_nr,
                    (*month_text).to_string(),
                ));
            }
        }

        if !new_to_old {
            buttons.push(BarButton::Year(true, format!("{year}")));
        }

        buttons.push(BarButton::Year(false, format!("{}", year - 1)));
    }

    if !new_to_old {
        buttons = buttons.into_iter().rev().collect();
    }

    buttons.insert(0, BarButton::Favorite(state.favorite.unwrap_or(false)));
    buttons.push(BarButton::Order(new_to_old));

    match (state.favorite, state.year) {
        (Some(true), _) | (_, Some(_)) => buttons.push(BarButton::Clear),
        (_, _) => buttons.push(BarButton::Empty),
    }

    let media = db::media_list(pool, &state, &state, PAGE_SIZE)
        .await
        .map_err(ServerError::DBError)?
        .into_iter()
        .map(|row| (row, &config.pathfinder).into())
        .collect();

    HxGallery {
        bar_buttons: &buttons,
        features: &config.features,
        media: &media,
        state: &state,
    }
    .render_response()
}

#[get("")]
async fn root(pool: Data<Pool<Sqlite>>, config: Data<Config>, state: Query<State>) -> Response {
    render(&pool, &config, state.into_inner()).await
}

#[derive(Template)]
#[template(path = "web/gallery/list.html")]
struct HxMore<'a> {
    features: &'a Features,
    media: &'a Vec<Media>,
}

#[get("/more")]
async fn more(
    pool: Data<Pool<Sqlite>>,
    config: Data<Config>,
    state: Query<State>,
    cursor: Query<Cursor>,
) -> Response {
    let media = db::media_list(&pool, (&*state, &*cursor), &*state, PAGE_SIZE)
        .await
        .map_err(ServerError::DBError)?
        .into_iter()
        .map(|row| (row, &config.pathfinder).into())
        .collect();

    HxMore {
        features: &config.features,
        media: &media,
    }
    .render_response()
}
