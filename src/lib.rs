use crate::prelude::*;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use config::{Config, Environment};
use media::util::THUMBNAIL_FORMAT;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub mod db;
pub mod media;
pub mod pipeline;
pub mod prelude;
pub mod server;

pub enum SpisServerListener {
    Address(String),
    Socket(String),
}

pub struct PathFinder {
    media_dir: String,
    media_path: String,
    thumbnail_path: String,
    thumbnail_ext: String,
}

impl PathFinder {
    #[must_use]
    pub fn new(
        media_dir: &str,
        media_path: &str,
        thumbnail_path: &str,
        thumbnail_ext: &str,
    ) -> Self {
        Self {
            media_dir: media_dir.to_string(),
            media_path: media_path.to_string(),
            thumbnail_path: thumbnail_path.to_string(),
            thumbnail_ext: thumbnail_ext.to_string(),
        }
    }

    #[must_use]
    pub fn thumbnail(&self, id: &uuid::Uuid) -> String {
        format!("{}/{}.{}", self.thumbnail_path, id, self.thumbnail_ext)
    }

    #[must_use]
    pub fn media(&self, path: &str) -> String {
        path.replace(&self.media_dir, &self.media_path)
    }
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
    feature_archive: bool,
    feature_favorite: bool,
}

impl SpisCfg {
    pub fn new() -> Result<Self> {
        tracing::debug!("Loading config");
        let b = Config::builder()
            .add_source(Environment::with_prefix("spis"))
            .set_default("processing_schedule", "0 0 2 * * *")?
            .set_default("processing_run_on_start", false)?
            .set_default("api_media_path", "/assets/media")?
            .set_default("api_thumbnail_path", "/assets/thumbnails")?
            .set_default("server_socket", "/var/run/spis.sock")?
            .set_default("feature_archive", true)?
            .set_default("feature_favorite", true)?
            .build()
            .wrap_err("Failed to build config")?;

        let mut c: Self = b
            .try_deserialize()
            .wrap_err("Failed to deserialize config")?;

        if !Path::new(&c.media_dir).is_dir() {
            return Err(eyre!("SPIS_MEDIA_DIR {} is not a directory", c.media_dir));
        }

        if !Path::new(&c.data_dir).is_dir() {
            return Err(eyre!("SPIS_DATA_DIR {} is not a directory", c.data_dir));
        }

        let media_dir = std::fs::canonicalize(&c.media_dir)
            .wrap_err("Failed to get media_dir absolute path")?;
        c.media_dir = W(media_dir).into();

        let data_dir =
            std::fs::canonicalize(&c.data_dir).wrap_err("Failed to get data_dir absolute path")?;
        c.data_dir = W(data_dir).into();

        tracing::debug!("Loaded config: {:?}", c);
        Ok(c)
    }

    #[must_use]
    pub fn server_listener(&self) -> SpisServerListener {
        match (&self.server_address, &self.server_socket) {
            (Some(address), _) => SpisServerListener::Address(address.clone()),
            (None, Some(socket)) => SpisServerListener::Socket(socket.clone()),
            _ => unreachable!("This should never happen"),
        }
    }

    #[must_use]
    pub fn pathfinder(&self) -> PathFinder {
        PathFinder::new(
            &self.media_dir,
            &self.api_media_path,
            &self.api_thumbnail_path,
            THUMBNAIL_FORMAT,
        )
    }

    #[must_use]
    pub fn media_dir(&self) -> PathBuf {
        PathBuf::from(self.media_dir.clone())
    }

    #[must_use]
    pub fn thumbnail_dir(&self) -> PathBuf {
        PathBuf::from(self.data_dir.clone()).join("thumbnails")
    }

    #[must_use]
    pub fn db_file(&self) -> String {
        self.data_dir.clone() + "/spis.db"
    }

    #[must_use]
    pub fn processing_schedule(&self) -> String {
        self.processing_schedule.clone()
    }

    #[must_use]
    pub const fn processing_run_on_start(&self) -> bool {
        self.processing_run_on_start
    }

    #[must_use]
    pub const fn feature_archive(&self) -> bool {
        self.feature_archive
    }

    #[must_use]
    pub const fn feature_favorite(&self) -> bool {
        self.feature_favorite
    }
}
