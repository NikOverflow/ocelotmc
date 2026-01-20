use ocelot_macros::Packet;
use uuid::Uuid;

use crate::{
    codec::{BoundedPrefixedArray, BoundedString, PrefixedArray, VarInt},
    types::Identifier,
};

use crate::packet::types::Properties;

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct DisconnectPacket {
    //text_component: Json<TextComponent>
}

#[derive(Packet)]
#[packet(id = 0x01)]
pub struct EncryptionRequestPacket {
    server_id: BoundedString<20>,
    public_key: PrefixedArray<u8>,
    verify_token: PrefixedArray<u8>,
    should_authenticate: bool,
}

#[derive(Packet)]
#[packet(id = 0x02)]
pub struct LoginSuccessPacket {
    uuid: Uuid,
    username: BoundedString<16>,
    properties: BoundedPrefixedArray<Properties, 16>,
}

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct SetCompressionPacket {
    threshold: VarInt,
}

#[derive(Packet)]
#[packet(id = 0x04)]
pub struct LoginPluginRequestPacket {
    message_id: VarInt,
    channel: Identifier,
    data: Vec<u8>,
}

#[derive(Packet)]
#[packet(id = 0x05)]
pub struct CookieRequestPacket {
    key: Identifier,
}
