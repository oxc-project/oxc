use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::{ToIdent, TypeWrapper},
};

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
        let unbox = |it| quote!(#it.as_ref());
        let reference = |it| quote!(&#it);

        derive(
            Self::trait_name(),
            "span",
            &self_type,
            &result_type,
            &result_expr,
            def,
            unbox,
            reference,
        )
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
        let unbox = |it| quote!(&mut **#it);
        let reference = |it| quote!(&mut #it);

        derive(
            Self::trait_name(),
            "span_mut",
            &self_type,
            &result_type,
            &result_expr,
            def,
            unbox,
            reference,
        )
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpanMut};
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn derive<U, R>(
    trait_name: &str,
    method_name: &str,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
    def: &TypeDef,
    unbox: U,
    reference: R,
) -> TokenStream
where
    U: Fn(TokenStream) -> TokenStream,
    R: Fn(TokenStream) -> TokenStream,
{
    let trait_ident = trait_name.to_ident();
    let method_ident = method_name.to_ident();
    match &def {
        TypeDef::Enum(def) => {
            derive_enum(def, &trait_ident, &method_ident, self_type, result_type, unbox)
        }
        TypeDef::Struct(def) => derive_struct(
            def,
            &trait_ident,
            &method_ident,
            self_type,
            result_type,
            result_expr,
            reference,
        ),
    }
}

fn derive_enum<U>(
    def: &EnumDef,
    trait_name: &Ident,
    method_name: &Ident,
    self_type: &TokenStream,
    result_type: &TokenStream,
    unbox: U,
) -> TokenStream
where
    U: Fn(TokenStream) -> TokenStream,
{
    let target_type = def.to_type();
    let generics = def.generics();

    let matches = def.all_variants().map(|var| {
        let ident = var.ident();
        let mut it = quote!(it);
        if var.fields.first().is_some_and(|it| it.typ.analysis().wrapper == TypeWrapper::Box) {
            it = unbox(it);
        }
        quote!(Self :: #ident(it) => #trait_name :: #method_name(#it))
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

fn derive_struct<R>(
    def: &StructDef,
    trait_name: &Ident,
    method_name: &Ident,
    self_type: &TokenStream,
    result_type: &TokenStream,
    result_expr: &TokenStream,
    reference: R,
) -> TokenStream
where
    R: Fn(TokenStream) -> TokenStream,
{
    let target_type = def.to_type();
    let generics = def.generics();

    let span_field = def.fields.iter().find(|field| field.markers.span);
    let result_expr = if let Some(span_field) = span_field {
        let ident = span_field.name.as_ref().map(ToIdent::to_ident).unwrap();
        let reference = reference(quote!(self.#ident));
        quote!(#trait_name :: #method_name (#reference))
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
