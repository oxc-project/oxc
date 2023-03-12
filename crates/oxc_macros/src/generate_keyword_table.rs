mod generate_hash_table;
mod hash_keyword;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, LitStr, Token,
};

use self::generate_hash_table::{generate_hash_table, HashTableMeta};

pub struct TableEntryMeta {
    pub key: String,
    pub value: Expr,
}

impl Parse for TableEntryMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let s = input.parse::<LitStr>()?;
        let key = s.value();
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}

pub struct TableMeta(Vec<TableEntryMeta>);

impl Parse for TableMeta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed = Punctuated::<TableEntryMeta, Token![,]>::parse_terminated(input)?;
        let entries = parsed.into_iter().collect::<Vec<_>>();
        Ok(Self(entries))
    }
}

pub fn dump_hash_table(hash_table: HashTableMeta) -> TokenStream {
    let HashTableMeta { size, seed, values } = hash_table;
    quote! {
        const HASH_TABLE_SIZE: usize = #size;
        const HASH_TABLE_SEED: u32 = #seed;

        static KEYWORD_HASH_TABLE: [Option<(Kind, Atom)>; HASH_TABLE_SIZE] = [
          #(#values),*
        ];

        #[inline]
        pub fn table_match_keyword(s: &str) -> Option<(Kind, &'static Atom)> {
          let slice = s.as_bytes();
          let hash_code = cityhash_sys::city_hash_64_with_seed(slice, HASH_TABLE_SEED as u64) as u32;
          let idx = hash_code as usize % HASH_TABLE_SIZE;
          KEYWORD_HASH_TABLE[idx].as_ref().and_then(|(kind, atom)| {
            let mut result = Some((*kind, atom));
            if slice != atom.as_bytes() {
              result = None;
            }
            result
          })
        }
    }
}

pub fn generate_keyword_table(table: &TableMeta) -> TokenStream {
    let entries = table.0.as_slice();
    let hash_table = generate_hash_table(entries);
    dump_hash_table(hash_table)
}
