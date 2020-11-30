mod ws;
mod messages;
mod lobby;
mod router;

use lobby::Lobby;
use actix::Actor;

use actix_web::{App, HttpServer};
use crate::router::start_connection;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create and spin up lobby to hold sessions/clients
    let chat_server = Lobby::default().start();

    // spin up an HTTP server to run app in
    HttpServer::new(move || {
        App::new()
            .service(start_connection)
            // register the lobby
            .data(chat_server.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}