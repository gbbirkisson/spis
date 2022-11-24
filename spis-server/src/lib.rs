use config::{Config, Environment};
use eyre::{eyre, Result};
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
    processing: SpisCfgProcessing,
    api: SpisCfgApi,
    server: SpisCfgServer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgProcessing {
    run_on_start: bool,
    schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SpisCfgApi {
    media_path: String,
    thumbnail_path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SpisCfgServer {
    address: Option<String>,
    socket: Option<String>,
}

impl SpisCfg {
    pub fn new() -> Result<Self> {
        tracing::info!("Loading config");
        let b = Config::builder()
            .add_source(Environment::with_prefix("spis"))
            .set_default("processing.schedule", "0 0 2 * * *")?
            .set_default("processing.run_on_start", false)?
            .set_default("api.media_path", "/assets/media")?
            .set_default("api.thumbnail_path", "/assets/thumbnails")?
            .set_default("server.socket", "/var/run/spis.sock")?
            .build()?;

        let c: SpisCfg = b.try_deserialize()?;

        if !Path::new(&c.media_dir).is_dir() {
            return Err(eyre!("SPIS_MEDIA.DIR {} is not a directory", c.media_dir));
        }

        if !Path::new(&c.data_dir).is_dir() {
            return Err(eyre!("SPIS_DATA.DIR {} is not a directory", c.data_dir));
        }

        tracing::debug!("Loaded config: {:?}", c);
        Ok(c)
    }

    pub fn server_listener(&self) -> SpisServerListener {
        match (&self.server.address, &self.server.socket) {
            (Some(address), _) => SpisServerListener::Address(address.clone()),
            (None, Some(socket)) => SpisServerListener::Socket(socket.clone()),
            _ => unreachable!("This should never happen"),
        }
    }

    pub fn media_converter(&self) -> MediaConverter {
        MediaConverter::new(
            &self.media_dir,
            &self.api.media_path,
            &self.api.thumbnail_path,
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
        self.processing.schedule.clone()
    }

    pub fn processing_run_on_start(&self) -> bool {
        self.processing.run_on_start
    }
}
