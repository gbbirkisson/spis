pub mod db;
pub mod media;
pub mod pipeline;
pub mod prelude;
pub mod server;

#[derive(Clone)]
pub struct CustomCommand {
    pub name: String,
    pub cmd: Vec<String>,
    pub fa_icon: String,
    pub hotkey: Option<char>,
}

#[derive(Clone, Debug)]
pub enum MediaEvent {
    Added(uuid::Uuid),
    Changed(uuid::Uuid),
    Archived(uuid::Uuid),
}

pub struct PathFinder {
    media_dir: String,
    media_path: String,
    thumbnail_path: String,
    thumbnail_ext: String,
}

impl PathFinder {
    #[must_use]
    pub fn new(
        media_dir: &str,
        media_path: &str,
        thumbnail_path: &str,
        thumbnail_ext: &str,
    ) -> Self {
        Self {
            media_dir: media_dir.to_string(),
            media_path: media_path.to_string(),
            thumbnail_path: thumbnail_path.to_string(),
            thumbnail_ext: thumbnail_ext.to_string(),
        }
    }

    #[must_use]
    pub fn thumbnail(&self, id: &uuid::Uuid) -> String {
        format!("{}/{}.{}", self.thumbnail_path, id, self.thumbnail_ext)
    }

    #[must_use]
    pub fn media(&self, path: &str) -> String {
        path.replace(&self.media_dir, &self.media_path)
    }
}
