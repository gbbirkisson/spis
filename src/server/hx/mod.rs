use actix_web::http::header::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::web::scope;
use actix_web::{get, ResponseError};
use actix_web::{HttpResponse, HttpResponseBuilder};
use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod bar;
mod gallery;
mod preview;

struct Media {
    uuid: String,
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

#[derive(Error, Debug)]
enum ServerError {
    #[error("templating failed: {0}")]
    TemplateError(color_eyre::eyre::Error),
    #[error("db operation failed: {0}")]
    DBError(color_eyre::eyre::Error),
}

impl ResponseError for ServerError {}

trait TemplatedResponse {
    fn render_response(&self) -> Result<HttpResponse, ServerError>;
}

impl<T: askama::Template> TemplatedResponse for T {
    fn render_response(&self) -> Result<HttpResponse, ServerError> {
        Ok(HttpResponseBuilder::new(StatusCode::OK)
            .content_type(HeaderValue::from_static(T::MIME_TYPE))
            .body(
                self.render()
                    .map_err(|e| ServerError::TemplateError(e.into()))?,
            ))
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
async fn index() -> Result<HttpResponse, ServerError> {
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
