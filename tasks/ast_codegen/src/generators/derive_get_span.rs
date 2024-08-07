use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Generics, Ident, Type};

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

define_generator! {
    pub struct DeriveGetSpanMut;
}

impl Generator for DeriveGetSpan {
    fn name(&self) -> &'static str {
        stringify!(DeriveGetSpan)
    }

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "derive_get_span.rs"),
            derive::<false>(ctx),
        ))
    }
}

impl Generator for DeriveGetSpanMut {
    fn name(&self) -> &'static str {
        stringify!(DeriveGetSpanMut)
    }

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "derive_get_span_mut.rs"),
            derive::<true>(ctx),
        ))
    }
}

fn derive<const MUT: bool>(ctx: &LateCtx) -> TokenStream {
    let (self_type, trait_ident, method_ident, result_type) = if MUT {
        (
            quote!(&mut self),
            format_ident!("GetSpanMut"),
            format_ident!("span_mut"),
            quote!(&mut Span),
        )
    } else {
        (quote!(&self), format_ident!("GetSpan"), format_ident!("span"), quote!(Span))
    };

    let derive_enum = |it: &EnumDef| {
        let generics = it.generics();
        let typ = it.to_type();
        impl_trait(
            &trait_ident,
            &method_ident,
            &generics,
            &typ,
            &self_type,
            &result_type,
            &derive_enum(it, &method_ident),
        )
    };

    let derive_struct = |it: &StructDef| {
        let generics = it.generics();
        let typ = it.to_type();
        impl_trait(
            &trait_ident,
            &method_ident,
            &generics,
            &typ,
            &self_type,
            &result_type,
            &derive_struct::<MUT>(it, &method_ident),
        )
    };
    let impls: Vec<TokenStream> = ctx
        .schema()
        .into_iter()
        .filter(|def| def.visitable())
        .map(|def| match &def {
            TypeDef::Enum(it) => derive_enum(it),
            TypeDef::Struct(it) => derive_struct(it),
        })
        .collect();

    let header = generated_header!();

    quote! {
        #header
        insert!("#![allow(clippy::match_same_arms)]");
        endl!();

        use crate::ast::*;
        use oxc_span::{#trait_ident, Span};

        #(#impls)*
    }
}

fn derive_enum(def: &EnumDef, method: &Ident) -> TokenStream {
    let matches = def.all_variants().map(|var| {
        let ident = var.ident();
        quote!(Self :: #ident(it) => it.#method())
    });

    quote! {
        match self {
            #(#matches),*
        }
    }
}

fn derive_struct<const MUT: bool>(def: &StructDef, method: &Ident) -> TokenStream {
    let inner_span_hint = def.fields.iter().find(|it| it.markers.span);
    if let Some(span_field) = inner_span_hint {
        let ident = span_field.name.as_ref().map(ToIdent::to_ident).unwrap();
        quote!(self.#ident.#method())
    } else if MUT {
        quote!(&mut self.span)
    } else {
        quote!(self.span)
    }
}

fn impl_trait(
    trait_ident: &Ident,
    method_ident: &Ident,
    generics: &Option<Generics>,
    target_type: &Type,
    self_: &TokenStream,
    result_type: &TokenStream,
    body: &TokenStream,
) -> TokenStream {
    quote! {
        endl!();
        impl #generics #trait_ident for #target_type {
            #[inline]
            fn #method_ident(#self_) -> #result_type {
                #body
            }
        }
    }
}
