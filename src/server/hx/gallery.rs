use crate::db;
use crate::PathFinder;

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
    media: &'a Vec<Media>,
    state: &'a State,
}

pub(super) async fn render(pool: &Pool<Sqlite>, pathfinder: &PathFinder, state: State) -> Response {
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

    let media = db::media_get(
        &pool,
        PAGE_SIZE.try_into().expect("PAGE_SIZE conversion failed"),
        false,
        state.favorite,
        None, // TODO
        None, // TODO
    )
    .await
    .map_err(ServerError::DBError)?
    .into_iter()
    .map(|row| (row, pathfinder).into())
    .collect();

    HxGallery {
        bar_buttons: &buttons,
        media: &media,
        state: &state,
    }
    .render_response()
}

#[get("")]
async fn root(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
) -> Response {
    render(&pool, &pathfinder, state.into_inner()).await
}

#[derive(Template)]
#[template(path = "gallery/list.html")]
struct HxMore<'a> {
    media: &'a Vec<Media>,
}

#[get("/more")]
async fn more(
    pool: Data<Pool<Sqlite>>,
    pathfinder: Data<PathFinder>,
    state: Query<State>,
    cursor: Query<Cursor>,
) -> Response {
    let media = db::media_get(
        &pool,
        PAGE_SIZE.try_into().expect("PAGE_SIZE conversion failed"),
        false,
        state.favorite,
        None, // TODO
        Some(cursor.cursor),
    )
    .await
    .map_err(ServerError::DBError)?
    .into_iter()
    .map(|row| (row, pathfinder.as_ref()).into())
    .collect();

    HxMore { media: &media }.render_response()
}
