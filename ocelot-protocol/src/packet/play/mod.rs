use ocelot_macros::Packet;

use crate::codec::{MinecraftCodec, PrefixedArray, VarInt};
use crate::types::{Identifier, Position};

pub struct DeathLocation {
    dimension_name: Identifier,
    location: Position,
}

impl MinecraftCodec for DeathLocation {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.dimension_name.encode(writer)?;
        self.location.encode(writer)
    }

    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            dimension_name: Identifier::decode(reader)?,
            location: Position::decode(reader)?,
        })
    }
}

#[derive(Packet)]
#[packet(id = 0x30)]
pub struct ClientboundLoginPacket {
    entity_id: i32,
    hardcore: bool,
    dimensions: PrefixedArray<Identifier>,
    max_players: VarInt,
    view_distance: VarInt,
    simulation_distance: VarInt,
    reduced_debuf_info: bool,
    enable_respawn_screen: bool,
    do_limited_crafting: bool,
    /// mapping defined in Registry Data packet
    dimension_type: VarInt,
    dimension_name: Identifier,
    hashed_seed: i64,
    game_mode: u8,
    /// -1 is undefined, used for game mode switch (F3+N & F3+F4)
    previous_game_mode: i8,
    is_debug: bool,
    is_flat: bool,
    death_location: Option<DeathLocation>,
    portal_cooldown: VarInt,
    sea_level: VarInt,
    enforces_secure_chat: bool,
}
