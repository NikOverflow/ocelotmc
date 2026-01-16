use ocelot_macros::Packet;

use crate::codec::BoundedString;

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct ClientboundFinishConfigurationPacket {}

#[derive(Packet)]
#[packet(id = 0x02)]
pub struct ServerboundPluginMessage {
    channel: BoundedString<32767>,
    data: Vec<u8>,
}
