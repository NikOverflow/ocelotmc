use ocelot_macros::{MinecraftCodec, Packet};
use uuid::Uuid;

use crate::codec::{BoundedPrefixedArray, BoundedString};

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ServerboundLoginStartPacket {
    name: BoundedString<16>,
    player_uuid: Uuid,
}

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct ServerboundLoginAcknowledgedPacket {}

#[derive(MinecraftCodec)]
pub struct Properties {
    name: BoundedString<64>,
    value: BoundedString<32767>,
}

#[derive(Packet)]
#[packet(id = 0x02)]
pub struct ClientboundLoginSuccessPacket {
    uuid: Uuid,
    username: BoundedString<16>,
    properties: BoundedPrefixedArray<Properties, 16>,
}
