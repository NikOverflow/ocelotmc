use ocelot_macros::Packet;
use uuid::Uuid;

use crate::codec::{BoundedString, VarInt};

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ServerboundLoginStartPacket {
    name: BoundedString<16>,
    player_uuid: Uuid,
}

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct ServerboundLoginAcknowledgedPacket {}

#[derive(Packet)]
#[packet(id = 0x02)]
pub struct ClientboundLoginSuccessPacket {
    uuid: Uuid,
    username: BoundedString<16>,
    test: VarInt,
}
