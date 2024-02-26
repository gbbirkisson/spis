use color_eyre::eyre::Context;
use color_eyre::Result;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Builder;
use uuid::Uuid;

pub const THUMBNAIL_FORMAT: &str = "webp";
const BUFFER_SIZE: usize = 512_000;

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

    let mut file = File::open(path).wrap_err("Failed to open file")?;
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    let mut hasher = Md5::new();

    loop {
        let read = file.read(&mut buffer).wrap_err("Failed to read file")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let result = hasher.finalize();
    Ok(*Builder::from_md5_bytes(result.into()).as_uuid())
}
