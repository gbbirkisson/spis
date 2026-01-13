use axum::{Router, response::Redirect, routing::get};
use color_eyre::Result;
use sqlx::{Pool, Sqlite};
use std::net::TcpListener;
use std::sync::Arc;
use tokio::net::UnixListener;
use tower_http::trace::TraceLayer;

use crate::PathFinder;

mod assets;
#[cfg(feature = "dev")]
mod dev;
mod hx;

pub enum Listener {
    Tcp(TcpListener),
    Socket(String),
}

pub struct Config {
    pub pathfinder: PathFinder,
    pub features: Features,
}

pub struct Features {
    pub archive_allow: bool,
    pub favorite_allow: bool,
    pub slideshow_duration: usize,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Sqlite>,
    pub config: Arc<Config>,
}

pub async fn run(listener: Listener, pool: Pool<Sqlite>, config: Config) -> Result<()> {
    let state = AppState {
        pool,
        config: Arc::new(config),
    };

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/hx") }))
        .nest("/hx", hx::create_router())
        .nest("/assets", assets::create_router());

    #[cfg(feature = "dev")]
    let app = app.nest("/dev", dev::create_router());

    let app = app.layer(TraceLayer::new_for_http()).with_state(state);

    match listener {
        Listener::Tcp(std_listener) => {
            std_listener.set_nonblocking(true)?;
            let listener = tokio::net::TcpListener::from_std(std_listener)?;
            axum::serve(listener, app).await?;
        }
        Listener::Socket(path) => {
            // Remove existing socket if it exists
            if std::path::Path::new(&path).exists() {
                std::fs::remove_file(&path)?;
            }
            let listener = UnixListener::bind(path)?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
