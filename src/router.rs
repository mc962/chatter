use crate::ws::WsConn;
use crate::lobby::Lobby;
use actix::Addr;
use actix_web::{get, web::Data, web::Path, web::Payload, Error, HttpResponse, HttpRequest};
use actix_web_actors::ws;
use uuid::Uuid;

/// Route to start Ws connection with group
///
/// # Arguments
/// * `req` - Incoming HTTP request, will be upgraded to Websocket request
/// * `stream` - Stream of data chunks from incoming request, to be passed to Ws
/// * `srv` - Lobby Actor to connect to, based on group_id of incoming connection
#[get("/{group_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    Path(group_id): Path<Uuid>,
    srv: Data<Addr<Lobby>>
) -> Result<HttpResponse, Error> {
    // create new Websocket Connection with a reference to the Lobby Actor
    let ws = WsConn::new(group_id, srv.get_ref().clone());

    // upgrade request to Websocket request, leading to open, persistent connection
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}