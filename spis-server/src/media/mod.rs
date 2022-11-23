use chrono::prelude::*;
use eyre::{eyre, Result};
use rayon::prelude::*;
use sqlx::{Pool, Sqlite};
use std::time::Duration;
use std::{
    fs::{self},
    path::PathBuf,
};
use uuid::{Builder, Uuid};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use walkdir::WalkDir;

use crate::db;
use crate::media::prelude::Thumbnail;

mod meta;
pub mod prelude;

static MEDIA_FORMAT: &[&str] = &[".jpg", ".jpeg"];
static THUMBNAIL_SIZE: u32 = 400;

trait HasExt {
    fn has_ext(&self, ext: &[&str]) -> bool;
}

impl HasExt for walkdir::DirEntry {
    fn has_ext(&self, ext: &[&str]) -> bool {
        match self.file_name().to_str() {
            None => (),
            Some(name) => {
                for e in ext {
                    if name.ends_with(e) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub struct ProcessedMedia {
    pub uuid: Uuid,
    pub media: PathBuf,
    pub data: Option<ProcessedMediaData>,
}

pub struct ProcessedMediaData {
    pub taken_at: DateTime<Utc>,
}

pub async fn process(pool: Pool<Sqlite>, media_dir: PathBuf, thumb_dir: PathBuf) {
    let start_time = Utc::now().time();
    tracing::info!("Media processing started");

    tracing::debug!("Mark entire database as unwalked");
    let mark = db::media_mark_unwalked(&pool).await;
    if mark.is_err() {
        tracing::error!("Failed marking media as unwalked: {:?}", &mark);
    }

    tracing::debug!("Setup processing channels and pool");
    let (tx, mut rx) = channel(1);
    let mut done_recv = media_processor(media_dir, thumb_dir, tx);
    let processor_pool = pool.clone();

    loop {
        tokio::select! {
            done = done_recv.recv() => {
                match done {
                    Some(count) => {
                        tracing::info!("Processed {} files", count);
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
                        }
                    }
                    None => {
                        tracing::debug!("None from channel");
                    }
                }

            }
        }
    }

    tracing::debug!("Delete all media that was not walked");
    if mark.is_ok() {
        match db::media_delete_unwalked(&pool).await {
            Ok(count) => {
                tracing::info!("Cleaned up {count} media entries");
            }
            Err(e) => {
                tracing::error!("Failed deleting unwalked media: {:?}", e);
            }
        }
    }

    // TODO: Cleanup thumbnails?

    if let Ok(count) = db::media_count(&pool).await {
        tracing::info!("DB now has {count} media entries");
    }

    let end_time = Utc::now().time();
    let diff = end_time - start_time;
    tracing::info!(
        "Media processing ended after {} minutes",
        diff.num_minutes()
    )
}

fn media_processor(
    media_dir: PathBuf,
    thumb_dir: PathBuf,
    media_sender: Sender<ProcessedMedia>,
) -> Receiver<usize> {
    let (done_send, done_recv) = channel(1);
    tokio::task::spawn_blocking(move || {
        do_walk(media_dir, thumb_dir, media_sender, done_send);
    });
    done_recv
}

fn do_walk(
    media_dir: PathBuf,
    thumb_dir: PathBuf,
    tx: Sender<ProcessedMedia>,
    done_send: Sender<usize>,
) {
    let walk: Vec<_> = WalkDir::new(media_dir)
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|e| e.has_ext(MEDIA_FORMAT))
        .par_bridge()
        .map(|e| {
            if let Err(err) = do_process(thumb_dir.clone(), tx.clone(), &e) {
                let path = e.path().to_str().unwrap();
                tracing::error!("Failed processing media {path}: {err}")
            }
        })
        .collect();

    // This sleep here is to make sure that the last media gets inserted before we kill processing
    std::thread::sleep(Duration::from_secs(5));

    if let Err(e) = done_send
        .blocking_send(walk.len())
        .map_err(|e| eyre!("Failed sending done to channel: {:?}", e.to_string()))
    {
        tracing::error!("{e}")
    }
}

fn do_process(
    thumb_dir: PathBuf,
    tx: Sender<ProcessedMedia>,
    media_entry: &walkdir::DirEntry,
) -> Result<()> {
    let media_bytes = fs::read(media_entry.path())?;
    let media_hash = md5::compute(&media_bytes);
    let media_uuid = *Builder::from_md5_bytes(media_hash.into()).as_uuid();

    let media_thumbnail_path = thumb_dir.get_thumbnail(&media_uuid);

    let mut media_data = None;

    if !media_thumbnail_path.exists() {
        tracing::debug!("Reading EXIF data for {:?}", media_entry.path());
        let exif = match meta::exif_read(&media_bytes) {
            Ok(e) => Some(e),
            Err(_) => {
                tracing::debug!("Failed to read EXIF data for {:?}", media_entry.path());
                None
            }
        };

        tracing::debug!("Creating thumbnail: {:?}", media_thumbnail_path);
        let mut image = image::open(media_entry.path())?;
        image = image.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
        if let Some(exif) = &exif {
            image = match exif.orientation.0 {
                90 => image.rotate90(),

                180 => image.rotate180(),

                270 => image.rotate270(),

                _ => image,
            };
            if exif.orientation.1 {
                image = image.flipv();
            }
        }

        let image_height = image.height();
        let image_width = image.width();
        image = match image_height > image_width {
            true => image.crop(
                0,
                (image_height - image_width) / 2,
                image_width,
                image_width,
            ),
            false => image.crop(
                (image_width - image_height) / 2,
                0,
                image_height,
                image_height,
            ),
        };
        image.save(media_thumbnail_path)?;

        let taken = match exif.map(|e| e.taken) {
            Some(taken) => taken,
            None => match media_entry.metadata() {
                Ok(meta) => match meta.modified() {
                    Ok(time) => Some(time.into()),
                    Err(_) => {
                        tracing::warn!("Could not determin timestamp for {:?}", media_entry.path());
                        None
                    }
                },
                Err(_) => None,
            },
        };

        let data = ProcessedMediaData {
            taken_at: taken.unwrap_or_else(Utc::now),
        };

        media_data = Some(data);
    }

    let media = ProcessedMedia {
        uuid: media_uuid,
        media: media_entry.path().to_path_buf(),
        data: media_data,
    };

    tracing::debug!("Sending media to channel {:?}", media.uuid);
    tx.blocking_send(media)
        .map_err(|e| eyre!("Failed sending media to channel: {:?}", e.to_string()))?;

    Ok(())
}
