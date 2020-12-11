use crate::lobby::Lobby;
use crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::{fut, ActorContext};
use actix::{Actor, ActorFuture, Addr, ContextFutureSpawner, Running, StreamHandler, WrapFuture};
use actix::{AsyncContext, Handler};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConn {
    // points to the particular 'room' that the Socket exists in
    room: Uuid,
    // address of the lobby in which the Socket actually exists in
    lobby_addr: Addr<Lobby>,
    // time since last heartbeat was received, helps determine if the socket is still alive
    hb: Instant,
    // id assigned to user(?) by the socket, helps for private messaging
    id: Uuid,
}

impl WsConn {
    pub fn new(room: Uuid, lobby: Addr<Lobby>) -> WsConn {
        WsConn {
            id: Uuid::new_v4(),
            room,
            hb: Instant::now(),
            lobby_addr: lobby,
        }
    }

    /// Emits heartbeat ensuring connection is maintained
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // Run heartbeat action on a set interval
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // if time since last heartbeat exceeds timeout setting, then drop connection
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting due to failed heartbeat");
                act.lobby_addr.do_send(Disconnect {
                    id: act.id,
                    room_id: act.room,
                });
                ctx.stop();
                return;
            }

            // send ping message ensuring heartbeat is maintained
            ctx.ping(b"PING");
        });
    }
}

impl Actor for WsConn {
    // Context in which Actor lives, in this case Context is a _Websocket Context_,
    //   and so should have those capabilities
    type Context = ws::WebsocketContext<Self>;

    /// Creates WS Actor
    fn started(&mut self, ctx: &mut Self::Context) {
        // triggers Heartbeat loop on an interval
        self.hb(ctx);

        // Send a Connect message to lobby address, giving information about the connector
        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                lobby_id: self.room,
                self_id: self.id,
            })
            .into_actor(self)
            // await Connect message, allowing to happen asynchronously
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    /// Destroys WS Actor
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Send message to lobby to disconnect from lobby
        self.lobby_addr.do_send(Disconnect {
            id: self.id,
            room_id: self.room,
        });
        // Stop Actor, even if Disconnect message was not received
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Match actions to all possible WebSocket messages
        match msg {
            // If incoming message (to server Actor) is a Ping,
            //   update recorded heartbeat and respond to the ping
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            // If incoming message (to client Actor) is a Pong,
            //   update recorded heartbeat
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            // If incoming message is a Binary, send it to WebSocket context to figure out what
            //   to do it it (unlikely message to be triggered)
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            // If incoming message is a Close message, just stop the Actor
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            // If incoming message is a Continuation message (for cases when payload does not fit into one message),
            //   do not respond to these messages
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            // If incoming message is a no-op, then take no action
            Ok(ws::Message::Nop) => (),
            // If incoming message is text (most likely message),
            //   send it to the lobby so that it may handle the message
            Ok(Text(s)) => self.lobby_addr.do_send(ClientActorMessage {
                id: self.id,
                msg: s,
                room_id: self.room,
            }),
            // On error, panic (TODO handle this better)
            Err(e) => panic!(e),
        }

        impl Handler<WsMessage> for WsConn {
            type Result = ();

            // If server sends a WebSocket Message message to Actor's mailbox, send it straight to client
            fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
                ctx.text(msg.0)
            }
        }
    }
}
