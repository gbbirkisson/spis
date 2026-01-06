use actix_web::{App, HttpServer, dev::Server, web};
use color_eyre::{Result, eyre::eyre};
use sqlx::{Pool, Sqlite};
use std::net::TcpListener;

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
}

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{from_fn, Next},
    Error,
};

async fn log_errors(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let res = next.call(req).await?;

    if let Some(error) = res.response().error() {
        tracing::error!("Error in response: {:?}", error);
    }

    Ok(res)
}

pub fn run(listener: Listener, pool: Pool<Sqlite>, config: Config) -> Result<Server> {
    let pool = web::Data::new(pool);
    let config = web::Data::new(config);

    let server = HttpServer::new(move || {
        let mut app = App::new().wrap(from_fn(log_errors));

        app = app
            .service(web::redirect("/", "/hx"))
            .service(hx::create_service("/hx"))
            .service(assets::create_service("/assets"));

        #[cfg(feature = "dev")]
        {
            app = app.route("/dev/ws", dev::create_socket());
        }

        app.app_data(pool.clone()).app_data(config.clone())
    });

    let server = match listener {
        Listener::Tcp(listener) => server.listen(listener)?,
        Listener::Socket(file) => {
            if cfg!(not(unix)) {
                return Err(eyre!("You can only use unix sockets on unix!"));
            }
            server.bind_uds(file)?
        }
    }
    .run();

    Ok(server)
}
