use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use thiserror::Error;

pub(super) type RenderResult = Result<Html<String>, ServerError>;

#[derive(Error, Debug)]
pub(super) enum ServerError {
    #[error("templating failed: {0}")]
    TemplateError(color_eyre::eyre::Error),
    #[error("db operation failed: {0}")]
    DBError(color_eyre::eyre::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            Self::TemplateError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {e}"),
            )
                .into_response(),
            Self::DBError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            )
                .into_response(),
        }
    }
}

pub(super) trait TemplatedResponse {
    fn render_response(&self) -> RenderResult;
}

impl<T: askama::Template> TemplatedResponse for T {
    fn render_response(&self) -> RenderResult {
        Ok(Html(
            self.render()
                .map_err(|e| ServerError::TemplateError(e.into()))?,
        ))
    }
}
