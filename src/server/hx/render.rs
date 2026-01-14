use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use thiserror::Error;

pub(super) type RenderResult = Result<Html<String>, ServerError>;

#[derive(Error, Debug)]
pub(super) enum ServerError {
    #[error("templating failed: {0}")]
    Template(color_eyre::eyre::Error),
    #[error("db operation failed: {0}")]
    DB(color_eyre::eyre::Error),
    #[error("command dispatch failed: {0}")]
    CommandDispatch(color_eyre::eyre::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            Self::Template(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Template error: {e}"),
            )
                .into_response(),
            Self::DB(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {e}"),
            )
                .into_response(),
            Self::CommandDispatch(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Command dispatch error: {e}"),
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
            self.render().map_err(|e| ServerError::Template(e.into()))?,
        ))
    }
}
