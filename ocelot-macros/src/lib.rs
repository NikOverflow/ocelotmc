use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Ident, LitInt, parse_macro_input};

#[proc_macro_derive(Packet, attributes(packet))]
pub fn packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let root = match crate_name("ocelot-protocol")
        .expect("ocelot-protocol crate is not present in Cargo.toml!")
    {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let identifier = format_ident!("{}", name);
            quote!(::#identifier)
        }
    };

    let fields = if let Data::Struct(r#struct) = &input.data {
        &r#struct.fields
    } else {
        panic!("Packet derive is only allowed for structs!")
    };

    let mut id: Option<i32> = None;

    for attribute in &input.attrs {
        if attribute.path().is_ident("packet") {
            id = Some(
                attribute
                    .parse_args::<LitInt>()
                    .expect("Packet id is invalid!")
                    .base10_parse::<i32>()
                    .expect("Packet id is invalid!"),
            );
        }
    }
    let packet_id = id.expect("Packet id is required!");

    let field_names: Vec<&Ident> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let getter_names: Vec<Ident> = field_names
        .iter()
        .map(|ident| format_ident!("get_{}", ident))
        .collect();
    let field_types: Vec<&syn::Type> = fields.iter().map(|f| &f.ty).collect();
    let expanded = quote! {
        impl #name {
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
                    #( #field_names: #root::codec::MinecraftCodec::decode(buffer)?, )*
                })
            }
        }
    };
    TokenStream::from(expanded)
}
