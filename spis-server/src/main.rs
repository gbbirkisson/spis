use async_cron_scheduler::{Job, Scheduler};
use chrono::Local;
use eyre::Result;
use spis_server::{db, med, server, SpisCfg};
use sqlx::{Pool, Sqlite};
use std::net::TcpListener;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
    let server = server::run(listener, pool, config).expect("Failed to create server");
    server.await?;

    Ok(())
}

async fn setup_processing(pool: Pool<Sqlite>, config: SpisCfg) -> Result<()> {
    tracing::info!("Setup processing");

    let media_dir = config.media_dir();
    let thumb_dir = config.thumbnail_dir();
    let schedule = config.processing.schedule;
    std::fs::create_dir_all(&thumb_dir)?;

    tokio::spawn(async move {
        if config.processing.run_on_start {
            tracing::info!("Running on-start processing");
            med::process(pool.clone(), media_dir.clone(), thumb_dir.clone()).await;
            tracing::info!("Done with on-start processing");
        }

        tracing::info!("Added processing schedule: {}", schedule);
        let (mut scheduler, sched_service) = Scheduler::<Local>::launch(tokio::time::sleep);
        let job = Job::cron(&schedule).unwrap();
        scheduler.insert(job, move |_| {
            let pool = pool.clone();
            let media_dir = media_dir.clone();
            let thumb_dir = thumb_dir.clone();
            let schedule = schedule.clone();

            tokio::spawn(async move {
                tracing::info!("Processing schedule triggered: {}", schedule);
                med::process(pool, media_dir, thumb_dir).await;
                tracing::info!("Processing schedule finished: {}", schedule);
            });
        });
        sched_service.await;
    });
    tracing::debug!("Setup processing done");
    Ok(())
}
