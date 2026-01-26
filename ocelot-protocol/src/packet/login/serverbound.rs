use ocelot_macros::MinecraftPacket;
use ocelot_types::{BoundedString, ResourceLocation, VarInt};
use uuid::Uuid;

use crate::codec::{BoundedPrefixedArray, PrefixedArray};

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct LoginStartPacket {
    name: BoundedString<16>,
    player_uuid: Uuid,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x01)]
pub struct EncryptionResponsePacket {
    shared_secret: PrefixedArray<u8>,
    verify_token: PrefixedArray<u8>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x02)]
pub struct LoginPluginResponsePacket {
    message_id: VarInt,
    data: Option<Vec<u8>>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x03)]
pub struct LoginAcknowledgedPacket {}

#[derive(MinecraftPacket)]
#[packet(id = 0x04)]
pub struct CookieResponsePacket {
    key: ResourceLocation,
    payload: Option<BoundedPrefixedArray<i8, 5120>>,
}
