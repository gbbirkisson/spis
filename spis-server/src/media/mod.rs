use crate::db;
use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use image::DynamicImage;
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use tokio::sync::mpsc::channel;
use uuid::Uuid;

use self::{
    processing::{single_media_process, GetMediaType},
    video::VideoProcessor,
};

mod images;
mod processing;
pub(crate) mod util;
mod video;

pub struct ProcessedMedia {
    pub uuid: Uuid,
    pub path: String,
    pub media_type: ProcessedMediaType,
    pub data: Option<ProcessedMediaData>,
}

pub enum ProcessedMediaType {
    Image,
    Video,
}

pub struct ProcessedMediaData {
    pub taken_at: DateTime<Utc>,
}

pub async fn process(
    pool: Pool<Sqlite>,
    media_dir: PathBuf,
    thumb_dir: PathBuf,
    force_uuid_calculation: bool,
) {
    let start_time = Utc::now().time();
    tracing::info!("Media processing started");

    tracing::debug!("Mark entire database as unwalked");
    let mark = db::media_mark_unwalked(&pool).await;
    if mark.is_err() {
        tracing::error!("Failed marking media as unwalked: {:?}", &mark);
    }

    let old_entries = match force_uuid_calculation {
        true => None,
        false => Some(
            db::media_hashmap(&pool)
                .await
                .expect("Failed to fetch all entries"),
        ),
    };

    tracing::debug!("Setup processing channels and pool");
    let (tx, mut rx) = channel(1);
    let mut done_recv = processing::media_processor(media_dir, thumb_dir, old_entries, tx);
    let processor_pool = pool.clone();

    let mut count = 0;
    loop {
        tokio::select! {
            done = done_recv.recv() => {
                match done {
                    Some(count) => {
                        tracing::info!("Processed {} files in total", count);
                        break;
                    },
                    None => {
                        tracing::debug!("None from done channel");
                    }
                }
            }
            media = rx.recv() => {
                match media {
                    Some(media) => {
                        tracing::debug!("Inserting media {}", media.uuid);
                        if let Err(e) = db::media_insert(&processor_pool, media).await {
                            tracing::error!("Failed inserting media into DB: {e}");
                        } else {
                            count += 1;
                            if count % 500 == 0 {
                                tracing::info!("Processed {} files so far ...", count);
                            }
                        }
                    }
                    None => {
                        tracing::debug!("None from channel");
                    }
                }
            }
        }
    }

    tracing::debug!("Update missing field in DB");
    if mark.is_ok() {
        db::media_mark_missing(&pool)
            .await
            .expect("Failed to mark missing");
    }

    // TODO: Cleanup thumbnails?

    if let Ok(counts) = db::media_count(&pool).await {
        tracing::info!("DB counts total:    {}", counts.count);
        if let Some(c) = counts.walked {
            tracing::info!("DB counts walked:   {}", c);
        }
        if let Some(c) = counts.favorite {
            tracing::info!("DB counts favorite: {}", c);
        }
        if let Some(c) = counts.archived {
            tracing::info!("DB counts archived: {}", c);
        }
        if let Some(c) = counts.missing {
            tracing::info!("DB counts missing:  {}", c);
        }
    }

    let end_time = Utc::now().time();
    let diff = end_time - start_time;
    tracing::info!(
        "Media processing ended after {} minutes",
        diff.num_minutes()
    )
}

pub fn process_single(path: PathBuf) -> Result<(DynamicImage, DateTime<Utc>)> {
    let video_processor = VideoProcessor::new()?;
    let media_type = path.media_type().ok_or(eyre!("Invalid file type"))?;
    let res = single_media_process(&Some(video_processor), &media_type, path.as_path())?;
    res.ok_or(eyre!("No data produced"))
}
