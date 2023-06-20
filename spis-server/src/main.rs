use clap::Parser;
use color_eyre::Result;
use notify::Watcher;
use spis_server::{
    db, pipeline,
    server::{self, Listener},
    SpisCfg, SpisServerListener,
};
use std::{net::TcpListener, path::PathBuf};
use tokio::sync::mpsc::channel;
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

    let config = SpisCfg::new()?;

    // Parse args
    let args = Args::parse();
    tracing::debug!("Got args: {:?}", args);
    if !args.test_media.is_empty() {
        let (file_sender, file_reciever) = channel(1);
        let (media_sender, mut media_reciever) = channel(1);
        let (tx, rx) = tokio::sync::oneshot::channel();

        pipeline::setup_media_processing(
            file_reciever,
            media_sender,
            config.thumbnail_dir(),
            true,
        )?;

        tokio::spawn(async move {
            while let Some(media) = media_reciever.recv().await {
                println!("{:?} {:?}", media.path, media.data.unwrap().taken_at);
            }
            tx.send(()).unwrap();
        });

        tokio::spawn(async move {
            for file in args.test_media {
                file_sender.send((None, file)).await.unwrap()
            }
            drop(file_sender);
        });

        rx.await.unwrap();

        return Ok(());
    }

    tracing::info!("Starting spis version {}", env!("CARGO_PKG_VERSION"));

    std::fs::create_dir_all(&config.thumbnail_dir())?;

    let pool = db::setup_db(&config.db_file()).await.unwrap();

    tracing::info!("Creating channels");

    // Channel with empty objects to trigger start of jobs
    let (job_sender, job_reciever) = channel(1);

    // Channel with PathBuf to send for media processing
    let (file_sender, file_reciever) = channel(rayon::current_num_threads());

    // Channel of processed media to save to DB
    let (media_sender, media_reciever) = channel(rayon::current_num_threads());

    tracing::info!("Setting up file watcher");
    let mut file_watcher = pipeline::setup_filewatcher(file_sender.clone())?;
    file_watcher.watch(&config.media_dir(), notify::RecursiveMode::Recursive)?;

    tracing::info!("Setting up file walker");
    pipeline::setup_filewalker(
        job_reciever,
        file_sender.clone(),
        config.media_dir(),
        pool.clone(),
    )?;

    tracing::info!("Setting up media processing");
    pipeline::setup_media_processing(file_reciever, media_sender, config.thumbnail_dir(), false)?;

    pipeline::setup_db_store(pool.clone(), media_reciever)?;

    if config.processing_run_on_start() {
        job_sender.send(()).await?;
    }

    pipeline::setup_cron(job_sender.clone(), config.processing_schedule())?;

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
