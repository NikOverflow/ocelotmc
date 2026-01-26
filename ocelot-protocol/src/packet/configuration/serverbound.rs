use ocelot_macros::MinecraftPacket;
use ocelot_types::{BoundedString, ResourceLocation};

use crate::{
    codec::PrefixedArray,
    packet::types::{ChatMode, KnownPack, MainHand, ParticleStatus},
};

#[derive(MinecraftPacket)]
#[packet(id = 0x00)]
pub struct ClientInformationPacket {
    locale: BoundedString<16>,
    view_distance: i8,
    chat_mode: ChatMode,
    chat_colors: bool,
    displayed_skin_parts: u8,
    main_hand: MainHand,
    enable_text_filtering: bool,
    allow_server_listings: bool,
    particle_status: ParticleStatus,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x02)]
pub struct PluginMessagePacket {
    channel: ResourceLocation,
    data: Vec<u8>,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x03)]
pub struct AcknowledgeFinishConfigurationPacket {}

#[derive(MinecraftPacket)]
#[packet(id = 0x07)]
pub struct KnownPacksPacket {
    known_packs: PrefixedArray<KnownPack>,
}
