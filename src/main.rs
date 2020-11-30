mod ws;
mod messages;
mod lobby;
mod router;

use lobby::Lobby;
// use start_connection::start_connection as start_connection_route;
use actix::Actor;

use actix_web::{App, HttpServer};
use crate::router::start_connection;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create and spin up lobby
    let chat_server = Lobby::default().start();

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