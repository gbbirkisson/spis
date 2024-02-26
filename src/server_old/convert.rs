use crate::db::{MediaRow, MediaTypeConverter};
use spis_model::Media;

pub struct MediaConverter {
    media_dir: String,
    media_path: String,
    thumbnail_path: String,
    thumbnail_ext: String,
}

impl MediaConverter {
    #[allow(clippy::must_use_candidate)]
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

    pub(crate) fn convert(&self, media: &MediaRow) -> Media {
        Media {
            uuid: media.id.to_string(),
            taken_at: media.taken_at,
            location: media.path.replace(&self.media_dir, &self.media_path),
            thumbnail: format!(
                "{}/{}.{}",
                self.thumbnail_path, media.id, self.thumbnail_ext
            ),
            media_type: media.media_type.convert(),
            archived: media.archived,
            favorite: media.favorite,
        }
    }
}
