use ocelot_macros::MinecraftCodec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::codec::{BoundedString, VarInt};

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: StatusResponseVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusResponsePlayers>,
    //pub description: Option<TextComponent>,
    //pub favicon: Option<String>,
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

#[derive(MinecraftCodec)]
pub struct Properties {
    name: BoundedString<64>,
    value: BoundedString<32767>,
    signature: Option<BoundedString<1024>>,
}
