use super::GalleryState;
use crate::MediaEvent;
use crate::db;
use crate::server::AppState;
use crate::server::hx::Media;
use askama::Template;
use axum::{
    Router,
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use futures_util::stream::{Stream, StreamExt as _};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Template)]
#[template(path = "web/gallery/media.html")]
struct HxMedia<'a> {
    features: &'a crate::server::Features,
    media: &'a Media,
    oob: bool,
}

async fn convert_event(
    app_state: AppState,
    state: GalleryState,
    event: MediaEvent,
) -> Option<Event> {
    tracing::debug!("Got media event for SSE: {:?}", event);
    match event {
        MediaEvent::Added(uuid) => {
            if let Ok((_, media, after)) =
                db::media_get(&app_state.pool, &state, &state, &uuid).await
                && let Some(media) = media
                && let Some(after) = after
            {
                let media: Media = (media, &app_state.config.pathfinder).into();
                let tmpl = HxMedia {
                    features: &app_state.config.features,
                    media: &media,
                    oob: false,
                };

                if let Ok(res) = tmpl.render() {
                    return Some(Event::default().data(format!(
                        r#"<div hx-swap-oob="beforebegin:#thumb-{}">{res}</div>"#,
                        after.id
                    )));
                }
            }
            None
        }
        MediaEvent::Changed(uuid) => {
            if let Ok((_, media, _)) = db::media_get(&app_state.pool, &state, &state, &uuid).await
                && let Some(media) = media
            {
                let media: Media = (media, &app_state.config.pathfinder).into();
                let tmpl = HxMedia {
                    features: &app_state.config.features,
                    media: &media,
                    oob: true,
                };
                if let Ok(res) = tmpl.render() {
                    return Some(Event::default().data(res));
                }
            }
            None
        }
        MediaEvent::Archived(uuid) => Some(Event::default().data(format!(
            r#"<li id="thumb-{uuid}" hx-swap-oob="delete"></li>"#
        ))),
    }
}

async fn handler(
    State(app_state): State<AppState>,
    Query(state): Query<GalleryState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = app_state.media_events.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| std::future::ready(result.ok()))
        .then(move |event| convert_event(app_state.clone(), state.clone(), event))
        .filter_map(std::future::ready)
        .map(Ok);

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub fn create_router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
