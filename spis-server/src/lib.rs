use std::path::{Path, PathBuf};

use config::{Config, Environment, File};
use img::prelude::THUMBNAIL_FORMAT;
use serde::Deserialize;

use eyre::{eyre, Result};
use uuid::Uuid;

pub mod db;
pub mod img;
pub mod server;

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfg {
    image_dir: String,
    data_dir: String,
    pub processing: SpisCfgProcessing,
    pub api: SpisCfgApi,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgProcessing {
    pub run_on_start: bool,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgApi {
    pub image_path: String,
    pub thumbnail_path: String,
}

impl SpisCfg {
    pub fn new() -> Result<Self> {
        tracing::info!("Loading config");
        let b = Config::builder()
            .add_source(File::with_name("/etc/spis/config.yaml").required(false))
            .add_source(Environment::with_prefix("spis"))
            .set_default("processing.schedule", "0 0 2 * * *")?
            .set_default("processing.run_on_start", false)?
            .set_default("api.image_path", "/assets/images")?
            .set_default("api.thumbnail_path", "/assets/thumbnails")?
            .build()?;

        let c: SpisCfg = b.try_deserialize()?;

        if !Path::new(&c.image_dir).is_dir() {
            return Err(eyre!("SPIS_IMAGE_DIR {} is not a directory", c.image_dir));
        }

        if !Path::new(&c.data_dir).is_dir() {
            return Err(eyre!("SPIS_DATA_DIR {} is not a directory", c.data_dir));
        }

        tracing::debug!("Loaded config: {:?}", c);
        Ok(c)
    }

    pub fn new_testing() -> Self {
        let tmp = PathBuf::from("/tmp/").join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&tmp).expect("Failed to create tmp dir");
        let tmp = tmp.to_str().unwrap().to_string();
        Self {
            image_dir: tmp.clone(),
            data_dir: tmp,
            processing: SpisCfgProcessing {
                run_on_start: false,
                schedule: "".to_string(),
            },
            api: SpisCfgApi {
                image_path: "".to_string(),
                thumbnail_path: "".to_string(),
            },
        }
    }

    pub fn image_dir(&self) -> PathBuf {
        PathBuf::from(self.image_dir.clone())
    }

    pub fn thumbnail_dir(&self) -> PathBuf {
        PathBuf::from(self.data_dir.clone()).join("thumbnails")
    }

    pub fn db_file(&self) -> PathBuf {
        PathBuf::from(self.data_dir.clone()).join("spis.db")
    }

    pub fn api_thumbnail(&self, uuid: &Uuid) -> String {
        self.api.thumbnail_path.clone() + "/" + &uuid.to_string() + "." + THUMBNAIL_FORMAT
    }

    pub fn api_image(&self, image_path: &str) -> String {
        image_path.replace(&self.image_dir, &self.api.image_path)
    }
}
