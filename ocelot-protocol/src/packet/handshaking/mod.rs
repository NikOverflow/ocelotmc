use ocelot_macros::{MinecraftCodec, Packet};

use crate::codec::{BoundedString, VarInt};

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum Intent {
    STATUS = 1,
    LOGIN = 2,
    TRANSFER = 3,
}

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ServerboundHandshakePacket {
    protocol_version: VarInt,
    server_address: BoundedString<255>,
    server_port: u16,
    intent: Intent,
}
