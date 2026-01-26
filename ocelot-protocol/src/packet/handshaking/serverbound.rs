use ocelot_macros::MinecraftPacket;
use ocelot_types::{BoundedString, VarInt};

use crate::packet::types::Intent;

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct HandshakePacket {
    protocol_version: VarInt,
    server_address: BoundedString<255>,
    server_port: u16,
    intent: Intent,
}
