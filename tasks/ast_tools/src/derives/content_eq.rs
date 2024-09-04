use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::LateCtx,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::ToIdent,
};

use super::{define_derive, Derive, DeriveOutput};

define_derive! {
    pub struct DeriveContentEq;
}

const IGNORE_FIELDS: [(/* field name */ &str, /* field type */ &str); 5] = [
    ("span", "Span"),
    ("trailing_comma", "Span"),
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
                        matches!(other, Self :: #ident (other) if it.content_eq(other))
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
                quote!(self.#ident.content_eq(&other.#ident))
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
