use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Media {
    pub uuid: String,
    pub location: String,
    pub thumbnail: String,
    pub taken_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct MediaSearchParams {
    pub page_size: usize,
    pub taken_after: Option<DateTime<Utc>>,
}
