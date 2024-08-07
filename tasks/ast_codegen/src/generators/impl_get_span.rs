use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    output,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::ToIdent,
    Generator, GeneratorOutput, LateCtx,
};

use super::{define_generator, generated_header};

define_generator! {
    pub struct ImplGetSpanGenerator;
}

impl Generator for ImplGetSpanGenerator {
    fn name(&self) -> &'static str {
        stringify!(ImplGetSpanGenerator)
    }

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let impls: Vec<TokenStream> = ctx
            .schema
            .definitions
            .iter()
            .filter(|def| def.visitable())
            .map(|def| match &def {
                TypeDef::Enum(it) => impl_enum(it),
                TypeDef::Struct(it) => impl_struct(it),
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

fn impl_enum(def: &EnumDef) -> TokenStream {
    let typ = def.to_type();
    let generics = &def.generics();
    let (matches, matches_mut): (Vec<TokenStream>, Vec<TokenStream>) = def
        .all_variants()
        .map(|var| {
            let ident = var.ident();
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

fn impl_struct(def: &StructDef) -> TokenStream {
    let typ = def.to_type();
    let generics = &def.generics();
    let inner_span_hint = def.fields.iter().find(|it| it.markers.span);
    let (span, span_mut) = if let Some(span_field) = inner_span_hint {
        let ident = span_field.name.as_ref().map(ToIdent::to_ident).unwrap();
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
