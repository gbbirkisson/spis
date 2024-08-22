use self::util::get_uuid;
use self::video::VideoProcessor;
use crate::media::images::ImageProcessor;
use crate::media::util::Thumbnail;
use chrono::{DateTime, Utc};
use color_eyre::eyre::Context;
use color_eyre::Result;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub mod images;
pub mod util;
pub mod video;

static THUMBNAIL_SIZE: u32 = 400;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ProcessedMedia {
    pub uuid: Uuid,
    pub path: String,
    pub media_type: ProcessedMediaType,
    pub data: Option<ProcessedMediaData>,
}

#[derive(Debug)]
pub enum ProcessedMediaType {
    Image,
    Video,
}

#[derive(Debug)]
pub struct ProcessedMediaData {
    pub taken_at: DateTime<Utc>,
}

pub(crate) struct MediaProcessor {
    video_processor: Option<VideoProcessor>,
    thumbnail_path: PathBuf,
    force_processing: bool,
}

impl MediaProcessor {
    pub fn new(thumbnail_path: PathBuf, force_processing: bool) -> Self {
        let video_processor = match VideoProcessor::new() {
            Ok(proc) => Some(proc),
            Err(error) => {
                tracing::warn!(
                    "Failed initializing video processor. No videos will be processed: {:?}",
                    error
                );
                None
            }
        };

        Self {
            video_processor,
            thumbnail_path,
            force_processing,
        }
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn process(
        &self,
        media_uuid: Option<uuid::Uuid>,
        media_path: &Path,
        media_type: ProcessedMediaType,
        allow_no_exif: bool,
    ) -> Result<ProcessedMedia> {
        let media_uuid = match media_uuid {
            Some(media_uuid) => media_uuid,
            None => get_uuid(media_path)?,
        };

        let media_path_str = media_path.display().to_string();
        let media_thumbnail_path = self.thumbnail_path.get_thumbnail(&media_uuid);

        let processed = if media_thumbnail_path.exists() && !self.force_processing {
            tracing::trace!("Thumbnail already exists: {:?}", media_thumbnail_path);
            None
        } else {
            match (&self.video_processor, &media_type) {
                (Some(video_processor), ProcessedMediaType::Video) => {
                    tracing::debug!("Processing video: {}", media_path_str);
                    let thumb = video_processor
                        .get_thumbnail(&media_path_str, THUMBNAIL_SIZE)
                        .wrap_err("Thumb creation failed")?;
                    let mut taken_at = video_processor
                        .get_timestamp(&media_path_str)
                        .wrap_err("Timestamp parsing failed");
                    if taken_at.is_err() && allow_no_exif {
                        tracing::trace!("No exif, using creation date: {}", media_path_str);
                        taken_at = get_creation_date(media_path);
                    }
                    Some((thumb, taken_at?))
                }
                (_, ProcessedMediaType::Image) => {
                    tracing::debug!("Processing image: {}", media_path_str);
                    let image_processor = ImageProcessor::new(media_path)?;
                    let thumb = image_processor
                        .get_thumbnail(THUMBNAIL_SIZE)
                        .wrap_err("Thumb creation failed")?;
                    let mut taken_at = image_processor
                        .get_timestamp()
                        .wrap_err("Timestamp parsing failed");
                    if taken_at.is_err() && allow_no_exif {
                        tracing::trace!("No exif, using creation date: {}", media_path_str);
                        taken_at = get_creation_date(media_path);
                    }
                    Some((thumb, taken_at?))
                }
                (_, _) => {
                    tracing::trace!("Skipping media: {}", media_path_str);
                    None
                }
            }
        };

        let media = match processed {
            Some((thumb, taken_at)) => {
                thumb.save(media_thumbnail_path)?;
                ProcessedMedia {
                    uuid: media_uuid,
                    path: media_path_str,
                    data: Some(ProcessedMediaData { taken_at }),
                    media_type,
                }
            }
            None => ProcessedMedia {
                uuid: media_uuid,
                path: media_path_str,
                data: None,
                media_type,
            },
        };

        Ok(media)
    }
}

fn get_creation_date(file: &Path) -> Result<DateTime<Utc>> {
    let metadata = std::fs::metadata(file)?;
    let created = metadata.created()?;
    Ok(DateTime::from(created))
}
