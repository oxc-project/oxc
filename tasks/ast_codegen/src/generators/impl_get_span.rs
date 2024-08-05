use proc_macro2::TokenStream;
use quote::quote;
use syn::Variant;

use crate::{
    output,
    schema::{REnum, RStruct, RType},
    CodegenCtx, Generator, GeneratorOutput,
};

use super::{define_generator, generated_header};

define_generator! {
    pub struct ImplGetSpanGenerator;
}

impl Generator for ImplGetSpanGenerator {
    fn name(&self) -> &'static str {
        stringify!(ImplGetSpanGenerator)
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
            output(crate::AST_CRATE, "span.rs"),
            quote! {
                #header
                insert!("#![allow(clippy::match_same_arms)]");
                endl!();

                use crate::ast::*;
                use oxc_span::{GetSpan, GetSpanMut, Span};

                #(#impls)*
            },
        ))
    }
}

fn impl_enum(it @ REnum { item, .. }: &REnum) -> TokenStream {
    let typ = it.as_type();
    let generics = &item.generics;
    let (matches, matches_mut): (Vec<TokenStream>, Vec<TokenStream>) = item
        .variants
        .iter()
        .map(|Variant { ident, .. }| {
            (quote!(Self :: #ident(it) => it.span()), quote!(Self :: #ident(it) => it.span_mut()))
        })
        .unzip();

    quote! {
        endl!();
        impl #generics GetSpan for #typ {
            fn span(&self) -> Span {
                match self {
                    #(#matches),*
                }
            }
        }
        endl!();

        impl #generics GetSpanMut for #typ {
            fn span_mut(&mut self) -> &mut Span {
                match self {
                    #(#matches_mut),*
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
    let (span, span_mut) = if let Some(span_field) = inner_span_hint {
        let ident = span_field.ident.as_ref().unwrap();
        (quote!(self.#ident.span()), quote!(self.#ident.span_mut()))
    } else {
        (quote!(self.span), quote!(&mut self.span))
    };
    quote! {
        endl!();
        impl #generics GetSpan for #typ {
            #[inline]
            fn span(&self) -> Span {
                #span
            }
        }
        endl!();

        impl #generics GetSpanMut for #typ {
            #[inline]
            fn span_mut(&mut self) -> &mut Span {
                #span_mut
            }
        }
    }
}
