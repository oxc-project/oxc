use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    output,
    schema::{EnumDef, GetIdent, StructDef, TypeDef},
    GeneratorOutput, LateCtx,
};

use super::{define_generator, generated_header, Generator};

define_generator! {
    pub struct DeriveCloneIn;
}

impl Generator for DeriveCloneIn {
    fn name(&self) -> &'static str {
        stringify!(DeriveCloneIn)
    }

    fn generate(&mut self, ctx: &LateCtx) -> GeneratorOutput {
        let impls: Vec<TokenStream> = ctx
            .schema
            .definitions
            .iter()
            .filter(|def| def.generates_derive("CloneIn"))
            .map(|def| match &def {
                TypeDef::Enum(it) => derive_enum(it),
                TypeDef::Struct(it) => derive_struct(it),
            })
            .collect();

        let header = generated_header!();

        GeneratorOutput::Stream((
            output(crate::AST_CRATE, "derive_clone_in.rs"),
            quote! {
                #header

                use oxc_allocator::{Allocator, CloneIn};
                endl!();
                use crate::ast::*;
                endl!();

                #(#impls)*
            },
        ))
    }
}

fn derive_enum(def: &EnumDef) -> TokenStream {
    let ty_ident = def.ident();
    let (alloc, body) = {
        let mut used_alloc = false;
        let matches = def
            .all_variants()
            .map(|var| {
                let ident = var.ident();
                if var.is_unit() {
                    quote!(Self :: #ident => #ty_ident :: #ident)
                } else {
                    used_alloc = true;
                    quote!(Self :: #ident(it) => #ty_ident :: #ident(it.clone_in(alloc)))
                }
            })
            .collect_vec();
        let alloc_ident = if used_alloc { format_ident!("alloc") } else { format_ident!("_") };
        (
            alloc_ident,
            quote! {
                match self {
                    #(#matches),*
                }
            },
        )
    };
    impl_clone_in(&ty_ident, def.has_lifetime, &alloc, &body)
}

fn derive_struct(def: &StructDef) -> TokenStream {
    let ty_ident = def.ident();
    let (alloc, body) = {
        let (alloc_ident, body) = if def.fields.is_empty() {
            (format_ident!("_"), TokenStream::default())
        } else {
            let fields = def.fields.iter().map(|field| {
                let ident = field.ident();
                quote!(#ident: self.#ident.clone_in(alloc))
            });
            (format_ident!("alloc"), quote!({ #(#fields),* }))
        };
        (alloc_ident, quote!( #ty_ident #body ))
    };
    impl_clone_in(&ty_ident, def.has_lifetime, &alloc, &body)
}

fn impl_clone_in(
    ty_ident: &Ident,
    has_lifetime: bool,
    alloc: &Ident,
    body: &TokenStream,
) -> TokenStream {
    if has_lifetime {
        quote! {
            endl!();
            impl <'old_alloc, 'new_alloc> CloneIn<'new_alloc> for #ty_ident<'old_alloc> {
                type Cloned = #ty_ident<'new_alloc>;
                fn clone_in(&self, #alloc: &'new_alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    } else {
        quote! {
            endl!();
            impl <'alloc> CloneIn<'alloc> for #ty_ident {
                type Cloned = #ty_ident;
                fn clone_in(&self, #alloc: &'alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    }
}
