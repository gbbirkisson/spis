use crate::db;
use crate::server::{Config, Features};

use super::render::ServerError;
use super::render::{Response, TemplatedResponse};
use super::Cursor;
use super::Media;
use super::State;
use actix_web::get;
use actix_web::web::Data;
use actix_web::web::Query;
use askama::Template;
use sqlx::Pool;
use sqlx::Sqlite;

const PAGE_SIZE: usize = 20;

mod filters {
    use core::fmt;

    use super::super::{gallery::PAGE_SIZE, Cursor, Media};

    const EMPTY: String = String::new();

    pub fn cursor(media: &Vec<Media>) -> ::askama::Result<String> {
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
    Clear,
    Empty,
}

#[derive(Template)]
#[template(path = "gallery/gallery.html")]
struct HxGallery<'a> {
    bar_buttons: &'a Vec<BarButton>,
    features: &'a Features,
    media: &'a Vec<Media>,
    state: &'a State,
}

pub(super) async fn render(pool: &Pool<Sqlite>, config: &Config, state: State) -> Response {
    let mut buttons = vec![BarButton::Favorite(state.favorite.unwrap_or(false))];
    let current_year = 2024;
    if state.year.is_none() {
        for i in (current_year - 14..=current_year).rev() {
            buttons.push(BarButton::Year(false, format!("{i}")));
        }
        buttons.push(BarButton::Empty);
    } else if let Some(year) = state.year {
        if year == current_year {
            buttons.push(BarButton::Empty);
            buttons.push(BarButton::Empty);
        } else {
            buttons.push(BarButton::Year(false, format!("{}", year + 1)));
            buttons.push(BarButton::Year(true, format!("{year}")));
        }

        for (month_nr, month_text) in vec![
            (1, "Jan"),
            (2, "Feb"),
            (3, "Mar"),
            (4, "Apr"),
            (5, "May"),
            (6, "Jun"),
            (7, "Jul"),
            (8, "Aug"),
            (9, "Sep"),
            (10, "Oct"),
            (11, "Nov"),
            (12, "Dec"),
        ]
        .iter()
        .rev()
        {
            buttons.push(BarButton::Month(
                Some(month_nr) == state.month.as_ref(),
                *month_nr,
                (*month_text).to_string(),
            ));
        }

        buttons.push(BarButton::Year(false, format!("{}", year - 1)));
        buttons.push(BarButton::Clear);
    }

    let media = db::media_list(
        pool,
        &state,
        db::Order::Desc,
        PAGE_SIZE.try_into().expect("PAGE_SIZE conversion failed"),
    )
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
#[template(path = "gallery/list.html")]
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
    let media = db::media_list(
        &pool,
        (&*state, &*cursor),
        db::Order::Desc,
        PAGE_SIZE.try_into().expect("PAGE_SIZE conversion failed"),
    )
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
