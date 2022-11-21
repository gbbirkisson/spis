use std::path::{Path, PathBuf};

use config::{Config, Environment, File};
use med::prelude::THUMBNAIL_FORMAT;
use serde::Deserialize;

use eyre::{eyre, Result};
use uuid::Uuid;

pub mod db;
pub mod med;
pub mod server;

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfg {
    media_dir: String,
    data_dir: String,
    pub processing: SpisCfgProcessing,
    pub api: SpisCfgApi,
    pub server: SpisCfgServer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgProcessing {
    pub run_on_start: bool,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgApi {
    pub media_path: String,
    pub thumbnail_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgServer {
    pub address: Option<String>,
    pub socket: Option<String>,
}

impl SpisCfg {
    pub fn new() -> Result<Self> {
        tracing::info!("Loading config");
        let b = Config::builder()
            .add_source(File::with_name("/etc/spis/config.yaml").required(false))
            .add_source(Environment::with_prefix("spis"))
            .set_default("processing.schedule", "0 0 2 * * *")?
            .set_default("processing.run_on_start", false)?
            .set_default("api.media_path", "/assets/media")?
            .set_default("api.thumbnail_path", "/assets/thumbnails")?
            .build()?;

        let c: SpisCfg = b.try_deserialize()?;

        if !Path::new(&c.media_dir).is_dir() {
            return Err(eyre!("SPIS_MEDIA.DIR {} is not a directory", c.media_dir));
        }

        if !Path::new(&c.data_dir).is_dir() {
            return Err(eyre!("SPIS_DATA.DIR {} is not a directory", c.data_dir));
        }

        if c.server.address.is_none() && c.server.socket.is_none() {
            return Err(eyre!(
                "You have to specify SPIS_SERVER.ADDRESS or SPIS_SERVER.SOCKET"
            ));
        }

        if c.server.address.is_some() && c.server.socket.is_some() {
            return Err(eyre!(
                "You cannot specify both SPIS_SERVER.ADDRESS and SPIS_SERVER.SOCKET"
            ));
        }

        tracing::debug!("Loaded config: {:?}", c);
        Ok(c)
    }

    pub fn new_testing() -> Self {
        let tmp = PathBuf::from("/tmp/").join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&tmp).expect("Failed to create tmp dir");
        let tmp = tmp.to_str().unwrap().to_string();
        Self {
            media_dir: tmp.clone(),
            data_dir: tmp,
            processing: SpisCfgProcessing {
                run_on_start: false,
                schedule: "".to_string(),
            },
            api: SpisCfgApi {
                media_path: "".to_string(),
                thumbnail_path: "".to_string(),
            },
            server: SpisCfgServer {
                address: Some("127.0.0.1:0".to_string()),
                socket: None,
            },
        }
    }

    pub fn media_dir(&self) -> PathBuf {
        PathBuf::from(self.media_dir.clone())
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

    pub fn api_media_location(&self, media_path: &str) -> String {
        media_path.replace(&self.media_dir, &self.api.media_path)
    }
}
