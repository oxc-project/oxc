//! Derive for `UnstableAddress` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, Schema, StructOrEnum};

use super::{Derive, define_derive};

/// Derive for `UnstableAddress` trait.
pub struct DeriveUnstableAddress;

define_derive!(DeriveUnstableAddress);

impl Derive for DeriveUnstableAddress {
    fn trait_name(&self) -> &'static str {
        "UnstableAddress"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            use oxc_allocator::UnstableAddress;
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        if let StructOrEnum::Struct(struct_def) = type_def {
            let ty = struct_def.ty_anon(schema);
            quote! {
                impl UnstableAddress for #ty {}
            }
        } else {
            panic!(
                "`UnstableAddress` can only be implemented with `#[generate_derive]` on structs: `{}`",
                type_def.name()
            );
        }
    }
}
