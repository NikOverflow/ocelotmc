use crate::codec::{BoundedString, MinecraftCodec};

pub struct Identifier {
    namespaced_value: BoundedString<32767>,
}

impl Identifier {
    pub const namespace_regex: &str = "[a-z0-9.-_]";
    pub const value_regex: &str = "[a-z0-9.-_/]";
    pub const total_regex: &str = "[a-z0-9.-_]:[a-z0-9.-_/]";

    pub fn from_string(namespaced_value: BoundedString<32767>) -> Self {
        // TODO: validate that the parameter matches the regex.
        Self { namespaced_value }
    }
}

impl MinecraftCodec for Identifier {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.namespaced_value.encode(writer)
    }

    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let namespaced_value = BoundedString::decode(reader)?;
        Ok(Self::from_string(namespaced_value))
    }
}

pub struct Position {
    x: i32,
    y: i16,
    z: i32,
}

impl MinecraftCodec for Position {
    // see https://minecraft.wiki/w/Java_Edition_protocol/Packets#Position for specs

    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let val = ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.z as i64 & 0x3FFFFFF) << 12)
            | (self.y as i64 & 0xFFF);
        val.encode(writer)
    }

    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let val = i64::decode(reader)?;
        Ok(Self {
            x: (val >> 38) as i32,
            y: (val << 52 >> 52) as i16,
            z: (val << 26 >> 38) as i32,
        })
    }
}
