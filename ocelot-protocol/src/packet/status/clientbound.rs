use ocelot_macros::MinecraftPacket;

use crate::{codec::Json, packet::types::StatusResponse};

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct StatusResponsePacket {
    response: Json<StatusResponse>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x01)]
pub struct PongResponsePacket {
    timestamp: i64,
}
