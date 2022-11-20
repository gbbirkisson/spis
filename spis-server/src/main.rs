use async_cron_scheduler::{Job, Scheduler};
use chrono::Local;
use config::{Config, Environment, File};
use eyre::{eyre, Result};
use serde::Deserialize;
use spis_server::{db, img, server};
use sqlx::{Pool, Sqlite};
use std::{
    net::TcpListener,
    path::{Path, PathBuf},
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfg {
    image_dir: String,
    data_dir: String,
    pub processing: SpisCfgProcessing,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpisCfgProcessing {
    pub run_on_start: bool,
    pub schedule: String,
}

impl SpisCfg {
    pub fn new() -> Result<Self> {
        tracing::info!("Loading config");
        let b = Config::builder()
            .add_source(File::with_name("/etc/spis/config.yaml").required(false))
            .add_source(Environment::with_prefix("spis"))
            .set_default("processing.schedule", "0 0 2 * * *")?
            .set_default("processing.run_on_start", false)?
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

    pub fn image_dir(&self) -> PathBuf {
        PathBuf::from(self.image_dir.clone())
    }

    pub fn thumbnail_dir(&self) -> PathBuf {
        PathBuf::from(self.data_dir.clone()).join("thumbnails")
    }

    pub fn db_file(&self) -> PathBuf {
        PathBuf::from(self.data_dir.clone()).join("spis.db")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    tracing::info!("Starting spis");

    let config = SpisCfg::new()?;
    let pool = db::setup_db(config.db_file()).await.unwrap();

    setup_processing(pool.clone(), config.clone()).await?;

    tracing::info!("Start server on http://0.0.0.0:8000");
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    let server =
        server::run(listener, pool, config.thumbnail_dir()).expect("Failed to create server");
    server.await?;

    Ok(())
}

async fn setup_processing(pool: Pool<Sqlite>, config: SpisCfg) -> Result<()> {
    tracing::info!("Setup processing");

    let img_dir = config.image_dir();
    let thumb_dir = config.thumbnail_dir();
    let schedule = config.processing.schedule;
    std::fs::create_dir_all(&thumb_dir)?;

    tokio::spawn(async move {
        if config.processing.run_on_start {
            tracing::info!("Running on-start processing");
            img::process(pool.clone(), img_dir.clone(), thumb_dir.clone()).await;
            tracing::info!("Done with on-start processing");
        }

        tracing::info!("Added processing schedule: {}", schedule);
        let (mut scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        let job = Job::cron(&schedule).unwrap();
        scheduler.insert(job, move |_| {
            let pool = pool.clone();
            let img_dir = img_dir.clone();
            let thumb_dir = thumb_dir.clone();
            let schedule = schedule.clone();

            tokio::spawn(async move {
                tracing::info!("Processing schedule triggered: {}", schedule);
                img::process(pool, img_dir, thumb_dir).await;
                tracing::info!("Processing schedule finished: {}", schedule);
            });
        });
        sched_service.await;
    });
    tracing::debug!("Setup processing done");
    Ok(())
}
