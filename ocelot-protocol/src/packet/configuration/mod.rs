use ocelot_macros::Packet;

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct ClientboundFinishConfigurationPacket {}
