use super::video::VideoProcessor;
use super::ProcessedMedia;
use crate::media::images::ImageProcessor;
use crate::media::util::Thumbnail;
use crate::media::{ProcessedMediaData, ProcessedMediaType};
use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use image::DynamicImage;
use md5::{Digest, Md5};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Builder;
use walkdir::WalkDir;

static EXT_IMAGE: &[&str] = &["jpg", "jpeg"];
static EXT_VIDEO: &[&str] = &["mp4"];
static THUMBNAIL_SIZE: u32 = 400;

pub(crate) trait GetMediaType {
    fn media_type(&self) -> Option<ProcessedMediaType>;
}

impl GetMediaType for Path {
    fn media_type(&self) -> Option<ProcessedMediaType> {
        if let Some(ext) = self.extension() {
            if let Some(ext) = ext.to_str() {
                for e in EXT_IMAGE {
                    if e.eq(&ext) {
                        return Some(ProcessedMediaType::Image);
                    }
                }
                for e in EXT_VIDEO {
                    if e.eq(&ext) {
                        return Some(ProcessedMediaType::Video);
                    }
                }
            }
        }
        None
    }
}

impl GetMediaType for walkdir::DirEntry {
    fn media_type(&self) -> Option<ProcessedMediaType> {
        self.path().media_type()
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub(crate) fn media_processor(
    media_dir: PathBuf,
    thumb_dir: PathBuf,
    old_entries: Option<HashMap<String, uuid::Uuid>>,
    media_sender: Sender<ProcessedMedia>,
) -> Receiver<usize> {
    let (done_send, done_recv) = channel(1);
    tokio::task::spawn_blocking(move || {
        do_walk(media_dir, thumb_dir, old_entries, media_sender, done_send);
    });
    done_recv
}

fn do_walk(
    media_dir: PathBuf,
    thumb_dir: PathBuf,
    old_entries: Option<HashMap<String, uuid::Uuid>>,
    tx: Sender<ProcessedMedia>,
    done_send: Sender<usize>,
) {
    let video_processor = match VideoProcessor::new() {
        Ok(proc) => Some(proc),
        Err(e) => {
            tracing::warn!(
                "Failed initializing video processor. No videos will be processed: {}",
                e
            );
            None
        }
    };

    let old_entries = Arc::new(old_entries.map(RwLock::new));

    let walk: Vec<_> = WalkDir::new(media_dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|r| r.ok())
        .filter_map(|e| e.media_type().map(|t| (e, t)))
        .par_bridge()
        .map(|e| {
            let (entry, media_type) = e;
            if let Err(err) = do_process(
                video_processor.clone(),
                thumb_dir.clone(),
                old_entries.clone(),
                tx.clone(),
                &entry,
                media_type,
            ) {
                let path = entry.path().to_str().unwrap();
                tracing::error!("Failed processing media {path}: {err}")
            }
        })
        .collect();

    // This sleep here is to make sure that the last media gets inserted before we kill processing
    tracing::info!("Processing done, waiting for grace period");
    std::thread::sleep(Duration::from_secs(5));

    if let Err(e) = done_send
        .blocking_send(walk.len())
        .map_err(|e| eyre!("Failed sending done to channel: {:?}", e.to_string()))
    {
        tracing::error!("{e}")
    }
}

fn get_uuid(path: &Path) -> Result<uuid::Uuid> {
    tracing::debug!("Calculating uuid for: {:?}", path);

    const BUFFER_SIZE: usize = 1024 * 1024; // 1mb

    let mut file = File::open(path)?;
    let mut buffer = [0; BUFFER_SIZE];
    let mut hasher = Md5::new();

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let result = hasher.finalize();
    Ok(*Builder::from_md5_bytes(result.into()).as_uuid())
}

fn do_process(
    video_processor: Option<VideoProcessor>,
    thumb_dir: PathBuf,
    old_entries: Arc<Option<RwLock<HashMap<String, uuid::Uuid>>>>,
    tx: Sender<ProcessedMedia>,
    media_entry: &walkdir::DirEntry,
    media_type: ProcessedMediaType,
) -> Result<()> {
    let media_path = media_entry.path().display().to_string();

    let media_uuid = match old_entries.as_ref() {
        Some(hash_map) => {
            let hash_map = hash_map
                .read()
                .map_err(|e| eyre!("RwLock poisoned: {}", e))?;
            hash_map.get(&media_path).copied()
        }
        None => None,
    };

    let media_uuid = match media_uuid {
        Some(id) => id,
        None => get_uuid(media_entry.path())?,
    };

    tracing::debug!("Processing {}: {}", media_uuid, media_path);

    let media_thumbnail_path = thumb_dir.get_thumbnail(&media_uuid);

    let processed = if !media_thumbnail_path.exists() {
        single_media_process(&video_processor, &media_type, media_entry.path())?
    } else {
        None
    };

    let processed_media = match processed {
        Some((thumb, taken_at)) => {
            thumb.save(media_thumbnail_path)?;
            ProcessedMedia {
                uuid: media_uuid,
                path: media_path,
                data: Some(ProcessedMediaData { taken_at }),
                media_type,
            }
        }
        None => ProcessedMedia {
            uuid: media_uuid,
            path: media_path,
            data: None,
            media_type,
        },
    };

    tx.blocking_send(processed_media)
        .map_err(|e| eyre!("Failed sending media to channel: {:?}", e.to_string()))?;

    Ok(())
}

pub(crate) fn single_media_process(
    video_processor: &Option<VideoProcessor>,
    media_type: &ProcessedMediaType,
    media_path: &Path,
) -> Result<Option<(DynamicImage, DateTime<Utc>)>> {
    let media_path_str = media_path.display().to_string();

    let res = match (video_processor, &media_type) {
        (Some(video_processor), ProcessedMediaType::Video) => {
            tracing::debug!("Processing video: {}", media_path_str);
            let thumb = video_processor.get_thumbnail(&media_path_str, THUMBNAIL_SIZE)?;
            let taken_at = video_processor.get_timestamp(&media_path_str)?;
            Some((thumb, taken_at))
        }
        (_, ProcessedMediaType::Image) => {
            tracing::debug!("Processing image: {}", media_path_str);
            let image_processor = ImageProcessor::new(media_path)?;
            let thumb = image_processor.get_thumbnail(THUMBNAIL_SIZE)?;
            let taken_at = image_processor.get_timestamp()?;
            Some((thumb, taken_at))
        }
        (_, _) => None,
    };

    Ok(res)
}
