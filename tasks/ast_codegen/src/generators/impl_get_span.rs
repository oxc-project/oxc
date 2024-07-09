use std::collections::HashMap;

use itertools::Itertools;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Attribute, Variant};

use crate::{
    schema::{REnum, RStruct, RType},
    CodegenCtx, Generator, GeneratorOutput,
};

use super::generated_header;

pub struct ImplGetSpanGenerator;

impl Generator for ImplGetSpanGenerator {
    fn name(&self) -> &'static str {
        "ImplGetSpanGenerator"
    }

    fn generate(&mut self, ctx: &CodegenCtx) -> GeneratorOutput {
        let impls: Vec<TokenStream> = ctx
            .ty_table
            .iter()
            .map(|it| it.borrow())
            .filter(|it| it.visitable())
            .filter(|it| matches!(&**it, RType::Enum(_) | RType::Struct(_)))
            .map(|kind| match &*kind {
                RType::Enum(it) => impl_enum(it),
                RType::Struct(it) => impl_struct(it),
                _ => unreachable!("already filtered out!"),
            })
            .collect();

        let header = generated_header!();

        GeneratorOutput::Stream((
            "span",
            quote! {
                #header
                insert!("#![allow(clippy::match_same_arms)]");
                endl!();

                use crate::ast::*;
                use oxc_span::{GetSpan, Span};

                #(#impls)*
            },
        ))
    }
}

fn impl_enum(it @ REnum { item, .. }: &REnum) -> TokenStream {
    let typ = it.as_type();
    let generics = &item.generics;
    let matches: Vec<TokenStream> = item
        .variants
        .iter()
        .map(|Variant { ident, .. }| quote!(Self :: #ident(it) => it.span()))
        .collect_vec();

    quote! {
        endl!();
        impl #generics GetSpan for #typ {
            fn span(&self) -> Span {
                match self {
                    #(#matches),*
                }
            }
        }
    }
}

fn impl_struct(it @ RStruct { item, .. }: &RStruct) -> TokenStream {
    let typ = it.as_type();
    let generics = &item.generics;
    let inner_span_hint =
        item.fields.iter().find(|it| it.attrs.iter().any(|a| a.path().is_ident("span")));
    let span = if let Some(span_field) = inner_span_hint {
        let ident = span_field.ident.as_ref().unwrap();
        quote!(#ident.span())
    } else {
        quote!(span)
    };
    quote! {
        endl!();
        impl #generics GetSpan for #typ {
            #[inline]
            fn span(&self) -> Span {
                self.#span
            }
        }
    }
}
