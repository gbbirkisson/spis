use color_eyre::Result;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Builder;
use uuid::Uuid;

pub static THUMBNAIL_FORMAT: &str = "webp";

pub(crate) trait Thumbnail {
    fn get_thumbnail(&self, uuid: &Uuid) -> PathBuf;
}

impl Thumbnail for PathBuf {
    fn get_thumbnail(&self, uuid: &Uuid) -> PathBuf {
        let mut res = self.join(uuid.to_string());
        res.set_extension(THUMBNAIL_FORMAT);
        res
    }
}

pub(crate) fn get_uuid(path: &Path) -> Result<uuid::Uuid> {
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
