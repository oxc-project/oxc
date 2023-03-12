use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use super::TableEntryMeta;
use crate::generate_keyword_table::hash_keyword::{extract_first_and_last_two_bytes, hash_u32};

pub struct HashTable<'a>(Vec<Option<(&'a str, Expr)>>);

impl<'a> HashTable<'a> {
    pub fn empty(size: usize) -> Self {
        Self(vec![None; size])
    }

    pub fn try_generate(
        &mut self,
        keys: &[(&'a str, Expr)],
        seed: u32,
        hasher: impl Fn(&[u8], u32) -> u32,
    ) -> bool {
        self.0.fill(None);
        for s in keys {
            let slice = s.0.as_bytes();
            let hash_code = hasher(slice, seed);
            let idx = hash_code as usize % self.0.len();
            let slot = &mut self.0[idx];
            if slot.is_some() {
                return false;
            }
            *slot = Some((s.0, s.1.clone()));
        }
        true
    }

    pub fn into_values(self) -> Vec<TokenStream> {
        self.0
            .into_iter()
            .map(|entry| {
                if let Some((_, exp)) = entry {
                    quote! { Some(#exp) }
                } else {
                    quote! {None}
                }
            })
            .collect()
    }
}

pub struct HashTableMeta {
    pub size: usize,
    pub seed: u32,
    pub values: Vec<TokenStream>,
}

#[allow(clippy::all)]
pub fn generate_hash_table(keys: &[TableEntryMeta]) -> HashTableMeta {
    const ATTEMPTS: u32 = 50_000;
    const MAX_SIZE: usize = 1024;

    let keys: Vec<_> = keys.iter().map(|meta| (meta.key.as_str(), meta.value.clone())).collect();
    let mut seed = 0x0070_ABCA;
    let mut size = keys.len().next_power_of_two();
    while size <= MAX_SIZE {
        let mut table = HashTable::empty(size);
        for _ in 0..ATTEMPTS {
            if table.try_generate(&keys, seed, |slice, seed| unsafe {
                let selection = extract_first_and_last_two_bytes(slice);
                hash_u32(selection, seed)
            }) {
                return HashTableMeta { size, seed, values: table.into_values() };
            }
            seed += 1;
        }
        size *= 2;
    }

    panic!("Failed to generate Hash Table.")
}
