use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub path: String,
}
