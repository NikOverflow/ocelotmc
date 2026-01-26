use ocelot_macros::MinecraftPacket;
use ocelot_types::ResourceLocation;

use crate::{
    codec::PrefixedArray,
    packet::types::{KnownPack, RegistryEntry},
};

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct CookieRequestPacket {
    key: ResourceLocation,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x03)]
pub struct FinishConfigurationPacket {}

#[derive(MinecraftPacket)]
#[packet(id = 0x07)]
pub struct RegistryDataPacket {
    registry_id: ResourceLocation,
    entries: PrefixedArray<RegistryEntry>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x0E)]
pub struct KnownPacksPacket {
    known_packs: PrefixedArray<KnownPack>,
}
