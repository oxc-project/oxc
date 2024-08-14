use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_quote;

use super::{
    defs::{EnumDef, StructDef, TypeDef, TypeRef},
    GetGenerics, GetIdent,
};

pub trait ToType {
    fn to_type(&self) -> syn::Type;
    fn to_type_elide(&self) -> syn::Type;
    fn to_type_with_explicit_generics(&self, generics: TokenStream) -> syn::Type;
}

impl ToType for TypeRef {
    fn to_type(&self) -> syn::Type {
        syn::parse_str(self.raw()).unwrap()
    }

    fn to_type_elide(&self) -> syn::Type {
        self.to_type_with_explicit_generics(proc_macro2::TokenStream::default())
    }

    fn to_type_with_explicit_generics(&self, generics: proc_macro2::TokenStream) -> syn::Type {
        let ident = self.name().first_ident();
        parse_quote!(#ident #generics)
    }
}

auto_impl_to_type! {
    TypeDef,
    EnumDef,
    StructDef,
}

macro_rules! auto_impl_to_type {
    ($($ty:ty,)+) => (
        $(
            impl ToType for $ty {
                fn to_type(&self) -> syn::Type {
                    self.to_type_with_explicit_generics(self.generics().to_token_stream())
                }

                fn to_type_elide(&self) -> syn::Type {
                    self.to_type_with_explicit_generics(TokenStream::default())
                }

                fn to_type_with_explicit_generics(&self, generics: TokenStream) -> syn::Type {
                    let name = self.ident();
                    parse_quote!(#name #generics)
                }
            }
        )+
    )
}

use auto_impl_to_type;
