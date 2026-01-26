use ocelot_macros::MinecraftPacket;

#[derive(MinecraftPacket)]
#[packet(id = 0x0C)]
pub struct ClientTickEndPacket {}
