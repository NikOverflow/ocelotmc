use std::io::{self, Read, Write};

use ocelot_types::{BoundedString, CustomType, ResourceLocation, VarInt, VarLong};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

pub trait MinecraftCodec: Sized {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;
}

impl MinecraftCodec for bool {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[*self as u8])
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 1];
        reader.read_exact(&mut buffer)?;
        match buffer[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid boolean value!",
            )),
        }
    }
}

macro_rules! number_codec {
    ($type:ty) => {
        impl MinecraftCodec for $type {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                writer.write_all(&self.to_be_bytes())
            }
            fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let mut buffer = [0u8; size_of::<Self>()];
                reader.read_exact(&mut buffer)?;
                Ok(Self::from_be_bytes(buffer))
            }
        }
    };
}
number_codec!(i8);
number_codec!(u8);
number_codec!(u16);
number_codec!(i32);
number_codec!(i64);
number_codec!(f32);
number_codec!(f64);

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

impl MinecraftCodec for ResourceLocation {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        BoundedString::<32767>::new(self.to_string())?.encode(writer)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        BoundedString::<32767>::decode(reader)?
            .0
            .try_into()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

impl MinecraftCodec for VarInt {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_to(writer)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        VarInt::read_from(reader)
    }
}

impl MinecraftCodec for VarLong {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_to(writer)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        VarLong::read_from(reader)
    }
}

// Position

impl MinecraftCodec for Uuid {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.as_u128().to_be_bytes())
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = [0u8; 16];
        reader.read_exact(&mut buffer)?;
        Ok(Uuid::from_u128(u128::from_be_bytes(buffer)))
    }
}

impl<T: MinecraftCodec> MinecraftCodec for Option<T> {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Some(value) => {
                true.encode(writer)?;
                value.encode(writer)
            }
            None => false.encode(writer),
        }
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let exists = bool::decode(reader)?;
        if exists {
            let value = T::decode(reader)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

impl MinecraftCodec for Vec<u8> {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

pub struct PrefixedArray<T>(pub Vec<T>);
impl<T: MinecraftCodec> PrefixedArray<T> {
    fn new(array: Vec<T>) -> Self {
        Self(array)
    }
    fn decode_items<R: Read>(reader: &mut R, size: usize) -> io::Result<Self> {
        let mut result = Vec::with_capacity(size as usize);
        for _ in 0..size {
            result.push(T::decode(reader)?);
        }
        Ok(Self(result))
    }
}
impl<T: MinecraftCodec> MinecraftCodec for PrefixedArray<T> {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        VarInt(self.0.len() as i32).encode(writer)?;
        self.0.iter().try_for_each(|value| value.encode(writer))
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let size = VarInt::decode(reader)?.0;
        Self::decode_items(reader, size as usize)
    }
}

pub struct BoundedPrefixedArray<T, const MAX: u64>(pub PrefixedArray<T>);
impl<T: MinecraftCodec, const MAX: u64> BoundedPrefixedArray<T, MAX> {
    pub fn new(array: Vec<T>) -> Self {
        Self(PrefixedArray::new(array))
    }
}
impl<T: MinecraftCodec, const MAX: u64> MinecraftCodec for BoundedPrefixedArray<T, MAX> {
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.0.0.len() > MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Array is too long!",
            ));
        }
        self.0.encode(writer)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let size = VarInt::decode(reader)?.0;
        if size > MAX as i32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Array is too long!",
            ));
        }
        Ok(Self(PrefixedArray::<T>::decode_items(
            reader,
            size as usize,
        )?))
    }
}

pub struct Json<T>(pub T);
impl<T> MinecraftCodec for Json<T>
where
    T: Serialize + DeserializeOwned,
{
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let json_string = serde_json::to_string(&self.0)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let bounded_string = BoundedString::<32767>::new(json_string)?;
        bounded_string.encode(writer)
    }
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let bounded_string = BoundedString::<32767>::decode(reader)?;
        let res = serde_json::from_str(&bounded_string.0)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Json(res))
    }
}

#[macro_export]
macro_rules! bitfield {
    ($name:ident, $type:ty, {
        $($field:ident => $bit:expr),* $(,)?
    }) => {
        bitflags::bitflags! {
            pub struct $name: $type {
                $(
                    const $field = 1 << $bit;
                )*
            }
        }
        impl $crate::codec::MinecraftCodec for $name where $type: $crate::codec::MinecraftCodec {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                self.bits().encode(writer)
            }
            fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                Ok(Self::from_bits_truncate(<$type>::decode(reader)?))
            }
        }
    };
}
