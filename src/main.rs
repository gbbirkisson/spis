use clap::{ArgAction, Args, Parser, Subcommand};
use color_eyre::eyre::{eyre, Error, OptionExt, WrapErr};
use color_eyre::Result;
use notify::Watcher;
use spis::media::util::THUMBNAIL_FORMAT;
use spis::PathFinder;
use spis::{
    db,
    pipeline::{self, JOB_TRIGGER},
    server::{self, Config, Listener},
};
use std::{net::TcpListener, path::PathBuf};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn dir_exists(s: &str) -> Result<PathBuf, String> {
    let pathbuf = PathBuf::from(s);
    if pathbuf.is_dir() {
        Ok(pathbuf)
    } else {
        Err(format!("directory '{s}' does not exist"))
    }
}

fn file_exists(s: &str) -> Result<PathBuf, String> {
    let pathbuf = PathBuf::from(s);
    if pathbuf.is_file() {
        Ok(pathbuf)
    } else {
        Err(format!("file '{s}' does not exist"))
    }
}

/// Simple private image server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Spis {
    /// Path to search for media files
    #[clap(long, env = "SPIS_MEDIA_DIR", value_parser = dir_exists)]
    pub media_dir: PathBuf,

    /// Path to store data
    #[clap(long, env = "SPIS_DATA_DIR", value_parser = dir_exists)]
    pub data_dir: PathBuf,

    /// Schedule to run processing on
    #[clap(long, env = "SPIS_PROCESSING_SCHEDULE", default_value = "0 0 2 * * *")]
    pub processing_schedule: String,

    /// Run processing on start
    #[clap(long, env = "SPIS_PROCESSING_RUN_ON_START", default_value = "false")]
    pub processing_run_on_start: bool,

    /// Path webserver will serve media on
    #[clap(long, env = "SPIS_API_MEDIA_PATH", default_value = "/assets/media")]
    pub api_media_path: String,

    /// Path webserver will serve thumbnails on
    #[clap(
        long,
        env = "SPIS_API_THUMBNAIL_PATH",
        default_value = "/assets/thumbnails"
    )]
    pub api_thumbnail_path: String,

    #[command(flatten)]
    pub listener: ServerListener,

    /// Disable feature favorite
    #[clap(long, env = "SPIS_FEATURE_FAVORITE", default_value = "true", action=ArgAction::SetFalse)]
    pub feature_favorite: bool,

    /// Disable feature archive
    #[clap(long, env = "SPIS_FEATURE_ARCHIVE", default_value = "true", action=ArgAction::SetFalse)]
    pub feature_archive: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Runs the server (default)
    Run,

    /// Test process media files
    Process {
        /// Media file to process
        #[clap(value_parser = file_exists)]
        media: Vec<PathBuf>,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Run
    }
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct ServerListener {
    /// Listen to address
    #[clap(long, group = "addr", env = "SPIS_SERVER_ADDRESS")]
    pub server_address: Option<String>,

    /// Listen to UNIX socket
    #[clap(long, group = "addr", env = "SPIS_SERVER_SOCKET")]
    pub server_socket: Option<String>,
}

impl TryFrom<&Spis> for PathFinder {
    type Error = Error;

    fn try_from(value: &Spis) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value
                .media_dir
                .to_str()
                .ok_or_eyre("failed to convert media dir to str")?,
            &value.api_media_path,
            &value.api_thumbnail_path,
            THUMBNAIL_FORMAT,
        ))
    }
}

impl Spis {
    #[must_use]
    pub fn thumbnail_dir(&self) -> PathBuf {
        self.data_dir.join("thumbnails")
    }

    #[must_use]
    pub fn db_file(&self) -> PathBuf {
        self.data_dir.join("spis.db")
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
    color_eyre::install().wrap_err("Failed to install color_eyre")?;

    // Load config and run
    let config = Spis::parse();
    tracing::debug!("Got config: {:?}", config);
    match config.command.clone().unwrap_or_default() {
        Command::Process { media } => process(config, media).await,
        Command::Run => run(config).await,
    }
}

async fn process(config: Spis, media: Vec<PathBuf>) -> Result<()> {
    let (file_sender, mut media_receiver) =
        pipeline::setup_media_processing(config.thumbnail_dir(), true)
            .wrap_err("Failed to setup media processing")?;

    let (done_sender, done_receiver) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        while let Some(media) = media_receiver.recv().await {
            println!(
                "{:?} {:?}",
                media.path,
                media.data.expect("Should get data").taken_at
            );
        }
        done_sender.send(()).expect("send failed");
    });

    tokio::spawn(async move {
        for file in media {
            file_sender.send((None, file)).await.expect("send failed");
        }
        drop(file_sender);
    });

    done_receiver.await.expect("receive failed");

    Ok(())
}

async fn run(config: Spis) -> Result<()> {
    std::fs::create_dir_all(config.thumbnail_dir())?;

    tracing::info!("Starting spis version {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Media dir: {:?}", config.media_dir);
    tracing::info!("Thumb dir: {:?}", config.thumbnail_dir());
    tracing::info!("Data file: {:?}", config.db_file());

    tracing::info!("Setting up DB");
    let pool = db::setup_db(
        config
            .db_file()
            .to_str()
            .ok_or(eyre!("failed to get db path"))?,
    )
    .await
    .wrap_err("Failed to initialize DB")?;

    tracing::info!("Setting up media processing");
    let (file_sender, media_receiver) =
        pipeline::setup_media_processing(config.thumbnail_dir(), false)
            .wrap_err("Failed to setup media processing")?;

    tracing::info!("Setting up file watcher");
    let mut file_watcher = pipeline::setup_filewatcher(file_sender.clone())
        .wrap_err("Failed to setup file watcher")?;
    file_watcher
        .watch(&config.media_dir, notify::RecursiveMode::Recursive)
        .wrap_err("Failed to start file watcher")?;

    tracing::info!("Setting up file walker");
    let job_sender =
        pipeline::setup_filewalker(pool.clone(), config.media_dir.clone(), file_sender.clone())
            .wrap_err("Failed to setup file walker")?;

    pipeline::setup_db_store(pool.clone(), media_receiver).wrap_err("Failed to setup db store")?;

    if config.processing_run_on_start {
        job_sender
            .send(JOB_TRIGGER)
            .await
            .wrap_err("Failed to trigger job")?;
    }

    pipeline::setup_cron(job_sender.clone(), &config.processing_schedule)
        .wrap_err("Failed to setup cron job")?;

    let listener = match (
        &config.listener.server_address,
        &config.listener.server_socket,
    ) {
        (Some(address), _) => {
            tracing::info!("Start listening on http://{}", address);
            Listener::Tcp(TcpListener::bind(address)?)
        }
        (None, Some(socket)) => {
            tracing::info!("Start listening on socket {}", socket);
            Listener::Socket(socket.clone())
        }
        _ => return Err(eyre!("neither address nor socket was provided")),
    };

    let config = Config {
        features: server::Features {
            archive_allow: config.feature_archive,
            favorite_allow: config.feature_favorite,
        },
        pathfinder: (&config).try_into()?,
    };
    let server = server::run(listener, pool, config).expect("Failed to create server");
    server.await?;

    Ok(())
}
