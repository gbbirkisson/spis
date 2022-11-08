use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Image {
    pub path: String,
    pub thumbnail: String,
    pub hash: String,
}
