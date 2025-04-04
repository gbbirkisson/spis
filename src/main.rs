use askama::Template;
use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, error::ErrorKind};
use color_eyre::Result;
use color_eyre::eyre::{Error, OptionExt, WrapErr, eyre};
use notify::Watcher;
use spis::PathFinder;
use spis::media::util::THUMBNAIL_FORMAT;
use spis::{
    db,
    pipeline::{self, JOB_TRIGGER},
    server::{self, Config, Listener},
};
use std::{net::TcpListener, path::PathBuf};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
#[allow(clippy::struct_excessive_bools)]
pub struct Spis {
    /// Path to search for media files
    #[clap(long, env = "SPIS_MEDIA_DIR", default_value = "")]
    pub media_dir: PathBuf,

    /// Path to store data
    #[clap(long, env = "SPIS_DATA_DIR", default_value = "")]
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

    /// Disable feature follow symlinks
    #[clap(long, env = "SPIS_FEATURE_FOLLOW_SYMLINKS", default_value = "true", action=ArgAction::SetFalse)]
    pub feature_follow_symlinks: bool,

    /// Disable feature no exif
    #[clap(long, env = "SPIS_FEATURE_NO_EXIF", default_value = "true", action=ArgAction::SetFalse)]
    pub feature_allow_no_exif: bool,

    #[command(subcommand)]
    command: Option<SpisCommand>,
}

#[derive(Subcommand, Debug, Clone)]
enum SpisCommand {
    /// Runs the server [default]
    Run,

    /// Test process media files
    Process {
        /// Media file to process
        #[clap(value_parser = file_exists)]
        media: Vec<PathBuf>,
    },

    /// Render configuration templates
    Template {
        #[command(subcommand)]
        template: SpisTemplate,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum SpisTemplate {
    /// Template nginx configuration
    Nginx {
        /// Nginx port
        #[arg(short, long, env = "NGINX_PORT")]
        port: u32,

        /// Full nginx config
        #[arg(short, long, default_value_t = false)]
        full: bool,
    },

    /// Template systemd configuration
    Systemd {
        /// User that should run SPIS
        #[arg(short, long)]
        user: String,

        /// Full path to SPIS binary
        #[arg(short, long)]
        bin: String,

        /// Log configuration
        #[arg(short, long, default_value = "error,spis=info")]
        log: String,
    },

    /// Template docker compose configuration
    DockerCompose {
        /// Full path to SPIS binary
        #[arg(short, long, default_value = "latest")]
        version: String,

        /// Log configuration
        #[arg(short, long, default_value = "error,spis=info")]
        log: String,
    },
}

impl Default for SpisCommand {
    fn default() -> Self {
        Self::Run
    }
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub struct ServerListener {
    /// Listen to address
    #[clap(long, group = "addr", env = "SPIS_SERVER_ADDRESS")]
    pub server_address: Option<String>,

    /// Listen to UNIX socket
    #[clap(long, group = "addr", env = "SPIS_SERVER_SOCKET")]
    pub server_socket: Option<String>,
}

fn validate_listener(config: &Spis) -> &'static str {
    match (
        &config.listener.server_address,
        &config.listener.server_socket,
    ) {
        (Some(_), None) | (None, Some(_)) => {}
        _ => {
            let mut cmd = Spis::command();
            cmd.error(
            ErrorKind::MissingRequiredArgument,
            "missing either '--server-address <SERVER_ADDRESS>' or '--server-socket <SERVER_SOCKET>'",
        )
        .exit();
        }
    }
    ""
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
        SpisCommand::Process { media } => process(config, media).await,
        SpisCommand::Run => run(config).await,
        SpisCommand::Template { template } => template_render(config, template),
    }
}

async fn process(config: Spis, media: Vec<PathBuf>) -> Result<()> {
    let (file_sender, mut media_receiver) = pipeline::setup_media_processing(
        config.thumbnail_dir(),
        config.feature_allow_no_exif,
        true,
    )
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
            .ok_or_else(|| eyre!("failed to get db path"))?,
    )
    .await
    .wrap_err("Failed to initialize DB")?;

    tracing::info!("Setting up media processing");
    let (file_sender, media_receiver) = pipeline::setup_media_processing(
        config.thumbnail_dir(),
        config.feature_allow_no_exif,
        false,
    )
    .wrap_err("Failed to setup media processing")?;

    tracing::info!("Setting up file watcher");
    let mut file_watcher = pipeline::setup_filewatcher(file_sender.clone())
        .wrap_err("Failed to setup file watcher")?;
    file_watcher
        .watch(&config.media_dir, notify::RecursiveMode::Recursive)
        .wrap_err("Failed to start file watcher")?;

    tracing::info!("Setting up file walker");
    let job_sender = pipeline::setup_filewalker(
        pool.clone(),
        config.media_dir.clone(),
        file_sender.clone(),
        config.feature_follow_symlinks,
    )
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

    validate_listener(&config);
    let listener = match (
        &config.listener.server_address,
        &config.listener.server_socket,
    ) {
        (Some(address), None) => {
            tracing::info!("Start listening on http://{}", address);
            Listener::Tcp(TcpListener::bind(address)?)
        }
        (None, Some(socket)) => {
            tracing::info!("Start listening on socket {}", socket);
            Listener::Socket(socket.clone())
        }
        _ => unreachable!(),
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

fn template_render(config: Spis, template: SpisTemplate) -> Result<()> {
    let res = match template {
        SpisTemplate::Nginx { port, full } => NginxTemplate {
            config,
            nginx_port: port,
            nginx_full: full,
        }
        .render()?,
        SpisTemplate::Systemd { user, bin, log } => SystemdTemplate {
            config,
            spis_user: user,
            spis_bin: bin,
            spis_log: log,
        }
        .render()?,
        SpisTemplate::DockerCompose { version, log } => DockerComposeTemplate {
            config,
            spis_version: version,
            spis_log: log,
        }
        .render()?,
    };
    println!("{res}");
    Ok(())
}

#[derive(Template)]
#[template(path = "config/nginx.conf", escape = "none")]
struct NginxTemplate {
    config: Spis,
    nginx_port: u32,
    nginx_full: bool,
}

#[derive(Template)]
#[template(path = "config/systemd.service", escape = "none")]
struct SystemdTemplate {
    config: Spis,
    spis_user: String,
    spis_bin: String,
    spis_log: String,
}

#[derive(Template)]
#[template(path = "config/docker-compose.yml", escape = "none")]
struct DockerComposeTemplate {
    config: Spis,
    spis_version: String,
    spis_log: String,
}
