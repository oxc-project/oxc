use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    markers::CloneInAttribute,
    schema::{EnumDef, GetIdent, Schema, StructDef, TypeDef},
};

use super::{define_derive, Derive};

pub struct DeriveCloneIn;

define_derive!(DeriveCloneIn);

impl Derive for DeriveCloneIn {
    fn trait_name() -> &'static str {
        "CloneIn"
    }

    fn prelude() -> TokenStream {
        quote! {
            #![allow(clippy::default_trait_access)]

            ///@@line_break
            use oxc_allocator::{Allocator, CloneIn};
        }
    }

    fn derive(&mut self, def: &TypeDef, _: &Schema) -> TokenStream {
        match &def {
            TypeDef::Enum(it) => derive_enum(it),
            TypeDef::Struct(it) => derive_struct(it),
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
    let clone_in_body = quote! {
        match self {
            #(#matches),*
        }
    };

    let clone_in_with_semantic_ids_token_stream = if used_alloc {
        let matches = def
        .all_variants()
        .map(|var| {
            let ident = var.ident();
            if var.is_unit() {
                quote!(Self :: #ident => #ty_ident :: #ident)
            } else {
                quote!(Self :: #ident(it) => #ty_ident :: #ident(CloneIn::clone_in_with_semantic_ids(it, allocator)))
            }
        })
        .collect_vec();

        let alloc_ident_param = if def.has_lifetime {
            quote!(#alloc_ident: &'new_alloc Allocator)
        } else {
            quote!(#alloc_ident: &'alloc Allocator)
        };
        quote!(
            ///@@line_break
            fn clone_in_with_semantic_ids(&self, #alloc_ident_param) -> Self::Cloned {
                match self {
                    #(#matches),*
                }
            }
        )
    } else {
        quote!()
    };

    impl_clone_in(
        &ty_ident,
        def.has_lifetime,
        &alloc_ident,
        &clone_in_body,
        &clone_in_with_semantic_ids_token_stream,
    )
}

fn derive_struct(def: &StructDef) -> TokenStream {
    let ty_ident = def.ident();
    let (alloc_ident, clone_in_body, clone_in_with_semantic_ids_function) = if def.fields.is_empty()
    {
        (format_ident!("_"), quote!(#ty_ident), quote!())
    } else {
        let alloc_ident = format_ident!("allocator");
        let clone_in_fields = def.fields.iter().map(|field| {
            let ident = field.ident();
            match field.markers.derive_attributes.clone_in {
                CloneInAttribute::Default => {
                    quote!(#ident: Default::default())
                }
                CloneInAttribute::None => {
                    quote!(#ident: CloneIn::clone_in(&self.#ident, allocator))
                }
            }
        });
        let clone_in_with_semantic_ids_token_stream = {
            let fields = def.fields.iter().map(|field| {
                let ident = field.ident();
                quote!(#ident: CloneIn::clone_in_with_semantic_ids(&self.#ident, allocator))
            });
            let alloc_ident_param = if def.has_lifetime {
                quote!(#alloc_ident: &'new_alloc Allocator)
            } else {
                quote!(#alloc_ident: &'alloc Allocator)
            };
            quote!(
                ///@@line_break
                fn clone_in_with_semantic_ids(&self, #alloc_ident_param) -> Self::Cloned {
                    #ty_ident { #(#fields),* }
                }
            )
        };
        (
            alloc_ident,
            quote!(#ty_ident { #(#clone_in_fields),* }),
            clone_in_with_semantic_ids_token_stream,
        )
    };

    impl_clone_in(
        &ty_ident,
        def.has_lifetime,
        &alloc_ident,
        &clone_in_body,
        &clone_in_with_semantic_ids_function,
    )
}

fn impl_clone_in(
    ty_ident: &Ident,
    has_lifetime: bool,
    alloc_ident: &Ident,
    clone_in_body: &TokenStream,
    clone_in_with_semantic_ids_function: &TokenStream,
) -> TokenStream {
    if has_lifetime {
        quote! {
            impl <'new_alloc> CloneIn<'new_alloc> for #ty_ident<'_> {
                type Cloned = #ty_ident<'new_alloc>;
                fn clone_in(&self, #alloc_ident: &'new_alloc Allocator) -> Self::Cloned {
                    #clone_in_body
                }

                #clone_in_with_semantic_ids_function
            }
        }
    } else {
        quote! {
            impl <'alloc> CloneIn<'alloc> for #ty_ident {
                type Cloned = #ty_ident;
                fn clone_in(&self, #alloc_ident: &'alloc Allocator) -> Self::Cloned {
                    #clone_in_body
                }

                #clone_in_with_semantic_ids_function
            }
        }
    }
}
