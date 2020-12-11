mod ws;
mod messages;
mod lobby;
mod router;

use lobby::Lobby;
use actix::Actor;

use actix_web::{App, HttpServer};
use crate::router::start_connection;
use actix_web::middleware::Logger;

use log::{info};
use std::{env, fs};
use std::path::{PathBuf};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let environment = env::var("ENV").unwrap();
    let config_log_path = env::var("LOG_PATH").unwrap_or(String::from(""));
    setup_logger(&environment, &config_log_path).unwrap();

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

fn setup_logger(environment: &str, config_log_path: &str) -> Result<(), fern::InitError> {
    let log_path: PathBuf = if config_log_path.is_empty() {
        [env::current_dir().unwrap(), PathBuf::from("tmp")].iter().collect()
    } else {
        PathBuf::from(config_log_path)
    };

    fs::create_dir_all(log_path).unwrap();


    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(format!("tmp/{}.log", environment.to_lowercase()))?)
        .apply()?;
    Ok(())
}