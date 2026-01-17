use ocelot_macros::{MinecraftCodec, Packet};

use crate::codec::{BoundedString, PrefixedArray, VarInt};

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct ClientboundFinishConfigurationPacket {}

#[derive(Packet)]
#[packet(id = 0x02)]
pub struct ServerboundPluginMessage {
    channel: BoundedString<32767>,
    data: Vec<u8>,
}

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum ChatMode {
    Enabled = 0,
    CommandsOnly = 1,
    Hidden = 2,
}

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum MainHand {
    Left = 0,
    Right = 1,
}

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum ParticleStatus {
    All = 0,
    Decreased = 1,
    Minimal = 2,
}

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ServerboundClientInformationPacket {
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

#[derive(Packet)]
#[packet(id = 0x03)]
pub struct ServerboundAcknowledgeFinishConfigurationPacket {}

#[derive(MinecraftCodec)]
pub struct KnownPacks {
    pub namespace: BoundedString<32767>,
    pub id: BoundedString<32767>,
    pub version: BoundedString<32767>,
}

#[derive(Packet)]
#[packet(id = 0x0E)]
pub struct ClientboundKnownPacksPacket {
    known_packs: PrefixedArray<KnownPacks>,
}

#[derive(Packet)]
#[packet(id = 0x07)]
pub struct ServerboundKnownPacksPacket {
    known_packs: PrefixedArray<KnownPacks>,
}
