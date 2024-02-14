use super::Cursor;
use super::Media;
use super::ServerError;
use super::State;
use super::TemplatedResponse;
use actix_web::get;
use actix_web::web::Data;
use actix_web::web::Query;
use actix_web::HttpResponse;
use askama::Template;
use sqlx::Pool;
use sqlx::Sqlite;

const PAGE_SIZE: usize = 50;

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

pub(super) async fn render(pool: &Pool<Sqlite>, state: State) -> Result<HttpResponse, ServerError> {
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

    let media = (0..PAGE_SIZE)
        .map(|_| Media {
            uuid: "123123".into(),
            url: "http://stufur:1337/assets/media/tota_myndir/2018/20180723_183916.jpg".into(),
            thumbnail:
                "http://stufur:1337/assets/thumbnails/1601707f-b75e-3640-91e4-0c4331ec7f6e.webp"
                    .into(),
            favorite: true,
            video: false,
            taken_at: chrono::offset::Utc::now(),
        })
        .collect();

    HxGallery {
        bar_buttons: &buttons,
        media: &media,
        state: &state,
    }
    .render_response()
}

#[get("")]
async fn root(pool: Data<Pool<Sqlite>>, state: Query<State>) -> Result<HttpResponse, ServerError> {
    render(&pool, state.into_inner()).await
}

#[derive(Template)]
#[template(path = "gallery/list.html")]
struct HxMore<'a> {
    media: &'a Vec<Media>,
}

#[get("/more")]
async fn more(_state: Query<State>, _cursor: Query<Cursor>) -> Result<HttpResponse, ServerError> {
    let media = (0..PAGE_SIZE)
        .map(|_| Media {
            uuid: "123123".into(),
            url: "http://stufur:1337/assets/media/tota_myndir/2018/20180723_183916.jpg".into(),
            thumbnail:
                "http://stufur:1337/assets/thumbnails/1601707f-b75e-3640-91e4-0c4331ec7f6e.webp"
                    .into(),
            favorite: true,
            video: false,
            taken_at: chrono::offset::Utc::now(),
        })
        .collect();

    HxMore { media: &media }.render_response()
}
