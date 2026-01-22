use crate::server::AppState;

use super::render::ServerError;
use super::{GalleryState, Media};
use crate::server::commands::CustomCommandTrigger;
use crate::{MediaEvent, db};
use axum::extract::{Json, Path, State};
use axum::{Router, routing::post};
use uuid::Uuid;

#[allow(clippy::collapsible_if)]
async fn archive(
    State(app_state): State<AppState>,
    Json(uuids): Json<Vec<Uuid>>,
) -> Result<(), ServerError> {
    let pool = &app_state.pool;
    let config = &app_state.config;

    for uuid in uuids {
        if config.features.delete_on_archive {
            if let Some(path) = db::media_get_path(pool, &uuid)
                .await
                .map_err(ServerError::DB)?
            {
                if let Err(e) = std::fs::remove_file(&path) {
                    tracing::error!("Failed to delete file {}: {}", path, e);
                } else {
                    tracing::info!("Deleted file: {}", path);
                }
            }
        }

        db::media_archive(pool, &uuid, true)
            .await
            .map_err(ServerError::DB)?;

        let _ = app_state.media_events.send(MediaEvent::Archived(uuid));
    }

    Ok(())
}

async fn command(
    State(app_state): State<AppState>,
    Path(cmd): Path<String>,
    Json(uuids): Json<Vec<Uuid>>,
) -> Result<(), ServerError> {
    let pool = &app_state.pool;
    let config = &app_state.config;

    // Validate command exists
    if !config
        .features
        .custom_commands
        .iter()
        .any(|c| c.name == cmd)
    {
        return Ok(());
    }

    for uuid in uuids {
        // We need to fetch the media to pass to the command runner
        // Using media_get with empty query/state is a bit heavy but works
        // Ideally we'd have a lighter `media_get_by_uuid`
        let (_, media, _) = db::media_get(
            pool,
            &GalleryState::default(),
            &GalleryState::default(),
            &uuid,
        )
        .await
        .map_err(ServerError::DB)?;

        if let Some(media) = media {
            let media: Media = (media, &config.pathfinder).into();
            app_state
                .cmd_runner
                .send(CustomCommandTrigger {
                    cmd: cmd.clone(),
                    media,
                })
                .await
                .map_err(|e| ServerError::CommandDispatch(e.into()))?;
        }
    }

    Ok(())
}

#[allow(clippy::collapsible_if)]
async fn favorite(
    State(app_state): State<AppState>,
    Json(uuids): Json<Vec<Uuid>>,
) -> Result<(), ServerError> {
    let pool = &app_state.pool;

    // Heuristic: If any in the selection is NOT a favorite, make all favorites.
    // Otherwise (all are favorites), make all NOT favorites.
    let mut make_fav = false;

    // Check state of selected items
    for uuid in &uuids {
        let (_, media, _) = db::media_get(
            pool,
            &GalleryState::default(),
            &GalleryState::default(),
            uuid,
        )
        .await
        .map_err(ServerError::DB)?;

        if let Some(media) = media {
            if !media.favorite {
                make_fav = true;
                break;
            }
        }
    }

    for uuid in uuids {
        db::media_favorite(pool, &uuid, make_fav)
            .await
            .map_err(ServerError::DB)?;
        let _ = app_state.media_events.send(MediaEvent::Changed(uuid));
    }

    Ok(())
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/archive", post(archive))
        .route("/command/{name}", post(command))
        .route("/favorite", post(favorite))
}
