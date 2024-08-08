use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    codegen::LateCtx,
    output,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::ToIdent,
    Generator, GeneratorOutput,
};

use super::{define_generator, generated_header};

define_generator! {
    pub struct DeriveGetSpan;
}

impl Generator for DeriveGetSpan {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let self_type = quote!(&self);
        let result_type = quote!(Span);
        let result_expr = quote!(self.span);
        let out = derive("GetSpan", "span", &self_type, &result_type, &result_expr, ctx);

        GeneratorOutput::Stream((output(crate::AST_CRATE, "derive_get_span.rs"), out))
    }
}

define_generator! {
    pub struct DeriveGetSpanMut;
}

impl Generator for DeriveGetSpanMut {
    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let self_type = quote!(&mut self);
        let result_type = quote!(&mut Span);
        let result_expr = quote!(&mut self.span);
        let out = derive("GetSpanMut", "span_mut", &self_type, &result_type, &result_expr, ctx);

        GeneratorOutput::Stream((output(crate::AST_CRATE, "derive_get_span_mut.rs"), out))
    }
}

fn derive(
    trait_name: &str,
    method_name: &str,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
    ctx: &LateCtx,
) -> TokenStream {
    let trait_ident = trait_name.to_ident();
    let method_ident = method_name.to_ident();
    let impls: Vec<TokenStream> = ctx
        .schema()
        .into_iter()
        .filter(|def| def.generates_derive(trait_name))
        .map(|def| match &def {
            TypeDef::Enum(def) => {
                derive_enum(def, &trait_ident, &method_ident, self_type, result_type)
            }
            TypeDef::Struct(def) => {
                derive_struct(def, &trait_ident, &method_ident, self_type, result_type, result_expr)
            }
        })
        .collect();

    let header = generated_header!();

    quote! {
        #header

        insert!("#![allow(clippy::match_same_arms)]");
        endl!();

        use oxc_span::{#trait_ident, Span};
        endl!();

        use crate::ast::*;
        endl!();

        #(#impls)*
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
        endl!();
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
        endl!();
    }
}
