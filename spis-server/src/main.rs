use async_cron_scheduler::{Job, Scheduler};
use chrono::Local;
use clap::Parser;
use color_eyre::Result;
use spis_server::{
    db, media,
    server::{self, Listener},
    SpisCfg, SpisServerListener,
};
use sqlx::{Pool, Sqlite};
use std::{net::TcpListener, path::PathBuf};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// The SPIS server application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to test process, and then exit
    #[arg(short, long)]
    test_media: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    // Parse args
    let args = Args::parse();
    if args.test_media.len() > 0 {
        for file in args.test_media {
            let path = file.display().to_string();
            let data = media::process_single(file)?;
            println!("{:?} {:?}", path, data.1);
        }
        return Ok(());
    }

    tracing::info!("Starting spis version {}", env!("CARGO_PKG_VERSION"));

    let config = SpisCfg::new()?;
    let pool = db::setup_db(&config.db_file()).await.unwrap();

    setup_processing(pool.clone(), config.clone()).await?;

    let listener = match &config.server_listener() {
        SpisServerListener::Address(address) => {
            tracing::info!("Start listening on http://{}", address);
            Listener::Tcp(TcpListener::bind(address)?)
        }
        SpisServerListener::Socket(socket) => {
            tracing::info!("Start listening on socket {}", socket);
            Listener::Socket(socket.clone())
        }
    };
    let converter = config.media_converter();

    let server = server::run(listener, pool, converter).expect("Failed to create server");
    server.await?;

    Ok(())
}

async fn setup_processing(pool: Pool<Sqlite>, config: SpisCfg) -> Result<()> {
    tracing::info!("Setup processing");

    let media_dir = config.media_dir();
    let thumb_dir = config.thumbnail_dir();
    let schedule = config.processing_schedule();
    std::fs::create_dir_all(&thumb_dir)?;

    tokio::spawn(async move {
        if config.processing_run_on_start() {
            tracing::info!("Running on-start processing");
            media::process(pool.clone(), media_dir.clone(), thumb_dir.clone(), false).await;
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
                media::process(pool, media_dir, thumb_dir, false).await;
                tracing::info!("Processing schedule finished: {}", schedule);
            });
        });
        sched_service.await;
    });
    tracing::debug!("Setup processing done");
    Ok(())
}
