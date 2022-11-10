use crate::session_agent::{self, SearchResult};
use actix::prelude::{Message, Recipient};
use rspotify::model::TrackId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchResultPayload {
    pub payload: SearchResult,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateUpdatePayload {
    pub payload: session_agent::State,
}

// TODO: change name. not everything is a response
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Response {
    SearchResult(SearchResultPayload),
    Shutdown,
    StateUpdate(StateUpdatePayload),
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
pub struct Search {
    pub query: String,
    pub session_id: Uuid,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SearchComplete {
    pub result: SearchResultPayload,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Queue {
    pub track_id: TrackId,
    pub session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct State {
    pub session_id: Uuid,
    pub connection_id: Uuid
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Vote {
    pub track_id: TrackId,
    pub session_id: Uuid,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StateUpdate {
    pub update: StateUpdatePayload,
    pub session_id: Uuid,
    pub connection_id: Option<Uuid>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Refresh {
    pub duration: Duration,
    pub session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Kill {
    pub session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct KillComplete {
    pub session_id: Uuid,
}