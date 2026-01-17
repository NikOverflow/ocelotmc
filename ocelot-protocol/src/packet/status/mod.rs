use ocelot_macros::Packet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::codec::{Json, VarInt};

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: StatusResponseVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusResponsePlayers>,
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponseVersion {
    pub name: String,
    pub protocol: VarInt,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponsePlayers {
    pub max: i32,
    pub online: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<StatusResponsePlayer>>,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponsePlayer {
    pub name: String,
    pub id: Uuid,
}

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ClientboundStatusResponsePacket {
    response: Json<StatusResponse>,
}
