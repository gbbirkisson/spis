use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Image {
    pub uuid: String,
    pub image: String,
    pub thumbnail: String,
    pub taken_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ImageSeachParams {
    pub page_size: usize,
    pub taken_after: Option<DateTime<Utc>>,
}
