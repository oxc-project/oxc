use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, parse_str, Type};

use super::{
    defs::{EnumDef, StructDef, TypeDef, TypeRef},
    GetGenerics, GetIdent,
};

pub trait ToType {
    fn to_type(&self) -> Type;
    fn to_type_elide(&self) -> Type;
    fn to_elided_type(&self) -> Type;
    fn to_type_with_explicit_generics(&self, generics: TokenStream) -> Type;
}

impl ToType for TypeRef {
    fn to_type(&self) -> Type {
        parse_str(self.raw()).unwrap()
    }

    fn to_type_elide(&self) -> Type {
        self.to_type_with_explicit_generics(proc_macro2::TokenStream::default())
    }

    fn to_elided_type(&self) -> Type {
        self.to_type_with_explicit_generics(parse_quote! {<'_>})
    }

    fn to_type_with_explicit_generics(&self, generics: proc_macro2::TokenStream) -> Type {
        let ident = self.name().first_ident();
        parse_quote!(#ident #generics)
    }
}

macro_rules! impl_to_type {
    ($($ty:ty,)+) => (
        $(
            impl ToType for $ty {
                fn to_type(&self) -> Type {
                    self.to_type_with_explicit_generics(self.generics().to_token_stream())
                }

                fn to_type_elide(&self) -> Type {
                    self.to_type_with_explicit_generics(TokenStream::default())
                }

                fn to_elided_type(&self) -> Type {
                    self.to_type_with_explicit_generics(parse_quote! {<'_>})
                }

                fn to_type_with_explicit_generics(&self, generics: TokenStream) -> Type {
                    let name = self.ident();
                    parse_quote!(#name #generics)
                }
            }
        )+
    )
}

impl_to_type! {
    TypeDef,
    EnumDef,
    StructDef,
}
