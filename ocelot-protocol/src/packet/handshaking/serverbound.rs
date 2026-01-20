use ocelot_macros::Packet;

use crate::{
    codec::{BoundedString, VarInt},
    packet::types::Intent,
};

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct HandshakePacket {
    protocol_version: VarInt,
    server_address: BoundedString<255>,
    server_port: u16,
    intent: Intent,
}
