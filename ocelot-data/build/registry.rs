use std::collections::HashMap;

use fastnbt::{SerOpts, to_bytes_with_opts};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use serde_json::Value;

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/synced_registries.json");

    let json_str = std::fs::read_to_string("../assets/synced_registries.json")
        .expect("Failed to read synced_registries.json");
    let data: HashMap<String, HashMap<String, Value>> =
        serde_json::from_str(&json_str).expect("Failed to parse synced_registries.json");
    let ensure_namespace = |s: &str| {
        if s.contains(':') {
            s.to_string()
        } else {
            format!("minecraft:{}", s)
        }
    };
    let registry = data.iter().map(|(reg_name, entries)| {
        let reg_name = ensure_namespace(reg_name);
        let entry_tokens = entries.iter().map(|(entry_name, entry_data)| {
            let entry_name = ensure_namespace(entry_name);
            let nbt_bytes = to_bytes_with_opts(&entry_data, SerOpts::network_nbt()).unwrap();
            let nbt_literal = Literal::byte_string(&nbt_bytes);
            quote! {
                StaticRegistryEntry {
                    name: #entry_name,
                    nbt_bytes: #nbt_literal,
                }
            }
        });

        quote! {
            StaticRegistry {
                registry_id: #reg_name,
                entries: &[#(#entry_tokens),*],
            }
        }
    });
    let expanded = quote! {
        pub struct StaticRegistryEntry {
            pub name: &'static str,
            pub nbt_bytes: &'static [u8],
        }

        pub struct StaticRegistry {
            pub registry_id: &'static str,
            pub entries: &'static [StaticRegistryEntry],
        }

        pub static SYNCED_REGISTRIES: &[StaticRegistry] = &[
            #(#registry),*
        ];

        pub struct RegistryEntryData {
            pub entry_id: String,
            pub data: Option<Vec<u8>>,
        }

        pub struct Registry {
            pub registry_id: String,
            pub registry_entries: Vec<RegistryEntryData>,
        }
    };
    TokenStream::from(expanded)
}
