use std::io::{self, Read, Write};

pub trait MinecraftCodec: Sized {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;
}

pub struct BoundedString<const MAX: u64>(String);
impl<const MAX: u64> BoundedString<MAX> {
    const MAX_LENGTH: u64 = 32767;

    pub fn new(string: impl Into<String>) -> io::Result<Self> {
        let s: String = string.into();
        let utf16_len: u64 = s.encode_utf16().count() as u64;
        if utf16_len > MAX || utf16_len > Self::MAX_LENGTH || s.len() as u64 > (MAX * 3) + 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long!",
            ));
        }
        Ok(Self(s))
    }
}
impl<const MAX: u64> MinecraftCodec for BoundedString<MAX> {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let bytes = self.0.as_bytes();
        VarInt(bytes.len() as i32).encode(writer)?;
        writer.write_all(bytes)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = vec![0u8; VarInt::decode(reader)?.0 as usize];
        reader.read_exact(&mut buffer)?;
        Self::new(
            String::from_utf8(buffer)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
        )
    }
}

pub struct VarInt(pub i32);
impl VarInt {
    const SEGMENT_BITS: u32 = 0x7F;
    const CONTINUE_BITS: u32 = 0x80;
}
impl MinecraftCodec for VarInt {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut value = self.0 as u32;
        loop {
            if (value & !Self::SEGMENT_BITS) == 0 {
                writer.write_all(&[value as u8])?;
                return Ok(());
            }
            writer
                .write_all(&[((value & Self::SEGMENT_BITS) as u8) | Self::CONTINUE_BITS as u8])?;
            value >>= 7;
        }
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut byte = [0u8; 1];
        loop {
            reader.read_exact(&mut byte)?;
            let current_byte = byte[0];
            value |= ((current_byte & Self::SEGMENT_BITS as u8) as i32) << position;
            if (current_byte & Self::CONTINUE_BITS as u8) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt is too big!",
                ));
            }
        }
        Ok(VarInt(value))
    }
}

pub struct VarLong(pub i64);
impl VarLong {
    const SEGMENT_BITS: u64 = 0x7F;
    const CONTINUE_BITS: u64 = 0x80;
}
impl MinecraftCodec for VarLong {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut value = self.0 as u64;
        loop {
            if (value & !Self::SEGMENT_BITS) == 0 {
                writer.write_all(&[value as u8])?;
                return Ok(());
            }
            writer
                .write_all(&[((value & Self::SEGMENT_BITS) as u8) | Self::CONTINUE_BITS as u8])?;
            value >>= 7;
        }
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut byte = [0u8; 1];
        loop {
            reader.read_exact(&mut byte)?;
            let current_byte = byte[0];
            value |= ((current_byte & Self::SEGMENT_BITS as u8) as i64) << position;
            if (current_byte & Self::CONTINUE_BITS as u8) == 0 {
                break;
            }
            position += 7;
            if position >= 64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt is too big!",
                ));
            }
        }
        Ok(VarLong(value))
    }
}

impl MinecraftCodec for bool {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[if *self { 1 } else { 0 }])
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        match byte[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid boolean value: '{}'!", byte[0]),
            )),
        }
    }
}
impl MinecraftCodec for i8 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        Ok(i8::from_be_bytes(buffer))
    }
}
impl MinecraftCodec for u8 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        Ok(u8::from_be_bytes(buffer))
    }
}
impl MinecraftCodec for i16 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 2];
        reader.read_exact(&mut buffer)?;
        Ok(i16::from_be_bytes(buffer))
    }
}
impl MinecraftCodec for u16 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 2];
        reader.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }
}
impl MinecraftCodec for i32 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        Ok(i32::from_be_bytes(buffer))
    }
}
impl MinecraftCodec for i64 {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;
        Ok(i64::from_be_bytes(buffer))
    }
}

#[cfg(test)]
mod tests {
    use crate::codec::{MinecraftCodec, VarInt, VarLong};

    use std::io::Cursor;

    #[test]
    fn encode_varint() {
        let encode_check = |varint: VarInt, expected: &[u8]| {
            let mut buffer = Vec::new();
            varint.encode(&mut buffer).unwrap();
            assert_eq!(buffer, expected);
        };
        encode_check(VarInt(0), &vec![0x00]);
        encode_check(VarInt(1), &vec![0x01]);
        encode_check(VarInt(2), &vec![0x02]);
        encode_check(VarInt(127), &vec![0x7f]);
        encode_check(VarInt(128), &vec![0x80, 0x01]);
        encode_check(VarInt(255), &vec![0xff, 0x01]);
        encode_check(VarInt(25565), &vec![0xdd, 0xc7, 0x01]);
        encode_check(VarInt(2097151), &vec![0xff, 0xff, 0x7f]);
        encode_check(VarInt(2147483647), &vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        encode_check(VarInt(-1), &vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        encode_check(VarInt(-2147483648), &vec![0x80, 0x80, 0x80, 0x80, 0x08]);
    }
    #[test]
    fn decode_varint() {
        let decode_check = |bytes: &[u8], expected: VarInt| {
            let mut cursor = Cursor::new(bytes);
            let varint = VarInt::decode(&mut cursor).unwrap();
            assert_eq!(varint.0, expected.0);
        };
        decode_check(&vec![0x00], VarInt(0));
        decode_check(&vec![0x01], VarInt(1));
        decode_check(&vec![0x02], VarInt(2));
        decode_check(&vec![0x7f], VarInt(127));
        decode_check(&vec![0x80, 0x01], VarInt(128));
        decode_check(&vec![0xff, 0x01], VarInt(255));
        decode_check(&vec![0xdd, 0xc7, 0x01], VarInt(25565));
        decode_check(&vec![0xff, 0xff, 0x7f], VarInt(2097151));
        decode_check(&vec![0xff, 0xff, 0xff, 0xff, 0x07], VarInt(2147483647));
        decode_check(&vec![0xff, 0xff, 0xff, 0xff, 0x0f], VarInt(-1));
        decode_check(&vec![0x80, 0x80, 0x80, 0x80, 0x08], VarInt(-2147483648));
    }
    #[test]
    fn encode_varlong() {
        let encode_check = |varint: VarLong, expected: &[u8]| {
            let mut buffer = Vec::new();
            varint.encode(&mut buffer).unwrap();
            assert_eq!(buffer, expected);
        };
        encode_check(VarLong(0), &vec![0x00]);
        encode_check(VarLong(1), &vec![0x01]);
        encode_check(VarLong(2), &vec![0x02]);
        encode_check(VarLong(127), &vec![0x7f]);
        encode_check(VarLong(128), &vec![0x80, 0x01]);
        encode_check(VarLong(255), &vec![0xff, 0x01]);
        encode_check(VarLong(2147483647), &vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        encode_check(
            VarLong(9223372036854775807),
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        );
        encode_check(
            VarLong(-1),
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        encode_check(
            VarLong(-2147483648),
            &vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        encode_check(
            VarLong(-9223372036854775808),
            &vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
    }
    #[test]
    fn decode_varlong() {
        let decode_check = |bytes: &[u8], expected: VarLong| {
            let mut cursor = Cursor::new(bytes);
            let varint = VarLong::decode(&mut cursor).unwrap();
            assert_eq!(varint.0, expected.0);
        };
        decode_check(&vec![0x00], VarLong(0));
        decode_check(&vec![0x01], VarLong(1));
        decode_check(&vec![0x02], VarLong(2));
        decode_check(&vec![0x7f], VarLong(127));
        decode_check(&vec![0x80, 0x01], VarLong(128));
        decode_check(&vec![0xff, 0x01], VarLong(255));
        decode_check(&vec![0xff, 0xff, 0xff, 0xff, 0x07], VarLong(2147483647));
        decode_check(
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            VarLong(9223372036854775807),
        );
        decode_check(
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            VarLong(-1),
        );
        decode_check(
            &vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
            VarLong(-2147483648),
        );
        decode_check(
            &vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            VarLong(-9223372036854775808),
        );
    }
}
