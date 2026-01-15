use std::io::{self, Read, Write};

pub trait MinecraftCodec: Sized {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;
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

#[cfg(test)]
mod tests {
    use crate::codec::{MinecraftCodec, VarInt};

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
}
