pub mod configuration;
pub mod handshaking;
pub mod login;
pub mod play;
pub mod status;

use crate::buffer::PacketBuffer;

use std::io;

pub trait MinecraftPacket: Sized {
    fn get_id(&self) -> i32;
    fn serialize(&self) -> io::Result<Vec<u8>>;
    fn deserialize(buffer: &mut PacketBuffer) -> io::Result<Self>;
}
