use ocelot_macros::MinecraftPacket;
use ocelot_types::text::TextComponent;
use ocelot_types::{BoundedString, ResourceLocation, VarInt};
use uuid::Uuid;

use crate::codec::{BoundedPrefixedArray, Json, PrefixedArray};

use crate::packet::types::Properties;

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct DisconnectPacket {
    text_component: Json<TextComponent>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x01)]
pub struct EncryptionRequestPacket {
    server_id: BoundedString<20>,
    public_key: PrefixedArray<u8>,
    verify_token: PrefixedArray<u8>,
    should_authenticate: bool,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x02)]
pub struct LoginSuccessPacket {
    uuid: Uuid,
    username: BoundedString<16>,
    properties: BoundedPrefixedArray<Properties, 16>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x03)]
pub struct SetCompressionPacket {
    threshold: VarInt,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x04)]
pub struct LoginPluginRequestPacket {
    message_id: VarInt,
    channel: ResourceLocation,
    data: Vec<u8>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x05)]
pub struct CookieRequestPacket {
    key: ResourceLocation,
}
