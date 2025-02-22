//! Derive for `GetAddress` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, EnumDef, Schema};

use super::{Derive, StructOrEnum, define_derive};

/// Derive for `GetAddress` trait.
pub struct DeriveGetAddress;

define_derive!(DeriveGetAddress);

impl Derive for DeriveGetAddress {
    fn trait_name(&self) -> &'static str {
        "GetAddress"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![expect(clippy::match_same_arms)]

            ///@@line_break
            use oxc_allocator::{Address, GetAddress};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        if let StructOrEnum::Enum(enum_def) = type_def {
            derive_enum(enum_def, schema)
        } else {
            panic!(
                "`GetAddress` can only be implemented with `#[generate_derive]` on enums: `{}`",
                type_def.name()
            );
        }
    }
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let ty = enum_def.ty_anon(schema);

    let matches = enum_def.all_variants(schema).map(|variant| {
        let variant_type = variant.field_type(schema).unwrap();
        assert!(
            variant_type.is_box(),
            "`GetAddress` can only be derived on enums where all variants are boxed: `{}::{}`",
            enum_def.name(),
            variant.name(),
        );

        let ident = variant.ident();
        quote!( Self::#ident(it) => GetAddress::address(it) )
    });

    quote! {
        impl GetAddress for #ty {
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
