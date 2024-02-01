use actix_web::{dev::Server, web, App, HttpServer};
use color_eyre::{eyre::eyre, Result};
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

pub fn run(listener: Listener, pool: Pool<Sqlite>, pathfinder: PathFinder) -> Result<Server> {
    let pool = web::Data::new(pool);
    let pathfinder = web::Data::new(pathfinder);

    let server = HttpServer::new(move || {
        let mut app = App::new()
            .service(web::redirect("/", "/hx"))
            .service(hx::create_service("/hx"))
            .service(assets::create_service("/assets"));

        #[cfg(feature = "dev")]
        {
            app = app.route("/dev/ws", dev::create_socket());
        }

        app.app_data(pool.clone()).app_data(pathfinder.clone())
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
