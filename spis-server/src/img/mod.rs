use chksum::{prelude::HashAlgorithm, Chksum};
use rayon::prelude::*;
use std::{fmt::Error, fs::File, path::PathBuf, time::SystemTime};

use tokio::sync::mpsc::Sender;
use walkdir::WalkDir;

static IMAGE_FORMAT: &[&str] = &[".jpg"];
static THUMBNAIL_FORMAT: &str = "webp";
static THUMBNAIL_HEIGHT: u32 = 200;
static HASH_ALGO: HashAlgorithm = HashAlgorithm::MD5;

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

pub struct ProcessedImage {
    pub image: PathBuf,
    pub hash: String,
    pub thumbnail: PathBuf,
    pub created_at: Option<SystemTime>,
    pub modified_at: Option<SystemTime>,
    pub exif: Option<exif::Exif>,
}

pub fn image_processor(img_dir: PathBuf, thumb_dir: PathBuf, tx: Sender<ProcessedImage>) {
    tracing::info!("Starting image processing");
    tokio::task::spawn_blocking(move || {
        do_walk(img_dir, thumb_dir, tx);
    });
}

fn do_walk(img_dir: PathBuf, thumb_dir: PathBuf, tx: Sender<ProcessedImage>) {
    let walk: Vec<_> = WalkDir::new(img_dir)
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|e| e.has_ext(IMAGE_FORMAT))
        .par_bridge()
        .map(|e| do_process(thumb_dir.clone(), tx.clone(), e).ok())
        .collect();
    tracing::info!("Successfully processed {} images", walk.len());
}

fn do_process(
    thumb_dir: PathBuf,
    tx: Sender<ProcessedImage>,
    image_entry: walkdir::DirEntry,
) -> Result<(), Error> {
    let mut image_file = File::open(image_entry.path()).expect("File should exist"); // TODO: Handle
    let image_hash_digest = image_file.chksum(HASH_ALGO).expect("Hashing should work"); // TODO: Handle
    let image_hash = format!("{:x}", image_hash_digest);

    let mut image_thumbnail_path = thumb_dir.join(image_hash.clone());
    image_thumbnail_path.set_extension(THUMBNAIL_FORMAT);

    let image_exif = None;
    let mut image_created_at = None;
    let mut image_modified_at = None;

    if !image_thumbnail_path.exists() {
        tracing::info!("Creating thumbnail: {:?}", image_thumbnail_path);
        let mut image = image::open(image_entry.path()).expect("should work");
        image = image.thumbnail(THUMBNAIL_HEIGHT * 2, THUMBNAIL_HEIGHT);

        image
            .save(image_thumbnail_path.clone())
            .expect("Saving image should work"); // TODO

        // TODO: load image exif data

        let image_meta = image_entry.metadata().expect("Able to get metadata"); // TODO
        image_created_at = Some(image_meta.created().expect("To work")); // TODO
        image_modified_at = Some(image_meta.modified().expect("To work"));
        // TODO
    }

    let image = ProcessedImage {
        image: image_entry.path().to_path_buf(),
        hash: image_hash,
        thumbnail: image_thumbnail_path,
        created_at: image_created_at,
        modified_at: image_modified_at,
        exif: image_exif,
    };
    tracing::debug!("Sending image to channel {:?}", image.image);
    if tx.blocking_send(image).is_err() {
        tracing::error!("Failed sending image to channel");
    }

    Ok(())
}
