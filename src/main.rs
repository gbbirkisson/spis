use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};
use notify::Watcher;
use spis::{
    db,
    pipeline::{self, JOB_TRIGGER},
    server::{self, Listener},
    SpisCfg, SpisServerListener,
};
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
    tracing::info!("Starting spis version {}", env!("CARGO_PKG_VERSION"));

    color_eyre::install().wrap_err("Failed to install color_eyre")?;

    let args = Args::parse();
    tracing::debug!("Got args: {:?}", args);

    let config = SpisCfg::new().wrap_err("Failed to read configuration")?;
    std::fs::create_dir_all(config.thumbnail_dir())?;

    if !args.test_media.is_empty() {
        let (file_sender, mut media_reciever) =
            pipeline::setup_media_processing(config.thumbnail_dir(), true)
                .wrap_err("Failed to setup media processing")?;

        let (done_sender, done_reciever) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            while let Some(media) = media_reciever.recv().await {
                println!(
                    "{:?} {:?}",
                    media.path,
                    media.data.expect("Should get data").taken_at
                );
            }
            done_sender.send(()).expect("send failed");
        });

        tokio::spawn(async move {
            for file in args.test_media {
                file_sender.send((None, file)).await.expect("send failed");
            }
            drop(file_sender);
        });

        done_reciever.await.expect("recieve failed");

        return Ok(());
    }

    tracing::info!("Media dir: {:?}", config.media_dir());
    tracing::info!("Thumb dir: {:?}", config.thumbnail_dir());
    tracing::info!("Data file: {:?}", config.db_file());

    tracing::info!("Setting up DB");
    let pool = db::setup_db(&config.db_file())
        .await
        .wrap_err("Failed to initialize DB")?;

    tracing::info!("Setting up media processing");
    let (file_sender, media_reciever) =
        pipeline::setup_media_processing(config.thumbnail_dir(), false)
            .wrap_err("Failed to setup media processing")?;

    tracing::info!("Setting up file watcher");
    let mut file_watcher = pipeline::setup_filewatcher(file_sender.clone())
        .wrap_err("Failed to setup file watcher")?;
    file_watcher
        .watch(&config.media_dir(), notify::RecursiveMode::Recursive)
        .wrap_err("Failed to start file watcher")?;

    tracing::info!("Setting up file walker");
    let job_sender =
        pipeline::setup_filewalker(pool.clone(), config.media_dir(), file_sender.clone())
            .wrap_err("Failed to setup file walker")?;

    pipeline::setup_db_store(pool.clone(), media_reciever).wrap_err("Failed to setup db store")?;

    if config.processing_run_on_start() {
        job_sender
            .send(JOB_TRIGGER)
            .await
            .wrap_err("Failed to trigger job")?;
    }

    pipeline::setup_cron(job_sender.clone(), config.processing_schedule())
        .wrap_err("Failed to setup cron job")?;

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

    let pathfinder = config.pathfinder();
    let server = server::run(listener, pool, pathfinder).expect("Failed to create server");
    server.await?;

    Ok(())
}
