use ocelot_macros::Packet;

use crate::{codec::Json, packet::types::StatusResponse};

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ClientboundStatusResponsePacket {
    response: Json<StatusResponse>,
}

#[derive(Packet)]
#[packet(id = 0x01)]
pub struct ClientboundPongResponsePacket {
    timestamp: i64,
}

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ServerboundStatusRequestPacket {}

#[derive(Packet)]
#[packet(id = 0x01)]
pub struct ServerboundPingRequestPacket {
    timestamp: i64,
}
