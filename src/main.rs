use askama::Template;
use clap::{ArgAction, Args as ClapArgs, CommandFactory, Parser, Subcommand, error::ErrorKind};
use color_eyre::Result;
use color_eyre::eyre::{Error, OptionExt, WrapErr, ensure, eyre};
use notify::Watcher;
use serde::Deserialize;
use spis::media::util::THUMBNAIL_FORMAT;
use spis::{CustomCommand, PathFinder};
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
pub struct SpisArgs {
    /// Path to configuration file
    #[clap(short, long, env = "SPIS_CONFIG_FILE", value_parser = file_exists)]
    pub config: Option<PathBuf>,

    /// Path to search for media files
    #[clap(long, env = "SPIS_MEDIA_DIR")]
    pub media_dir: Option<PathBuf>,

    /// Path to store data
    #[clap(long, env = "SPIS_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// Schedule to run processing on (default: "0 0 2 * * *")
    #[clap(long, env = "SPIS_PROCESSING_SCHEDULE")]
    pub processing_schedule: Option<String>,

    /// Run processing on start (default: false)
    #[clap(long, env = "SPIS_PROCESSING_RUN_ON_START", action = ArgAction::SetTrue)]
    pub processing_run_on_start: Option<bool>,

    /// Path webserver will serve media on (default: "/assets/media")
    #[clap(long, env = "SPIS_API_MEDIA_PATH")]
    pub api_media_path: Option<String>,

    /// Path webserver will serve thumbnails on (default: "/assets/thumbnails")
    #[clap(long, env = "SPIS_API_THUMBNAIL_PATH")]
    pub api_thumbnail_path: Option<String>,

    #[command(flatten)]
    pub listener: ServerListener,

    /// Disable feature favorite (default: true)
    #[clap(long, env = "SPIS_FEATURE_FAVORITE", action=ArgAction::SetFalse)]
    pub feature_favorite: Option<bool>,

    /// Disable feature archive (default: true)
    #[clap(long, env = "SPIS_FEATURE_ARCHIVE", action=ArgAction::SetFalse)]
    pub feature_archive: Option<bool>,

    /// Disable feature follow symlinks (default: true)
    #[clap(long, env = "SPIS_FEATURE_FOLLOW_SYMLINKS", action=ArgAction::SetFalse)]
    pub feature_follow_symlinks: Option<bool>,

    /// Disable feature no exif (default: true)
    #[clap(long, env = "SPIS_FEATURE_NO_EXIF", action=ArgAction::SetFalse)]
    pub feature_allow_no_exif: Option<bool>,

    /// Slideshow duration in seconds (default: 5)
    #[clap(long, env = "SPIS_SLIDESHOW_DURATION_SECONDS")]
    pub slideshow_duration_seconds: Option<usize>,

    #[command(subcommand)]
    command: Option<SpisCommand>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SpisConfig {
    pub dirs: Option<DirsConfig>,
    pub processing: Option<ProcessingConfig>,
    pub api: Option<ApiConfig>,
    pub listener: Option<ListenerConfig>,
    pub features: Option<FeaturesConfig>,
    pub slideshow: Option<SlideshowConfig>,
    pub custom_command: Option<Vec<CustomCommandConfig>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct CustomCommandConfig {
    pub name: String,
    pub cmd: Vec<String>,
    pub fa_icon: String,
    pub hotkey: Option<char>,
}

impl TryFrom<CustomCommandConfig> for CustomCommand {
    type Error = Error;

    fn try_from(value: CustomCommandConfig) -> std::result::Result<Self, Self::Error> {
        ensure!(
            !value.name.contains(' '),
            "command names cannot contain spaces"
        );
        ensure!(
            value.name.to_lowercase() == value.name,
            "command names must be lowercase"
        );
        ensure!(!value.cmd.is_empty(), "command cannot be empty");
        if let Some(key) = value.hotkey {
            ensure!(key.is_lowercase(), "hotkey must be a lowercase character");
        }
        Ok(Self {
            name: value.name,
            cmd: value.cmd,
            fa_icon: value.fa_icon,
            hotkey: value.hotkey,
        })
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct DirsConfig {
    pub media: Option<PathBuf>,
    pub data: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ProcessingConfig {
    pub schedule: Option<String>,
    pub run_on_start: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ApiConfig {
    pub media_path: Option<String>,
    pub thumbnail_path: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListenerConfig {
    pub address: Option<String>,
    pub socket: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct FeaturesConfig {
    pub favorite: Option<bool>,
    pub archive: Option<bool>,
    pub follow_symlinks: Option<bool>,
    pub allow_no_exif: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SlideshowConfig {
    pub duration_seconds: Option<usize>,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Spis {
    pub media_dir: PathBuf,
    pub data_dir: PathBuf,
    pub processing_schedule: String,
    pub processing_run_on_start: bool,
    pub api_media_path: String,
    pub api_thumbnail_path: String,
    pub listener: ServerListener,
    pub feature_favorite: bool,
    pub feature_archive: bool,
    pub feature_follow_symlinks: bool,
    pub feature_allow_no_exif: bool,
    pub slideshow_duration_seconds: usize,
    pub custom_commands: Vec<CustomCommandConfig>,
    pub command: Option<SpisCommand>,
}

#[derive(Subcommand, Default, Debug, Clone)]
pub enum SpisCommand {
    /// Runs the server [default]
    #[default]
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
pub enum SpisTemplate {
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

#[derive(ClapArgs, Debug, Deserialize, Clone, Default)]
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
            let mut cmd = SpisArgs::command();
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
    pub fn resolve(args: SpisArgs) -> Result<Self> {
        let config_file = args.config.clone();
        let config: SpisConfig = if let Some(path) = config_file {
            let content = std::fs::read_to_string(&path)
                .wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;
            toml::from_str(&content)
                .wrap_err_with(|| format!("Failed to parse config file: {}", path.display()))?
        } else {
            SpisConfig::default()
        };

        // Merge logic: Config > Args > Default
        let dirs = config.dirs.unwrap_or_default();
        let media_dir = dirs
            .media
            .or(args.media_dir)
            .unwrap_or_else(|| PathBuf::from(""));
        let data_dir = dirs
            .data
            .or(args.data_dir)
            .unwrap_or_else(|| PathBuf::from(""));

        let processing = config.processing.unwrap_or_default();
        let processing_schedule = processing
            .schedule
            .or(args.processing_schedule)
            .unwrap_or_else(|| "0 0 2 * * *".to_string());
        let processing_run_on_start = processing
            .run_on_start
            .or(args.processing_run_on_start)
            .unwrap_or(false);

        let api = config.api.unwrap_or_default();
        let api_media_path = api
            .media_path
            .or(args.api_media_path)
            .unwrap_or_else(|| "/assets/media".to_string());
        let api_thumbnail_path = api
            .thumbnail_path
            .or(args.api_thumbnail_path)
            .unwrap_or_else(|| "/assets/thumbnails".to_string());

        // Listener merging
        let listener_config = config.listener.unwrap_or_default();
        let listener = ServerListener {
            server_address: listener_config.address.or(args.listener.server_address),
            server_socket: listener_config.socket.or(args.listener.server_socket),
        };

        let features = config.features.unwrap_or_default();
        let feature_favorite = features.favorite.or(args.feature_favorite).unwrap_or(true);
        let feature_archive = features.archive.or(args.feature_archive).unwrap_or(true);
        let feature_follow_symlinks = features
            .follow_symlinks
            .or(args.feature_follow_symlinks)
            .unwrap_or(true);
        let feature_allow_no_exif = features
            .allow_no_exif
            .or(args.feature_allow_no_exif)
            .unwrap_or(true);

        let slideshow = config.slideshow.unwrap_or_default();
        let slideshow_duration_seconds = slideshow
            .duration_seconds
            .or(args.slideshow_duration_seconds)
            .unwrap_or(5);

        let custom_commands = config.custom_command.unwrap_or_default();

        Ok(Self {
            media_dir,
            data_dir,
            processing_schedule,
            processing_run_on_start,
            api_media_path,
            api_thumbnail_path,
            listener,
            feature_favorite,
            feature_archive,
            feature_follow_symlinks,
            feature_allow_no_exif,
            slideshow_duration_seconds,
            custom_commands,
            command: args.command,
        })
    }

    #[must_use]
    pub fn thumbnail_dir(&self) -> PathBuf {
        self.data_dir.join("thumbnails")
    }

    #[must_use]
    pub fn db_file(&self) -> PathBuf {
        self.data_dir.join("spis.db")
    }
}

#[allow(clippy::cognitive_complexity)]
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
    let args = SpisArgs::parse();
    let config = Spis::resolve(args)?;

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

#[allow(clippy::cognitive_complexity)]
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

    let pathfinder = (&config).try_into()?;
    let config = Config {
        root_path: config
            .media_dir
            .to_str()
            .ok_or_eyre("failed to convert media dir to str")?
            .to_string(),
        features: server::Features {
            archive_allow: config.feature_archive,
            favorite_allow: config.feature_favorite,
            slideshow_duration: config.slideshow_duration_seconds,
            custom_commands: config
                .custom_commands
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        },
        pathfinder,
    };
    server::run(listener, pool, config).await?;

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
