mod lobby;
mod logging;
mod messages;
mod router;
mod ws;

use actix::Actor;
use lobby::Lobby;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use router::start_connection;

use log::info;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let environment = env::var("ENV").unwrap();
    let config_log_path = env::var("LOG_PATH").unwrap_or(String::from(""));
    logging::setup_logger(&environment, &config_log_path).unwrap();

    // create and spin up lobby to hold sessions/clients
    let chat_server = Lobby::default().start();

    info!(target: "app", "=> Booting server");
    info!(target: "app", "=> Starting application version {} in {}", &env!("CARGO_PKG_VERSION"), &environment);

    // spin up an HTTP server to run app in
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(start_connection)
            // register the lobby
            .data(chat_server.clone())
    })
    .bind(format!("127.0.0.1:{}", env::var("PORT").unwrap()))?
    .run()
    .await
}
