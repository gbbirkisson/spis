use chrono::prelude::*;
use eyre::{eyre, Result};
use rayon::prelude::*;
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::{
    fs::{self},
    path::PathBuf,
    time::SystemTime,
};
use uuid::{Builder, Uuid};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use walkdir::WalkDir;

use crate::db;

static IMAGE_FORMAT: &[&str] = &[".jpg"];
static THUMBNAIL_FORMAT: &str = "webp";
static THUMBNAIL_HEIGHT: u32 = 200;

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

trait TimeConv {
    fn conv(&self) -> DateTime<Utc>;
}

impl TimeConv for SystemTime {
    fn conv(&self) -> DateTime<Utc> {
        let (sec, nsec) = match self.duration_since(UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
            Err(e) => {
                // unlikely but should be handled
                let dur = e.duration();
                let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
                if nsec == 0 {
                    (-sec, 0)
                } else {
                    (-sec - 1, 1_000_000_000 - nsec)
                }
            }
        };
        Utc.timestamp_opt(sec, nsec).unwrap()
    }
}

pub struct ImageProcessError {
    pub msg: String,
}

pub struct ProcessedImage {
    pub uuid: Uuid,
    pub image: PathBuf,
    pub data: Option<ProcessedImageData>,
}

pub struct ProcessedImageData {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub exif: Option<exif::Exif>,
}

pub fn image_thumb(thumb_dir: &Path, uuid: &Uuid) -> PathBuf {
    let mut res = thumb_dir.join(uuid.to_string());
    res.set_extension(THUMBNAIL_FORMAT);
    res
}

pub async fn process(pool: Pool<Sqlite>, img_dir: PathBuf, thumb_dir: PathBuf) {
    tracing::info!("Starting image processing");
    let (tx, mut rx) = channel(32);
    let mut done_recv = image_processor(img_dir, thumb_dir, tx);
    let processor_pool = pool.clone();

    loop {
        tokio::select! {
            done = done_recv.recv() => {
                match done {
                    Some(count) => {
                        tracing::info!("Processed {} images", count);
                        break;
                    },
                    None => {
                        tracing::debug!("None from done channel");
                    }
                }
            }
            img = rx.recv() => {
                match img {
                    Some(img) => {
                        db::insert_image(&processor_pool, img).await;
                    }
                    None => {
                        tracing::debug!("None from channel");
                    }
                }

            }
        }
    }
}

fn image_processor(
    img_dir: PathBuf,
    thumb_dir: PathBuf,
    image_sender: Sender<ProcessedImage>,
) -> Receiver<usize> {
    let (done_send, done_recv) = channel(1);
    tokio::task::spawn_blocking(move || {
        do_walk(img_dir, thumb_dir, image_sender, done_send);
    });
    done_recv
}

fn do_walk(
    img_dir: PathBuf,
    thumb_dir: PathBuf,
    tx: Sender<ProcessedImage>,
    done_send: Sender<usize>,
) {
    let walk: Vec<_> = WalkDir::new(img_dir)
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|e| e.has_ext(IMAGE_FORMAT))
        .par_bridge()
        .map(|e| {
            if let Err(err) = do_process(thumb_dir.clone(), tx.clone(), &e) {
                let path = e.path().to_str().unwrap();
                tracing::error!("Failed processing image {path}: {err}")
            }
        })
        .collect();

    if let Err(e) = done_send
        .blocking_send(walk.len())
        .map_err(|e| eyre!("Failed sending done to channel: {:?}", e.to_string()))
    {
        tracing::error!("{e}")
    }
}

fn do_process(
    thumb_dir: PathBuf,
    tx: Sender<ProcessedImage>,
    image_entry: &walkdir::DirEntry,
) -> Result<()> {
    let image_bytes = fs::read(image_entry.path())?;
    let image_hash = md5::compute(&image_bytes);
    let image_uuid = *Builder::from_md5_bytes(image_hash.into()).as_uuid();

    let image_thumbnail_path = image_thumb(&thumb_dir, &image_uuid);

    let mut image_data = None;

    if !image_thumbnail_path.exists() {
        tracing::info!("Creating thumbnail: {:?}", image_thumbnail_path);
        let mut image = image::open(image_entry.path())?;
        image = image.thumbnail(THUMBNAIL_HEIGHT * 2, THUMBNAIL_HEIGHT);
        image.save(image_thumbnail_path)?;

        // TODO: load image exif data

        let image_meta = image_entry.metadata()?;
        let data = ProcessedImageData {
            created_at: image_meta.created()?.conv(),
            modified_at: image_meta.modified()?.conv(),
            exif: None,
        };

        image_data = Some(data);
    }

    let image = ProcessedImage {
        uuid: image_uuid,
        image: image_entry.path().to_path_buf(),
        data: image_data,
    };

    tracing::debug!("Sending image to channel {:?}", image.image);
    tx.blocking_send(image)
        .map_err(|e| eyre!("Failed sending image to channel: {:?}", e.to_string()))?;

    Ok(())
}
