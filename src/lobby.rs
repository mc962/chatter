use crate::messages::{
    ClientActorMessage, Connect, DataType, Disconnect, MessagePayload, WsMessage,
};
use actix::prelude::{Actor, Context, Handler, Recipient};
use log::info;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// store Socket as a recipient of Websocket Message
type Socket = Recipient<WsMessage>;

/// Holds information for user sessions and available 'rooms'
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,     // match self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>, // match room id to list of users id
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

impl Lobby {
    /// Sends a message to a client with a given id
    ///
    /// # Arguments
    /// * `message` - Message data to send to client
    /// * `id_to` - Unique uuid to identify user session/client to send message to
    fn send_message(&self, message: &str, id_to: &Uuid) {
        // sends message to client with given id, if client exists
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("Sending message failed, could not find user id");
        }
    }
}

// Make Lobby an Actor
impl Actor for Lobby {
    type Context = Context<Self>;
}

// Handler for Disconnect messages
impl Handler<Disconnect> for Lobby {
    type Result = ();

    /// Handles disconnecting a client from the held sessions and communicating disconnection
    ///   with connected sessions
    ///
    /// # Arguments
    /// * `msg` - Message with data to send to client
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // take action to disconnect client if it exists
        if self.sessions.remove(&msg.id).is_some() {
            // send client id to self
            let disconnect_data = MessagePayload {
                kind: DataType::Disconnect,
                content: msg.id.to_string(),
            };
            let payload = Value::to_string(&json!(disconnect_data));

            // get clients for room, message that particular client will be disconnected
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|user_id| self.send_message(&payload, user_id));
            // if room exists
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    //  if there are multiple clients in the lobby for that room, remove that client
                    lobby.remove(&msg.id);
                } else {
                    // if there is only 1 client in the room, remove it entirely to avoid filling up the HashMap
                    self.rooms.remove(&msg.room_id);
                }

                info!("User {} disconnected from room {}", &msg.id, &msg.room_id);
            }
        }
    }
}

// Handler for Connect messages
impl Handler<Connect> for Lobby {
    type Result = ();

    /// Handles connecting a client, adding it to a room and connected sessions
    ///
    /// # Arguments
    /// * `msg` - Message with data to send to client
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // create a room if does not exist, and add id for client to it
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new)
            .insert(msg.self_id);

        let connect_data = MessagePayload {
            kind: DataType::Connect,
            content: msg.self_id.to_string(),
        };
        let payload = Value::to_string(&json!(connect_data));

        // send to all in room that new uuid just joined
        self.rooms
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            // don't send message to current actor's id
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| self.send_message(&payload, conn_id));

        // store client address
        self.sessions.insert(msg.self_id, msg.addr);

        info!("User {} connected to room {}", &msg.self_id, &msg.lobby_id);

        // send client id to self
        self.send_message(&payload, &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    /// Handles sending a message to connected clients
    ///
    /// # Arguments
    /// * `msg` - Message with data to send to client
    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        let message_data = MessagePayload {
            kind: DataType::Message,
            content: msg.msg.to_string(),
        };

        let payload = Value::to_string(&json!(message_data));

        // if message starts with \w (whisper), send message to specific client
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&payload, &Uuid::parse_str(id_to).unwrap());
            }
        } else {
            //  if not a whisper, send to all clients in the room
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .for_each(|client| self.send_message(&payload, client));
        }
    }
}
