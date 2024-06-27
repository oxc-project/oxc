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

const EDGE_CASES: [&str; 1] = ["BindingPattern"];

fn edge_case(it: &std::cell::Ref<RType>) -> bool {
    !it.ident().is_some_and(|it| EDGE_CASES.contains(&it.to_string().as_str()))
}

fn edge_case_impls() -> TokenStream {
    quote! {
        endl!();
        impl<'a> GetSpan for BindingPattern<'a> {
            fn span(&self) -> Span {
                self.kind.span()
            }
        }
    }
}

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
            .filter(edge_case)
            .map(|kind| match &*kind {
                RType::Enum(it) => impl_enum(it),
                RType::Struct(it) => impl_struct(it),
                _ => unreachable!("already filtered out!"),
            })
            .collect();

        let edge_impls = edge_case_impls();

        let header = generated_header!();

        GeneratorOutput::One(quote! {
            #header
            insert!("#![allow(clippy::match_same_arms)]");
            endl!();

            use crate::ast::*;
            use oxc_span::{GetSpan, Span};

            #(#impls)*

            #edge_impls

        })
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
    quote! {
        endl!();
        impl #generics GetSpan for #typ {
            #[inline]
            fn span(&self) -> Span {
                self.span
            }
        }
    }
}
