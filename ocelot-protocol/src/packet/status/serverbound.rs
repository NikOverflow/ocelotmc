use ocelot_macros::MinecraftPacket;

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct StatusRequestPacket {}

#[derive(MinecraftPacket)]
#[packet(id = 0x01)]
pub struct PingRequestPacket {
    timestamp: i64,
}
