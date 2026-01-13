use crate::db;
use crate::server::AppState;
use chrono::Datelike;

use super::Cursor;
use super::GalleryState;
use super::Media;
use super::render::RenderResult;
use super::render::ServerError;
use super::render::TemplatedResponse;
use askama::Template;
use axum::extract::{Query, State};
use axum::{Router, routing::get};

const PAGE_SIZE: usize = 200;
const YEARS: usize = 14;

mod filters {
    use core::fmt;

    use super::super::{Cursor, Media, gallery::PAGE_SIZE};

    const EMPTY: String = String::new();

    #[askama::filter_fn]
    pub fn cursor(media: &[Media], _: &dyn askama::Values) -> askama::Result<String> {
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

#[derive(Debug)]
enum BarButton {
    Favorite(bool),
    Year(bool, String),
    Month(bool, u8, String),
    Order(bool),
    Collection(bool),
    Clear,
    Empty,
}

#[derive(Template)]
#[template(path = "web/gallery/gallery.html")]
struct HxGallery<'a> {
    bar_buttons: &'a Vec<BarButton>,
    features: &'a crate::server::Features,
    media: &'a Vec<Media>,
    state: &'a GalleryState,
}

pub(super) async fn render(app_state: &AppState, state: GalleryState) -> RenderResult {
    let now = chrono::Utc::now();
    let pool = &app_state.pool;
    let config = &app_state.config;

    #[allow(clippy::cast_sign_loss)]
    let current_year = now.year() as usize;
    #[allow(clippy::cast_possible_truncation)]
    let current_month = now.month() as u8;

    let new_to_old = state.new_to_old.unwrap_or(true);

    let mut buttons = Vec::with_capacity(18);

    if state.collection.is_none() {
        if state.year.is_none() {
            for i in (current_year - YEARS..=current_year).rev() {
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

    buttons.push(BarButton::Collection(state.collection.is_some()));

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

async fn root(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> RenderResult {
    render(&app_state, state).await
}

#[derive(Template)]
#[template(path = "web/gallery/list.html")]
struct HxMore<'a> {
    features: &'a crate::server::Features,
    media: &'a Vec<Media>,
}

async fn more(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
    Query(cursor): Query<Cursor>,
) -> RenderResult {
    let pool = &app_state.pool;
    let config = &app_state.config;
    let media = db::media_list(pool, (&state, &cursor), &state, PAGE_SIZE)
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

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/more", get(more))
}
