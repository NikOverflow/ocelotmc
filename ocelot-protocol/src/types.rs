use crate::codec::MinecraftCodec;

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
