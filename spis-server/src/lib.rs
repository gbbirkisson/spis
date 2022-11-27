use color_eyre::{eyre::eyre, Result};
use config::{Config, Environment};
use media::util::THUMBNAIL_FORMAT;
use serde::Deserialize;
use server::convert::MediaConverter;
use std::path::{Path, PathBuf};

pub mod db;
pub mod media;
pub mod server;

pub enum SpisServerListener {
    Address(String),
    Socket(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfg {
    media_dir: String,
    data_dir: String,
    processing_schedule: String,
    processing_run_on_start: bool,
    api_media_path: String,
    api_thumbnail_path: String,
    server_address: Option<String>,
    server_socket: Option<String>,
}

impl SpisCfg {
    pub fn new() -> Result<Self> {
        tracing::info!("Loading config");
        let b = Config::builder()
            .add_source(Environment::with_prefix("spis"))
            .set_default("processing_schedule", "0 0 2 * * *")?
            .set_default("processing_run_on_start", false)?
            .set_default("api_media_path", "/assets/media")?
            .set_default("api_thumbnail_path", "/assets/thumbnails")?
            .set_default("server_socket", "/var/run/spis.sock")?
            .build()?;

        let c: SpisCfg = b.try_deserialize()?;

        if !Path::new(&c.media_dir).is_dir() {
            return Err(eyre!("SPIS_MEDIA_DIR {} is not a directory", c.media_dir));
        }

        if !Path::new(&c.data_dir).is_dir() {
            return Err(eyre!("SPIS_DATA_DIR {} is not a directory", c.data_dir));
        }

        tracing::debug!("Loaded config: {:?}", c);
        Ok(c)
    }

    pub fn server_listener(&self) -> SpisServerListener {
        match (&self.server_address, &self.server_socket) {
            (Some(address), _) => SpisServerListener::Address(address.clone()),
            (None, Some(socket)) => SpisServerListener::Socket(socket.clone()),
            _ => unreachable!("This should never happen"),
        }
    }

    pub fn media_converter(&self) -> MediaConverter {
        MediaConverter::new(
            &self.media_dir,
            &self.api_media_path,
            &self.api_thumbnail_path,
            THUMBNAIL_FORMAT,
        )
    }

    pub fn media_dir(&self) -> PathBuf {
        PathBuf::from(self.media_dir.clone())
    }

    pub fn thumbnail_dir(&self) -> PathBuf {
        PathBuf::from(self.data_dir.clone()).join("thumbnails")
    }

    pub fn db_file(&self) -> String {
        self.data_dir.clone() + "/spis.db"
    }

    pub fn processing_schedule(&self) -> String {
        self.processing_schedule.clone()
    }

    pub fn processing_run_on_start(&self) -> bool {
        self.processing_run_on_start
    }
}
