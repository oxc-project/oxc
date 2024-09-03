use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    codegen::LateCtx,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::ToIdent,
};

use super::{define_derive, Derive, DeriveOutput};

define_derive! {
    pub struct DeriveGetSpan;
}

impl Derive for DeriveGetSpan {
    fn trait_name() -> &'static str {
        "GetSpan"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        let self_type = quote!(&self);
        let result_type = quote!(Span);
        let result_expr = quote!(self.span);

        derive(Self::trait_name(), "span", &self_type, &result_type, &result_expr, def)
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpan};
        }
    }
}

define_derive! {
    pub struct DeriveGetSpanMut;
}

impl Derive for DeriveGetSpanMut {
    fn trait_name() -> &'static str {
        "GetSpanMut"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        let self_type = quote!(&mut self);
        let result_type = quote!(&mut Span);
        let result_expr = quote!(&mut self.span);

        derive(Self::trait_name(), "span_mut", &self_type, &result_type, &result_expr, def)
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpanMut};
        }
    }
}

fn derive(
    trait_name: &str,
    method_name: &str,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
    def: &TypeDef,
) -> TokenStream {
    let trait_ident = trait_name.to_ident();
    let method_ident = method_name.to_ident();
    match &def {
        TypeDef::Enum(def) => derive_enum(def, &trait_ident, &method_ident, self_type, result_type),
        TypeDef::Struct(def) => {
            derive_struct(def, &trait_ident, &method_ident, self_type, result_type, result_expr)
        }
    }
}

fn derive_enum(
    def: &EnumDef,
    trait_name: &Ident,
    method_name: &Ident,
    self_type: &TokenStream,
    result_type: &TokenStream,
) -> TokenStream {
    let target_type = def.to_type();
    let generics = def.generics();

    let matches = def.all_variants().map(|var| {
        let ident = var.ident();
        quote!(Self :: #ident(it) => it.#method_name())
    });

    quote! {
        impl #generics #trait_name for #target_type {
            fn #method_name(#self_type) -> #result_type {
                match self {
                    #(#matches),*
                }
            }
        }
    }
}

fn derive_struct(
    def: &StructDef,
    trait_name: &Ident,
    method_name: &Ident,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
) -> TokenStream {
    let target_type = def.to_type();
    let generics = def.generics();

    let span_field = def.fields.iter().find(|field| field.markers.span);
    let result_expr = if let Some(span_field) = span_field {
        let ident = span_field.name.as_ref().map(ToIdent::to_ident).unwrap();
        quote!(self.#ident.#method_name())
    } else {
        result_expr.clone()
    };

    quote! {
        impl #generics #trait_name for #target_type {
            #[inline]
            fn #method_name(#self_type) -> #result_type {
                #result_expr
            }
        }
    }
}
