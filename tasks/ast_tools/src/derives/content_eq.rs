use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    schema::{EnumDef, GetGenerics, Schema, StructDef, ToType, TypeDef},
    util::ToIdent,
};

use super::{define_derive, Derive};

const IGNORE_FIELD_TYPES: [/* type name */ &str; 4] = [
    "Span",
    "ScopeId",
    "SymbolId",
    "ReferenceId",
];

pub struct DeriveContentEq;

define_derive!(DeriveContentEq);

impl Derive for DeriveContentEq {
    fn trait_name() -> &'static str {
        "ContentEq"
    }

    fn prelude() -> TokenStream {
        quote! {
            // NOTE: writing long match expressions formats better than using `matches` macro.
            #![allow(clippy::match_like_matches_macro)]

            ///@@line_break
            use oxc_span::cmp::ContentEq;
        }
    }

    fn derive(&mut self, def: &TypeDef, _: &Schema) -> TokenStream {
        let (other, body) = match &def {
            TypeDef::Enum(it) => derive_enum(it),
            TypeDef::Struct(it) => derive_struct(it),
        };

        impl_content_eq(def, other, &body)
    }
}

fn derive_enum(def: &EnumDef) -> (&str, TokenStream) {
    let body = if def.is_unit() {
        // we assume unit enums implement `PartialEq`
        quote!(self == other)
    } else {
        let matches = def.all_variants().map(|var| {
            let ident = var.ident();
            if var.is_unit() {
                quote!(Self :: #ident => matches!(other, Self :: #ident))
            } else {
                quote! {
                    Self :: #ident(it) => {
                        // NOTE: writing the match expression formats better than using `matches` macro.
                        match other {
                            Self :: #ident (other) if ContentEq::content_eq(it, other) => true,
                            _ => false,
                        }
                    }
                }
            }
        });
        quote! {
            match self {
                #(#matches),*
            }
        }
    };

    ("other", body)
}

fn derive_struct(def: &StructDef) -> (&str, TokenStream) {
    if def.fields.is_empty() {
        ("_", quote!(true))
    } else {
        let fields = def
            .fields
            .iter()
            .filter(|field| {
                !IGNORE_FIELD_TYPES.iter().any(|it| field.typ.name().inner_name() == *it)
            })
            .map(|field| {
                let ident = field.ident();
                quote!(ContentEq::content_eq(&self.#ident, &other.#ident))
            })
            .collect_vec();
        if fields.is_empty() {
            ("_", quote!(true))
        } else {
            ("other", quote!(#(#fields)&&*))
        }
    }
}

fn impl_content_eq(def: &TypeDef, other_name: &str, body: &TokenStream) -> TokenStream {
    let ty = if def.has_lifetime() { def.to_elided_type() } else { def.to_type_elide() };
    let other = other_name.to_ident();

    quote! {
        impl ContentEq for #ty {
            fn content_eq(&self, #other: &Self) -> bool {
                #body
            }
        }
    }
}
