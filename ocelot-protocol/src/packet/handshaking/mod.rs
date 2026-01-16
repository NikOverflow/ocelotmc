use std::fmt::{Display, Formatter};

use ocelot_macros::{MinecraftCodec, Packet};

use crate::codec::{BoundedString, VarInt};

#[derive(MinecraftCodec)]
#[codec(via = VarInt)]
pub enum Intent {
    STATUS = 1,
    LOGIN = 2,
    TRANSFER = 3,
}
impl Display for Intent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let name = match self {
            Self::STATUS => "Status",
            Self::LOGIN => "Login",
            Self::TRANSFER => "Transfer",
        };
        write!(f, "{}", name)
    }
}

#[derive(Packet)]
#[packet(id = 0x00)]
pub struct ServerboundHandshakePacket {
    protocol_version: VarInt,
    server_address: BoundedString<255>,
    server_port: u16,
    intent: Intent,
}
