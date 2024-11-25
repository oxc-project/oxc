use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    schema::{EnumDef, Schema, ToType, TypeDef},
    util::TypeWrapper,
};

use super::{define_derive, Derive};

pub struct DeriveGetAddress;

define_derive!(DeriveGetAddress);

impl Derive for DeriveGetAddress {
    fn trait_name() -> &'static str {
        "GetAddress"
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_allocator::{Address, GetAddress};
        }
    }

    fn derive(&mut self, def: &TypeDef, _schema: &Schema) -> TokenStream {
        if let TypeDef::Enum(enum_def) = def {
            derive_enum(enum_def)
        } else {
            panic!("`GetAddress` can only be implemented with `#[generate_derive]` on enums");
        }
    }
}

fn derive_enum(def: &EnumDef) -> TokenStream {
    let target_type = def.to_type();

    let matches = def.all_variants().map(|variant| {
        assert!(
            variant.fields.len() == 1
                && variant.fields[0].typ.analysis().wrapper == TypeWrapper::Box,
            "`GetAddress` can only be derived on enums where all variants are boxed"
        );

        let ident = variant.ident();
        quote!(Self::#ident(it) => GetAddress::address(it))
    });

    quote! {
        impl<'a> GetAddress for #target_type {
            ///@ `#[inline]` because compiler should boil this down to a single assembly instruction
            #[inline]
            fn address(&self) -> Address {
                match self {
                    #(#matches),*
                }
            }
        }
    }
}
