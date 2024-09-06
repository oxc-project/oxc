use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::{define_derive, Derive, DeriveOutput};
use crate::{
    codegen::LateCtx,
    markers::CloneInAttribute,
    schema::{EnumDef, GetIdent, StructDef, TypeDef},
};

define_derive! {
    pub struct DeriveCloneIn;
}

impl Derive for DeriveCloneIn {
    fn trait_name() -> &'static str {
        "CloneIn"
    }

    fn derive(&mut self, def: &TypeDef, _: &LateCtx) -> TokenStream {
        match &def {
            TypeDef::Enum(it) => derive_enum(it),
            TypeDef::Struct(it) => derive_struct(it),
        }
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::default_trait_access)]

            ///@@line_break
            use oxc_allocator::{Allocator, CloneIn};
        }
    }
}

fn derive_enum(def: &EnumDef) -> TokenStream {
    let ty_ident = def.ident();

    let mut used_alloc = false;
    let matches = def
        .all_variants()
        .map(|var| {
            let ident = var.ident();
            if var.is_unit() {
                quote!(Self :: #ident => #ty_ident :: #ident)
            } else {
                used_alloc = true;
                quote!(Self :: #ident(it) => #ty_ident :: #ident(CloneIn::clone_in(it, allocator)))
            }
        })
        .collect_vec();

    let alloc_ident = if used_alloc { format_ident!("allocator") } else { format_ident!("_") };
    let body = quote! {
        match self {
            #(#matches),*
        }
    };

    impl_clone_in(&ty_ident, def.has_lifetime, &alloc_ident, &body)
}

fn derive_struct(def: &StructDef) -> TokenStream {
    let ty_ident = def.ident();

    let (alloc_ident, body) = if def.fields.is_empty() {
        (format_ident!("_"), quote!(#ty_ident))
    } else {
        let fields = def.fields.iter().map(|field| {
            let ident = field.ident();
            match field.markers.derive_attributes.clone_in {
                CloneInAttribute::Default => quote!(#ident: Default::default()),
                CloneInAttribute::None => {
                    quote!(#ident: CloneIn::clone_in(&self.#ident, allocator))
                }
            }
        });
        (format_ident!("allocator"), quote!(#ty_ident { #(#fields),* }))
    };

    impl_clone_in(&ty_ident, def.has_lifetime, &alloc_ident, &body)
}

fn impl_clone_in(
    ty_ident: &Ident,
    has_lifetime: bool,
    alloc_ident: &Ident,
    body: &TokenStream,
) -> TokenStream {
    if has_lifetime {
        quote! {
            impl <'old_alloc, 'new_alloc> CloneIn<'new_alloc> for #ty_ident<'old_alloc> {
                type Cloned = #ty_ident<'new_alloc>;
                fn clone_in(&self, #alloc_ident: &'new_alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    } else {
        quote! {
            impl <'alloc> CloneIn<'alloc> for #ty_ident {
                type Cloned = #ty_ident;
                fn clone_in(&self, #alloc_ident: &'alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    }
}
