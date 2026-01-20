use darling::{FromDeriveInput, FromField, FromVariant, ast};
use proc_macro::TokenStream;
use proc_macro_crate::FoundCrate;
use quote::{format_ident, quote};
use syn::{DeriveInput, Expr, Ident, Path, Type, parse_macro_input};

const PRIMITIVES: [&str; 16] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "char", "bool",
];

#[derive(FromDeriveInput)]
#[darling(attributes(codec), supports(enum_unit, struct_named))]
struct CodecReceiver {
    ident: Ident,
    #[darling(rename = "via")]
    codec: Option<Path>,
    data: ast::Data<CodecVariantReceiver, CodecFieldReceiver>,
}

#[derive(FromField)]
struct CodecFieldReceiver {
    ident: Option<Ident>,
    ty: Type,
}

#[derive(FromVariant)]
struct CodecVariantReceiver {
    ident: Ident,
    discriminant: Option<Expr>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(packet), supports(struct_named))]
struct PacketReceiver {
    ident: Ident,
    id: i32,
    data: ast::Data<(), PacketFieldReceiver>,
}

#[derive(FromField)]
struct PacketFieldReceiver {
    ident: Option<Ident>,
    ty: Type,
}

fn get_root_path() -> proc_macro2::TokenStream {
    match proc_macro_crate::crate_name("ocelot-protocol")
        .expect("ocelot-protocol crate is not present in Cargo.toml!")
    {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let identifier = format_ident!("{}", name);
            quote!(::#identifier)
        }
    }
}

#[proc_macro_derive(MinecraftCodec, attributes(codec))]
pub fn codec_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = match CodecReceiver::from_derive_input(&input) {
        Ok(res) => res,
        Err(err) => return err.write_errors().into(),
    };
    let root = get_root_path();

    let name = &receiver.ident;

    let expanded = match receiver.data {
        ast::Data::Struct(fields) => {
            let field_names: Vec<_> = fields.iter().map(|field| &field.ident).collect();
            quote! {
                impl #root::codec::MinecraftCodec for #name {
                    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                        #( #root::codec::MinecraftCodec::encode(&self.#field_names, writer)?; )*
                        Ok(())
                    }
                    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                        Ok(Self {
                            #( #field_names: #root::codec::MinecraftCodec::decode(reader)?, )*
                        })
                    }
                }
            }
        }
        ast::Data::Enum(variants) => {
            let codec_path = &receiver.codec;
            let codec_str = quote!(#codec_path).to_string();
            let variant_names: Vec<_> = variants.iter().map(|variant| &variant.ident).collect();
            let encode_patterns: Vec<_> = variants
                .iter()
                .map(|variant| {
                    let ident = &variant.ident;
                    let discriminant = variant
                        .discriminant
                        .as_ref()
                        .expect("Explicit discriminant required!");
                    let value = if PRIMITIVES.contains(&codec_str.as_str()) {
                        quote! { (#discriminant as #codec_path) }
                    } else {
                        quote! { #codec_path(#discriminant) }
                    };
                    quote! {
                        Self::#ident => <#codec_path as #root::codec::MinecraftCodec>::encode(&#value, writer)?,
                    }
                })
                .collect();
            let decode_patterns: Vec<_> = variants
                .iter()
                .map(|variant| {
                    let ident = &variant.ident;
                    let discriminant = &variant
                        .discriminant
                        .as_ref()
                        .expect("Explicit discriminant required!");
                    let pattern = if PRIMITIVES.contains(&codec_str.as_str()) {
                        quote! { #discriminant }
                    } else {
                        quote! { #codec_path(#discriminant) }
                    };
                    quote! { #pattern => Ok(Self::#ident), }
                })
                .collect();
            let display_names: Vec<String> = variants
                .iter()
                .map(|variant| {
                    let ident = variant.ident.to_string();
                    let mut lowercase = ident.to_lowercase();
                    if let Some(first) = lowercase.get_mut(0..1) {
                        first.make_ascii_uppercase();
                    }
                    lowercase
                })
                .collect();
            quote! {
                impl #root::codec::MinecraftCodec for #name {
                    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                        match self {
                            #(#encode_patterns)*
                        }
                        Ok(())
                    }
                    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                        let id: #codec_path = <#codec_path as #root::codec::MinecraftCodec>::decode(reader)?;
                        match id {
                            #(#decode_patterns)*
                            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown id for enum {}", stringify!(#name)))),
                        }
                    }
                }
                impl std::fmt::Display for #name {
                    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #( Self::#variant_names => write!(formatter, "{}", #display_names), )*
                        }
                    }
                }
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Packet, attributes(packet))]
pub fn packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = match PacketReceiver::from_derive_input(&input) {
        Ok(res) => res,
        Err(err) => return err.write_errors().into(),
    };
    let root = get_root_path();

    let name = &receiver.ident;
    let packet_id = receiver.id;
    let fields = receiver.data.take_struct().unwrap().fields; // This can't fail at the moment.
    let field_names: Vec<&Ident> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let getter_names: Vec<Ident> = field_names
        .iter()
        .map(|ident| format_ident!("get_{}", ident))
        .collect();
    let field_types: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();
    let expanded = quote! {
        impl #name {
            pub const ID: i32 = #packet_id;
            pub fn new(#( #field_names: #field_types, )*) -> Self {
                Self {
                    #( #field_names, )*
                }
            }
            #(
                pub fn #getter_names(&self) -> &#field_types {
                    &self.#field_names
                }
            )*
        }
        impl #root::packet::MinecraftPacket for #name {
            fn get_id(&self) -> i32 {
                #packet_id
            }
            fn serialize(&self) -> std::io::Result<Vec<u8>> {
                let mut writer = #root::buffer::PacketWriter::new();
                #root::codec::MinecraftCodec::encode(&#root::codec::VarInt(#packet_id), &mut writer)?;
                #( #root::codec::MinecraftCodec::encode(&self.#field_names, &mut writer)?; )*
                Ok(writer.build())
            }
            fn deserialize(buffer: &mut #root::buffer::PacketBuffer) -> std::io::Result<Self> {
                Ok(Self {
                    #( #field_names: <#field_types as #root::codec::MinecraftCodec>::decode(buffer)?, )*
                })
            }
        }
    };
    TokenStream::from(expanded)
}
