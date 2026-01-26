use ocelot_macros::MinecraftPacket;
use ocelot_types::{ResourceLocation, VarInt};

use crate::{
    codec::{MinecraftCodec, PrefixedArray},
    packet::types::{GameEvent, GameMode, TeleportFlags},
    types::Position,
};

#[derive(MinecraftPacket)]
#[packet(id = 0x26)]
pub struct GameEventPacket {
    event: GameEvent,
    value: f32,
}

// TODO: change this at some point
pub struct DeathLocation {
    dimension_name: ResourceLocation,
    location: Position,
}

impl MinecraftCodec for DeathLocation {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.dimension_name.encode(writer)?;
        self.location.encode(writer)
    }

    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            dimension_name: ResourceLocation::decode(reader)?,
            location: Position::decode(reader)?,
        })
    }
}

#[derive(MinecraftPacket)]
#[packet(id = 0x30)]
pub struct LoginPacket {
    entity_id: i32,
    hardcore: bool,
    dimensions: PrefixedArray<ResourceLocation>,
    max_players: VarInt,
    view_distance: VarInt,
    simulation_distance: VarInt,
    reduced_debug_info: bool,
    enable_respawn_screen: bool,
    do_limited_crafting: bool,
    /// mapping defined in Registry Data packet
    dimension_type: VarInt,
    dimension_name: ResourceLocation,
    hashed_seed: i64,
    game_mode: GameMode,
    previous_game_mode: GameMode,
    is_debug: bool,
    is_flat: bool,
    death_location: Option<DeathLocation>,
    portal_cooldown: VarInt,
    sea_level: VarInt,
    enforces_secure_chat: bool,
}

#[derive(MinecraftPacket)]
#[packet(id = 0x46)]
pub struct SynchronizePlayerPositionPacket {
    teleport_id: VarInt,
    x: f64,
    y: f64,
    z: f64,
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
    yaw: f32,
    pitch: f32,
    flags: TeleportFlags,
}
