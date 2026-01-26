pub mod text;

use std::{
    fmt::{self, Display, Formatter},
    io::{self, Read, Write},
    sync::OnceLock,
};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, de};
use thiserror::Error;

pub trait CustomType: Sized {
    fn read_from<R: Read>(reader: &mut R) -> io::Result<Self>;
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

pub const MAX_STRING_LENGTH: u64 = 32767;
pub const SEGMENT_BITS: u8 = 0x7F;
pub const CONTINUE_BITS: u8 = 0x80;

pub struct BoundedString<const MAX: u64>(pub String);
impl<const MAX: u64> BoundedString<MAX> {
    pub fn new(string: impl Into<String>) -> io::Result<Self> {
        let s: String = string.into();
        let utf16_len: u64 = s.encode_utf16().count() as u64;
        if utf16_len > MAX || utf16_len > MAX_STRING_LENGTH || s.len() as u64 > (MAX * 3) + 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "String too long!",
            ));
        }
        Ok(Self(s))
    }
}

#[derive(Serialize, Deserialize)]
pub struct VarInt(pub i32);
impl CustomType for VarInt {
    fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut byte = [0u8; 1];
        loop {
            reader.read_exact(&mut byte)?;
            let current_byte = byte[0];
            value |= ((current_byte & SEGMENT_BITS as u8) as i32) << position;
            if (current_byte & CONTINUE_BITS as u8) == 0 {
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
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut value = self.0 as u32;
        loop {
            if (value & !(SEGMENT_BITS as u32)) == 0 {
                writer.write_all(&[value as u8])?;
                return Ok(());
            }
            writer.write_all(&[((value & SEGMENT_BITS as u32) as u8) | CONTINUE_BITS as u8])?;
            value >>= 7;
        }
    }
}

pub struct VarLong(pub i64);
impl CustomType for VarLong {
    fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = 0;
        let mut position = 0;
        let mut byte = [0u8; 1];
        loop {
            reader.read_exact(&mut byte)?;
            let current_byte = byte[0];
            value |= ((current_byte & SEGMENT_BITS as u8) as i64) << position;
            if (current_byte & CONTINUE_BITS as u8) == 0 {
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
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut value = self.0 as u64;
        loop {
            if (value & !(SEGMENT_BITS as u64)) == 0 {
                writer.write_all(&[value as u8])?;
                return Ok(());
            }
            writer.write_all(&[((value & SEGMENT_BITS as u64) as u8) | CONTINUE_BITS as u8])?;
            value >>= 7;
        }
    }
}

#[derive(Error, Debug)]
pub enum ResourceLocationError {
    #[error("The resource location is invalid!")]
    Invalid { namespace: String, path: String },
}

pub struct ResourceLocation {
    namespace: String,
    path: String,
}
impl ResourceLocation {
    pub fn from(
        namespace: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self, ResourceLocationError> {
        let namespace = namespace.into();
        let path = path.into();
        static REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = REGEX.get_or_init(|| Regex::new("^([a-z0-9._-]+:)?([a-z0-9/._-]+)$").unwrap());
        if regex.captures(&format!("{}:{}", namespace, path)).is_none() {
            return Err(ResourceLocationError::Invalid { namespace, path });
        }
        Ok(Self {
            namespace: namespace,
            path: path.into(),
        })
    }
    pub fn from_vanilla(path: impl Into<String>) -> Result<Self, ResourceLocationError> {
        Self::from("minecraft", path.into())
    }
}

impl Display for ResourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}
impl TryFrom<String> for ResourceLocation {
    type Error = ResourceLocationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.splitn(2, ':').collect();
        if parts.len() == 2 {
            Self::from(parts[0], parts[1])
        } else {
            Self::from_vanilla(parts[0])
        }
    }
}
impl Serialize for ResourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .try_into()
            .map_err(|error: ResourceLocationError| de::Error::custom(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{CustomType, VarInt, VarLong};

    use std::io::Cursor;

    #[test]
    fn read_varint() {
        let read_check = |bytes: &[u8], expected: VarInt| {
            let mut cursor = Cursor::new(bytes);
            let varint = VarInt::read_from(&mut cursor).unwrap();
            assert_eq!(varint.0, expected.0);
        };
        read_check(&vec![0x00], VarInt(0));
        read_check(&vec![0x01], VarInt(1));
        read_check(&vec![0x02], VarInt(2));
        read_check(&vec![0x7f], VarInt(127));
        read_check(&vec![0x80, 0x01], VarInt(128));
        read_check(&vec![0xff, 0x01], VarInt(255));
        read_check(&vec![0xdd, 0xc7, 0x01], VarInt(25565));
        read_check(&vec![0xff, 0xff, 0x7f], VarInt(2097151));
        read_check(&vec![0xff, 0xff, 0xff, 0xff, 0x07], VarInt(2147483647));
        read_check(&vec![0xff, 0xff, 0xff, 0xff, 0x0f], VarInt(-1));
        read_check(&vec![0x80, 0x80, 0x80, 0x80, 0x08], VarInt(-2147483648));
    }
    #[test]
    fn write_varint() {
        let write_check = |varint: VarInt, expected: &[u8]| {
            let mut buffer = Vec::new();
            varint.write_to(&mut buffer).unwrap();
            assert_eq!(buffer, expected);
        };
        write_check(VarInt(0), &vec![0x00]);
        write_check(VarInt(1), &vec![0x01]);
        write_check(VarInt(2), &vec![0x02]);
        write_check(VarInt(127), &vec![0x7f]);
        write_check(VarInt(128), &vec![0x80, 0x01]);
        write_check(VarInt(255), &vec![0xff, 0x01]);
        write_check(VarInt(25565), &vec![0xdd, 0xc7, 0x01]);
        write_check(VarInt(2097151), &vec![0xff, 0xff, 0x7f]);
        write_check(VarInt(2147483647), &vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        write_check(VarInt(-1), &vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        write_check(VarInt(-2147483648), &vec![0x80, 0x80, 0x80, 0x80, 0x08]);
    }
    #[test]
    fn read_varlong() {
        let read_check = |bytes: &[u8], expected: VarLong| {
            let mut cursor = Cursor::new(bytes);
            let varint = VarLong::read_from(&mut cursor).unwrap();
            assert_eq!(varint.0, expected.0);
        };
        read_check(&vec![0x00], VarLong(0));
        read_check(&vec![0x01], VarLong(1));
        read_check(&vec![0x02], VarLong(2));
        read_check(&vec![0x7f], VarLong(127));
        read_check(&vec![0x80, 0x01], VarLong(128));
        read_check(&vec![0xff, 0x01], VarLong(255));
        read_check(&vec![0xff, 0xff, 0xff, 0xff, 0x07], VarLong(2147483647));
        read_check(
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            VarLong(9223372036854775807),
        );
        read_check(
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            VarLong(-1),
        );
        read_check(
            &vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
            VarLong(-2147483648),
        );
        read_check(
            &vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            VarLong(-9223372036854775808),
        );
    }
    #[test]
    fn write_varlong() {
        let write_check = |varint: VarLong, expected: &[u8]| {
            let mut buffer = Vec::new();
            varint.write_to(&mut buffer).unwrap();
            assert_eq!(buffer, expected);
        };
        write_check(VarLong(0), &vec![0x00]);
        write_check(VarLong(1), &vec![0x01]);
        write_check(VarLong(2), &vec![0x02]);
        write_check(VarLong(127), &vec![0x7f]);
        write_check(VarLong(128), &vec![0x80, 0x01]);
        write_check(VarLong(255), &vec![0xff, 0x01]);
        write_check(VarLong(2147483647), &vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        write_check(
            VarLong(9223372036854775807),
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        );
        write_check(
            VarLong(-1),
            &vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        write_check(
            VarLong(-2147483648),
            &vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
        );
        write_check(
            VarLong(-9223372036854775808),
            &vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        );
    }
}
