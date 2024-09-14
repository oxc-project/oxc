use proc_macro2::TokenStream;
use quote::quote;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    schema::{EnumDef, GetGenerics, StructDef, ToType, TypeDef},
    util::TypeWrapper,
};

define_derive! {
    pub struct DeriveGetNodeId;
}

impl Derive for DeriveGetNodeId {
    fn trait_name() -> &'static str {
        "GetNodeId"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        match &def {
            TypeDef::Enum(def) => derive_enum(def),
            TypeDef::Struct(def) => derive_struct(def),
        }
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_syntax::node::{NodeId, GetNodeId};
        }
    }
}

fn derive_enum(def: &EnumDef) -> TokenStream {
    let target_type = def.to_type();
    let generics = def.generics();

    let (matches, matches_mut) = def
        .all_variants()
        .map(|var| {
            let ident = var.ident();
            let it = quote!(it);
            let (it, it_mut) = if var
                .fields
                .first()
                .is_some_and(|it| it.typ.analysis().wrapper == TypeWrapper::Box)
            {
                (&quote!(it.as_ref()), &quote!(&mut **it))
            } else {
                (&it, &it)
            };
            (
                quote!(Self :: #ident(it) => GetNodeId::node_id(#it)),
                quote!(Self :: #ident(it) => GetNodeId::node_id_mut(#it_mut)),
            )
        })
        .collect::<(Vec<_>, Vec<_>)>();

    quote! {
        impl #generics GetNodeId for #target_type {
            fn node_id(&self) -> NodeId {
                match self {
                    #(#matches),*
                }
            }

            fn node_id_mut(&mut self) -> &mut NodeId {
                match self {
                    #(#matches_mut),*
                }
            }
        }
    }
}

fn derive_struct(def: &StructDef) -> TokenStream {
    let target_type = def.to_type();
    let generics = def.generics();

    quote! {
        impl #generics GetNodeId for #target_type {
            #[inline]
            fn node_id(&self) -> NodeId {
                self.node_id
            }

            #[inline]
            fn node_id_mut(&mut self) -> &mut NodeId {
                &mut self.node_id
            }
        }
    }
}
