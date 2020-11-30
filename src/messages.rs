use actix::prelude::{Message, Recipient};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
// derive(Message) indicates that target is an Actor Message
// rtype(result = "SomeType") indicates that target should have particular return type after message is handled

// WS Connection responds to this message to pipe it through to the actual client
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

// Ws Connection sends this Connect message indicating desire to be connected to Lobby
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: Uuid,
    pub self_id: Uuid
}

// Ws Connection sends this Connect message indicating desire to be disconnected from Lobby
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub room_id: Uuid,
    pub id: Uuid
}

// Client sends this Message to the lobby for the lobby to echo out
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid
}

#[derive(Serialize, Deserialize)]
pub struct MessagePayload {
    pub kind: DataType,
    pub content: String
}

#[derive(Serialize, Deserialize)]
pub enum DataType {
    Connect,
    Message,
    Disconnect
}