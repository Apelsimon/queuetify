use crate::session_agent::{self, SearchResult};
use actix::prelude::{Message, Recipient};
use rspotify::model::TrackId;
use rspotify::model::device::Device;
use rspotify::model::enums::types::DeviceType;
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeviceInfo {
    id: String,
    name: String,
    dev_type: String
}

impl TryFrom<Device> for DeviceInfo {
    type Error = ();

    fn try_from(device: Device) -> Result<Self, Self::Error> {
        let id = match device.id {
            Some(id) => id,
            None => return Err(()),
        };

        Ok(DeviceInfo {
            id,
            name: device.name,
            dev_type: device_type_to_string(device._type)
        })
    }
}

fn device_type_to_string(dev_type: DeviceType) -> String {
    let result = match dev_type {
        DeviceType::Computer => "Computer",
        DeviceType::Tablet => "Tablet",
        DeviceType::Smartphone => "Smartphone",
        DeviceType::Speaker => "Speaker",
        DeviceType::Tv => "Tv",
        _ => "Unknown"
    };

    result.to_string()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DevicesPayload {
    pub payload: Vec<DeviceInfo>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransferResponsePayload {
    pub payload: String
}

// TODO: change name. not everything is a response
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Response {
    SearchResult(SearchResultPayload),
    Shutdown,
    StateUpdate(StateUpdatePayload),
    Devices(DevicesPayload),
    TransferResponse(TransferResponsePayload)
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct Devices {
    pub session_id: Uuid,
    pub connection_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DevicesComplete {
    pub connection_id: Uuid,
    pub devices: Vec<DeviceInfo>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Transfer {
    pub session_id: Uuid,
    pub connection_id: Uuid,
    pub device_id: String
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TransferComplete {
    pub connection_id: Uuid,
    pub result: String
}