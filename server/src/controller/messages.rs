use actix::prelude::{Message, Recipient};
use uuid::Uuid;
use rspotify::model::TrackId;
use crate::controller::SearchResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchResultPayload {
    pub payload: SearchResult
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    SearchResult(SearchResultPayload)
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub Response);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub client_addr: Recipient<WsMessage>,
    pub session_id: Uuid,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: Uuid,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub session_id: Uuid,
    pub connection_id: Uuid,
    pub msg: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Search {
    pub query: String,
    pub session_id: Uuid,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SearchComplete {
    pub result: SearchResult,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Queue {
    pub track_id: TrackId,
    pub session_id: Uuid,
}
