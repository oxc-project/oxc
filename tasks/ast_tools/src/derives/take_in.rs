//! Derive for `TakeIn` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, Schema};

use super::{Derive, StructOrEnum, define_derive};

/// Derive for `TakeIn` trait.
pub struct DeriveTakeIn;

define_derive!(DeriveTakeIn);

impl Derive for DeriveTakeIn {
    fn trait_name(&self) -> &'static str {
        "TakeIn"
    }

    fn trait_has_lifetime(&self) -> bool {
        true
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![expect(clippy::elidable_lifetime_names)]

            ///@@line_break
            use oxc_allocator::TakeIn;
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let ty = type_def.ty(schema);
        quote! {
            impl<'a> TakeIn<'a> for #ty {}
        }
    }
}
