use ocelot_macros::MinecraftCodec;
use ocelot_types::{BoundedString, ResourceLocation, VarInt, text::TextComponent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bitfield;

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: StatusResponseVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusResponsePlayers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<TextComponent>,
    //pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponseVersion {
    pub name: String,
    pub protocol: VarInt,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponsePlayers {
    pub max: i32,
    pub online: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<StatusResponsePlayer>>,
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponsePlayer {
    pub name: String,
    pub id: Uuid,
}

#[derive(MinecraftCodec)]
pub struct Properties {
    name: BoundedString<64>,
    value: BoundedString<32767>,
    signature: Option<BoundedString<1024>>,
}

#[derive(MinecraftCodec)]
pub struct RegistryEntry {
    pub id: ResourceLocation,
    pub data: Option<Vec<u8>>, // TODO: has to be nbt data
}

#[derive(MinecraftCodec)]
pub struct KnownPack {
    pub namespace: BoundedString<32767>,
    pub id: BoundedString<32767>,
    pub version: BoundedString<32767>,
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

#[derive(MinecraftCodec)]
#[codec(via = u8)]
pub enum GameEvent {
    NoRespawnBlockAvailable = 0,
    BeginRaining = 1,
    EndRaining = 2,
    ChangeGameMode = 3,
    WinGame = 4,
    DemoEvent = 5,
    ArrowHitPlayer = 6,
    RainLevelChange = 7,
    ThunderLevelChange = 8,
    PlayPufferfishStingSound = 9,
    PlayElderGuardianMobAppearance = 10,
    EnableRespawnScreen = 11,
    LimitedCrafting = 12,
    StartWaitingForLevelChunks = 13,
}

#[derive(MinecraftCodec)]
#[codec(via = i8)]
pub enum GameMode {
    Undefined = -1,
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

bitfield!(TeleportFlags, i32, {
    RelativeX => 0,
    RelativeY => 1,
    RelativeZ => 2,
    RelativeYaw => 3,
    RelativePitch => 4,
    RelativeVelocityX => 5,
    RelativeVelocityY => 6,
    RelativeVelocityZ => 7,
    RotateVelocity => 8
});
