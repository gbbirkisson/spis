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
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum MediaType {
    #[serde(rename = "img")]
    Image,
    #[serde(rename = "vid")]
    Video,
}

#[derive(Deserialize)]
pub struct MediaSearchParams {
    pub page_size: usize,
    pub taken_after: Option<DateTime<Utc>>,
}
