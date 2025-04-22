use actix_web::ResponseError;
use actix_web::web::Html;
use thiserror::Error;

pub(super) type Response = Result<Html, ServerError>;

#[derive(Error, Debug)]
pub(super) enum ServerError {
    #[error("templating failed: {0}")]
    TemplateError(color_eyre::eyre::Error),
    #[error("db operation failed: {0}")]
    DBError(color_eyre::eyre::Error),
}

impl ResponseError for ServerError {}

pub(super) trait TemplatedResponse {
    fn render_response(&self) -> Response;
}

impl<T: askama::Template> TemplatedResponse for T {
    fn render_response(&self) -> Response {
        Ok(Html::new(
            self.render()
                .map_err(|e| ServerError::TemplateError(e.into()))?,
        ))
    }
}
