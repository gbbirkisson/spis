use std::path::PathBuf;
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
