use spis_model::Media;

pub type IconColor = String;

#[derive(Clone, Eq, PartialEq)]
pub struct MediaDataEntry {
    pub index: usize,
    pub total: usize,
    pub media: Media,
}

pub type MediaData = Vec<MediaDataEntry>;

pub trait ToMediaData {
    fn to_media_data(self) -> MediaData;
}

impl ToMediaData for Vec<Media> {
    fn to_media_data(self) -> MediaData {
        let total = self.len();
        self.into_iter()
            .enumerate()
            .map(|(index, media)| MediaDataEntry {
                index,
                media,
                total,
            })
            .collect()
    }
}

pub trait SafeRemove {
    fn safe_remove(self, index: usize) -> MediaData;
}

impl SafeRemove for MediaData {
    fn safe_remove(mut self, index: usize) -> MediaData {
        self.remove(index);
        let res: Vec<Media> = self.into_iter().map(|e| e.media).collect();
        res.to_media_data()
    }
}
