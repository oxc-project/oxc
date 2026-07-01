//! Derive for `ReplaceWith` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, Schema};

use super::{Derive, StructOrEnum, define_derive};

/// Derive for `ReplaceWith` trait.
pub struct DeriveReplaceWith;

define_derive!(DeriveReplaceWith);

impl Derive for DeriveReplaceWith {
    fn trait_name(&self) -> &'static str {
        "ReplaceWith"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            ///@@line_break
            use oxc_allocator::ReplaceWith;
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let ty = type_def.ty_anon(schema);
        quote! {
            impl ReplaceWith for #ty {}
        }
    }
}
