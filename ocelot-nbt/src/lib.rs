use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

pub trait NbtBinaryCodec: Sized {
    fn encode_binary<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode_binary<R: Read>(reader: &mut R) -> io::Result<Self>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TagType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::End),
            1 => Some(Self::Byte),
            2 => Some(Self::Short),
            3 => Some(Self::Int),
            4 => Some(Self::Long),
            5 => Some(Self::Float),
            6 => Some(Self::Double),
            7 => Some(Self::ByteArray),
            8 => Some(Self::String),
            9 => Some(Self::List),
            10 => Some(Self::Compound),
            11 => Some(Self::IntArray),
            12 => Some(Self::LongArray),
            _ => None,
        }
    }

    pub fn as_id(&self) -> u8 {
        (*self) as u8
    }
}

impl NbtBinaryCodec for TagType {
    fn encode_binary<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.as_id().encode_binary(writer)
    }

    fn decode_binary<R: Read>(reader: &mut R) -> io::Result<Self> {
        match Self::from_id(u8::decode_binary(reader)?) {
            Some(res) => Ok(res),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Tag Type Id",
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Tag {
    // End does not exist in memory, so having it representable might increase bugs
    // End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(TagType, Vec<Tag>),
    Compound(HashMap<String, Tag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Debug, PartialEq)]
pub struct NamedTag(String, Tag);

impl Tag {
    pub fn tag_type(&self) -> TagType {
        match self {
            Tag::Byte(_) => TagType::Byte,
            Tag::Short(_) => TagType::Short,
            Tag::Int(_) => TagType::Int,
            Tag::Long(_) => TagType::Long,
            Tag::Float(_) => TagType::Float,
            Tag::Double(_) => TagType::Double,
            Tag::ByteArray(_) => TagType::ByteArray,
            Tag::String(_) => TagType::String,
            Tag::List(..) => TagType::List,
            Tag::Compound(_) => TagType::Compound,
            Tag::IntArray(_) => TagType::IntArray,
            Tag::LongArray(_) => TagType::LongArray,
        }
    }

    pub fn encode_string<W: Write>(string: &str, writer: &mut W) -> io::Result<()> {
        let data = string.as_bytes();
        writer.write_all(&(data.len() as u16).to_be_bytes())?;
        writer.write_all(data)
    }

    fn encode_binary<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Byte(data) => data.encode_binary(writer),
            Self::Short(data) => data.encode_binary(writer),
            Self::Int(data) => data.encode_binary(writer),
            Self::Long(data) => data.encode_binary(writer),
            Self::Float(data) => data.encode_binary(writer),
            Self::Double(data) => data.encode_binary(writer),
            Self::ByteArray(items) => items.encode_binary(writer),
            Self::String(string) => string.encode_binary(writer),
            Self::List(tag_type, nameless_tags) => {
                tag_type.as_id().encode_binary(writer)?;
                (nameless_tags.len() as i32).encode_binary(writer)?;
                nameless_tags
                    .iter()
                    .try_for_each(|tag| tag.encode_binary(writer))
            }
            Self::Compound(named_tags) => {
                named_tags.iter().try_for_each(|(name, tag)| {
                    name.encode_binary(writer)?;
                    tag.encode_binary(writer)
                })?;
                TagType::End.encode_binary(writer)
            }
            Self::IntArray(items) => items.encode_binary(writer),
            Self::LongArray(items) => items.encode_binary(writer),
        }
    }
    fn decode_binary<R: Read>(tag_type: TagType, reader: &mut R) -> io::Result<Self> {
        match tag_type {
            TagType::End => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Trying to deserialize tag of type End",
            )),
            TagType::Byte => Ok(Self::Byte(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::Short => Ok(Self::Short(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::Int => Ok(Self::Int(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::Long => Ok(Self::Long(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::Float => Ok(Self::Float(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::Double => Ok(Self::Double(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::ByteArray => Ok(Self::ByteArray(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::String => Ok(Self::String(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::List => {
                let tag_type = TagType::decode_binary(reader)?;
                let len = i32::decode_binary(reader)? as usize;
                let mut buffer = Vec::with_capacity(len);
                for _ in 0..len {
                    buffer.push(Self::decode_binary(tag_type, reader)?);
                }
                Ok(Self::List(tag_type, buffer))
            }
            TagType::Compound => {
                let mut buffer = HashMap::new();
                let mut tag_type = TagType::decode_binary(reader)?;
                while tag_type != TagType::End {
                    let name = String::decode_binary(reader)?;
                    let tag = Tag::decode_binary(tag_type, reader)?;
                    buffer.insert(name, tag);
                    tag_type = TagType::decode_binary(reader)?;
                }
                Ok(Self::Compound(buffer))
            }
            TagType::IntArray => Ok(Self::IntArray(NbtBinaryCodec::decode_binary(reader)?)),
            TagType::LongArray => Ok(Self::LongArray(NbtBinaryCodec::decode_binary(reader)?)),
        }
    }
}

impl NamedTag {
    fn encode_binary<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.1.tag_type().encode_binary(writer)?;
        self.0.encode_binary(writer)?;
        self.1.encode_binary(writer)
    }

    fn decode_binary<R: Read>(reader: &mut R) -> io::Result<Option<Self>> {
        let tag_type = TagType::decode_binary(reader)?;
        if tag_type == TagType::End {
            Ok(None)
        } else {
            Ok(Some(Self(
                String::decode_binary(reader)?,
                Tag::decode_binary(tag_type, reader)?,
            )))
        }
    }

    fn encode_binary_to_network<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.1.tag_type().encode_binary(writer)?;
        debug_assert_eq!(self.0, "");
        self.1.encode_binary(writer)
    }

    fn decode_binary_from_network<R: Read>(reader: &mut R) -> io::Result<Option<Self>> {
        let tag_type = TagType::decode_binary(reader)?;
        if tag_type == TagType::End {
            Ok(None)
        } else {
            Ok(Some(Self("".into(), Tag::decode_binary(tag_type, reader)?)))
        }
    }
}

macro_rules! int_macro {
    ($type_name:ty) => {
        impl NbtBinaryCodec for $type_name {
            fn encode_binary<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                writer.write_all(&self.to_be_bytes())
            }
            fn decode_binary<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let mut buffer = [0u8; size_of::<Self>()];
                reader.read_exact(&mut buffer)?;
                Ok(Self::from_be_bytes(buffer))
            }
        }
    };
}

int_macro!(i8);
int_macro!(u8);
int_macro!(i16);
int_macro!(u16);
int_macro!(i32);
int_macro!(i64);
int_macro!(f32);
int_macro!(f64);

impl NbtBinaryCodec for String {
    fn encode_binary<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let data = self.as_bytes();
        (data.len() as u16).encode_binary(writer)?;
        writer.write_all(data)
    }

    fn decode_binary<R: Read>(reader: &mut R) -> io::Result<Self> {
        let len = u16::decode_binary(reader)? as usize;
        let mut buffer = Vec::new();
        reader.take(len as u64).read_to_end(&mut buffer)?;
        if buffer.len() != len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not enough data for string",
            ));
        }
        Self::from_utf8(buffer).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

impl<T: NbtBinaryCodec> NbtBinaryCodec for Vec<T> {
    fn encode_binary<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        (self.len() as i32).encode_binary(writer)?;
        self.iter().try_for_each(|data| data.encode_binary(writer))
    }

    fn decode_binary<R: Read>(reader: &mut R) -> io::Result<Self> {
        let len = i32::decode_binary(reader)? as usize;
        let mut buffer = Self::with_capacity(len);
        for _ in 0..len {
            buffer.push(T::decode_binary(reader)?);
        }
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {

    use std::io::Cursor;

    use super::*;

    #[test]
    fn tag_fromid_toid() {
        for i in 0..=12 {
            assert_eq!(TagType::from_id(i).map(|tag| tag.as_id()), Some(i));
        }
    }

    #[test]
    fn hello_world_nbt() {
        let data =
            hex::decode("0a000b68656c6c6f20776f726c640800046e616d65000942616e616e72616d6100")
                .unwrap();
        let nbt = NamedTag::decode_binary(&mut Cursor::new(data))
            .unwrap()
            .unwrap();
        let expected = NamedTag(
            "hello world".into(),
            Tag::Compound(HashMap::from([(
                "name".into(),
                Tag::String("Bananrama".into()),
            )])),
        );
        assert_eq!(nbt, expected);
    }

    #[test]
    fn biggest_nbt() {
        let data = hex::decode("0a00054c6576656c0400086c6f6e67546573747fffffffffffffff02000973686f7274546573747fff08000a737472696e6754657374002948454c4c4f20574f524c4420544849532049532041205445535420535452494e4720c385c384c39621050009666c6f6174546573743eff1832030007696e74546573747fffffff0a00146e657374656420636f6d706f756e6420746573740a000368616d0800046e616d65000648616d70757305000576616c75653f400000000a00036567670800046e616d6500074567676265727405000576616c75653f000000000009000f6c6973745465737420286c6f6e67290400000005000000000000000b000000000000000c000000000000000d000000000000000e000000000000000f0900136c697374546573742028636f6d706f756e64290a000000020800046e616d65000f436f6d706f756e642074616720233004000a637265617465642d6f6e000001265237d58d000800046e616d65000f436f6d706f756e642074616720233104000a637265617465642d6f6e000001265237d58d0001000862797465546573747f07006562797465417272617954657374202874686520666972737420313030302076616c756573206f6620286e2a6e2a3235352b6e2a3729253130302c207374617274696e672077697468206e3d302028302c2036322c2033342c2031362c20382c202e2e2e2929000003e8003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a063006000a646f75626c65546573743fdf8f6bbbff6a5e00").unwrap();
        let mut nbt = NamedTag::decode_binary(&mut Cursor::new(data))
            .unwrap()
            .unwrap();

        let mut expected = NamedTag("Level".into(), Tag::Compound(HashMap::from([
	    ("nested compound test".into(), Tag::Compound(HashMap::from([
		("egg".into(), Tag::Compound(HashMap::from([
		    ("name".into(), Tag::String("Eggbert".into())),
		    ("value".into(), Tag::Float(0.5)),
		]))),
		("ham".into(), Tag::Compound(HashMap::from([
		    ("name".into(), Tag::String("Hampus".into())),
		    ("value".into(), Tag::Float(0.75)),
		]))),
	    ]))),
	    ("intTest".into(), Tag::Int(2147483647)),
	    ("byteTest".into(), Tag::Byte(127)),
	    //	    ("stringTest".into(), Tag::String("HELLO WORLD THIS IS A TEST STRING \xc5\xc4\xd6!".into())),
	    ("stringTest".into(), Tag::String("HELLO WORLD THIS IS A TEST STRING ÅÄÖ!".into())),
	    ("listTest (long)".into(), Tag::List(TagType::Long, vec![
		Tag::Long(11),
		Tag::Long(12),
		Tag::Long(13),
		Tag::Long(14),
		Tag::Long(15),
	    ])),
	    ("doubleTest".into(), Tag::Double(0.49312871321823148)),
	    ("floatTest".into(), Tag::Float(0.49823147058486938)),
	    ("longTest".into(), Tag::Long(9223372036854775807)),
	    ("listTest (compound)".into(), Tag::List(TagType::Compound, vec![
		Tag::Compound(HashMap::from([
		    ("created-on".into(), Tag::Long(1264099775885)),
		    ("name".into(), Tag::String("Compound tag #0".into())),
		])),
		Tag::Compound(HashMap::from([
		    ("created-on".into(), Tag::Long(1264099775885)),
		    ("name".into(), Tag::String("Compound tag #1".into())),
		])),
	    ])),
	    ("byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))".into(), Tag::ByteArray(create_byte_array())),
	    ("shortTest".into(), Tag::Short(32767)),
	])));
        compare_nbt(&mut nbt, &mut expected, &mut vec![]);
        assert_eq!(nbt, expected);
    }

    #[test]
    fn biggest_nbt_network() {
        let data = hex::decode("0a0400086c6f6e67546573747fffffffffffffff02000973686f7274546573747fff08000a737472696e6754657374002948454c4c4f20574f524c4420544849532049532041205445535420535452494e4720c385c384c39621050009666c6f6174546573743eff1832030007696e74546573747fffffff0a00146e657374656420636f6d706f756e6420746573740a000368616d0800046e616d65000648616d70757305000576616c75653f400000000a00036567670800046e616d6500074567676265727405000576616c75653f000000000009000f6c6973745465737420286c6f6e67290400000005000000000000000b000000000000000c000000000000000d000000000000000e000000000000000f0900136c697374546573742028636f6d706f756e64290a000000020800046e616d65000f436f6d706f756e642074616720233004000a637265617465642d6f6e000001265237d58d000800046e616d65000f436f6d706f756e642074616720233104000a637265617465642d6f6e000001265237d58d0001000862797465546573747f07006562797465417272617954657374202874686520666972737420313030302076616c756573206f6620286e2a6e2a3235352b6e2a3729253130302c207374617274696e672077697468206e3d302028302c2036322c2033342c2031362c20382c202e2e2e2929000003e8003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a0630003e2210080a162c4c12462004564e505c0e2e5828024a3830323e54103a0a482c1a12142036561c502a0e60585a02183862320c54423a3c485e1a44145236241c1e2a4060265a34180662000c2242083c165e4c44465204244e1e5c402e2628344a063006000a646f75626c65546573743fdf8f6bbbff6a5e00").unwrap();
        let mut nbt = NamedTag::decode_binary_from_network(&mut Cursor::new(data))
            .unwrap()
            .unwrap();

        let mut expected = NamedTag("".into(), Tag::Compound(HashMap::from([
	    ("nested compound test".into(), Tag::Compound(HashMap::from([
		("egg".into(), Tag::Compound(HashMap::from([
		    ("name".into(), Tag::String("Eggbert".into())),
		    ("value".into(), Tag::Float(0.5)),
		]))),
		("ham".into(), Tag::Compound(HashMap::from([
		    ("name".into(), Tag::String("Hampus".into())),
		    ("value".into(), Tag::Float(0.75)),
		]))),
	    ]))),
	    ("intTest".into(), Tag::Int(2147483647)),
	    ("byteTest".into(), Tag::Byte(127)),
	    //	    ("stringTest".into(), Tag::String("HELLO WORLD THIS IS A TEST STRING \xc5\xc4\xd6!".into())),
	    ("stringTest".into(), Tag::String("HELLO WORLD THIS IS A TEST STRING ÅÄÖ!".into())),
	    ("listTest (long)".into(), Tag::List(TagType::Long, vec![
		Tag::Long(11),
		Tag::Long(12),
		Tag::Long(13),
		Tag::Long(14),
		Tag::Long(15),
	    ])),
	    ("doubleTest".into(), Tag::Double(0.49312871321823148)),
	    ("floatTest".into(), Tag::Float(0.49823147058486938)),
	    ("longTest".into(), Tag::Long(9223372036854775807)),
	    ("listTest (compound)".into(), Tag::List(TagType::Compound, vec![
		Tag::Compound(HashMap::from([
		    ("created-on".into(), Tag::Long(1264099775885)),
		    ("name".into(), Tag::String("Compound tag #0".into())),
		])),
		Tag::Compound(HashMap::from([
		    ("created-on".into(), Tag::Long(1264099775885)),
		    ("name".into(), Tag::String("Compound tag #1".into())),
		])),
	    ])),
	    ("byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))".into(), Tag::ByteArray(create_byte_array())),
	    ("shortTest".into(), Tag::Short(32767)),
	])));
        compare_nbt(&mut nbt, &mut expected, &mut vec![]);
        assert_eq!(nbt, expected);
    }

    fn compare_nbt(nbt: &NamedTag, expected: &NamedTag, path: &mut Vec<String>) {
        assert_eq!(nbt.0, expected.0, "{:?}", path);
        path.push(nbt.0.clone());
        compare_tag(&nbt.1, &expected.1, path);
        path.pop();
    }

    fn compare_tag(nbt: &Tag, expected: &Tag, path: &mut Vec<String>) {
        match expected {
            Tag::Compound(expected_tags) => match nbt {
                Tag::Compound(nbt_tags) => {
                    assert_eq!(
                        nbt_tags.len(),
                        expected_tags.len(),
                        "Size differs in compound at {:?}",
                        path
                    );
                    for (expected_name, expected_element) in expected_tags {
                        if let Some(nbt_element) = nbt_tags.get(expected_name) {
                            path.push(expected_name.clone());
                            compare_tag(nbt_element, expected_element, path);
                            path.pop();
                        } else {
                            assert!(
                                false,
                                "Expected element with name {:?} at {:?}, other names: {:?}",
                                expected_name,
                                path,
                                nbt_tags.keys()
                            )
                        }
                    }
                }
                _ => assert!(false, "Expected a compound at {:?} but got {:?}", path, nbt),
            },
            Tag::List(expected_tag_type, expected_tags) => match nbt {
                Tag::List(nbt_tag_type, nbt_tags) => {
                    assert_eq!(nbt_tag_type, expected_tag_type, "{:?}", path);
                    assert_eq!(
                        nbt_tags.len(),
                        expected_tags.len(),
                        "Size of List at {:?} differs",
                        path
                    );
                    for i in 0..expected_tags.len() {
                        path.push(i.to_string());
                        compare_tag(&nbt_tags[i], &expected_tags[i], path);
                        path.pop();
                    }
                }
                _ => assert!(false, "Expected a list at {:?} but got {:?}", path, nbt),
            },
            _ => assert_eq!(nbt, expected, "{:?}", path),
        }
    }

    fn create_byte_array() -> Vec<i8> {
        let mut res = Vec::with_capacity(1000);
        for n in 0..1000 {
            res.push(((n * n * 255 + n * 7) % 100) as i8);
        }
        res
    }
}
