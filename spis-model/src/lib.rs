use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Media {
    pub uuid: String,
    pub location: String,
    pub thumbnail: String,
    pub taken_at: DateTime<Utc>,
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub archived: bool,
    pub favorite: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum MediaType {
    #[serde(rename = "img")]
    Image,
    #[serde(rename = "vid")]
    Video,
}

#[derive(Serialize, Deserialize)]
pub struct MediaListParams {
    pub page_size: usize,
    pub archived: Option<bool>,
    pub taken_after: Option<DateTime<Utc>>,
    pub taken_before: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct MediaEditParams {
    pub archive: Option<bool>,
    pub favorite: Option<bool>,
}
