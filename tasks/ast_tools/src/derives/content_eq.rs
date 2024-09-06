use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::ToIdent,
};

define_derive! {
    pub struct DeriveContentEq;
}

const IGNORE_FIELDS: [(/* field name */ &str, /* field type */ &str); 6] = [
    ("span", "Span"),
    ("trailing_comma", "Span"),
    ("this_span", "Span"),
    ("scope_id", "ScopeId"),
    ("symbol_id", "SymbolId"),
    ("reference_id", "ReferenceId"),
];

impl Derive for DeriveContentEq {
    fn trait_name() -> &'static str {
        "ContentEq"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        let (other, body) = match &def {
            TypeDef::Enum(it) => derive_enum(it),
            TypeDef::Struct(it) => derive_struct(it),
        };

        impl_content_eq(def, other, &body)
    }

    fn prelude() -> TokenStream {
        quote! {
            // NOTE: writing long match expressions formats better than using `matches` macro.
            #![allow(clippy::match_like_matches_macro)]

            ///@@line_break
            use oxc_span::cmp::ContentEq;
        }
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
                let Some(name) = field.name.as_ref() else { return false };
                !IGNORE_FIELDS
                    .iter()
                    .any(|it| name == it.0 && field.typ.name().inner_name() == it.1)
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
    let ty = def.to_type();
    let generics = def.generics();
    let other = other_name.to_ident();

    quote! {
        impl #generics ContentEq for #ty {
            fn content_eq(&self, #other: &Self) -> bool {
                #body
            }
        }
    }
}
