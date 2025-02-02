//! Derive for `CloneIn` trait.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    schema::{Def, EnumDef, Schema, StructDef},
    Result,
};

use super::{define_derive, AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum};

/// Derive for `CloneIn` trait.
pub struct DeriveCloneIn;

define_derive!(DeriveCloneIn);

impl Derive for DeriveCloneIn {
    fn trait_name(&self) -> &'static str {
        "CloneIn"
    }

    /// Register that accept `#[clone_in]` attr on struct fields.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("clone_in", AttrPositions::StructField)]
    }

    /// Parse `#[clone_in(default)]` on struct field.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `clone_in`, because that's the only attribute this derive handles.
        // Ditto location can only be `StructField`.
        let AttrLocation::StructField(struct_def, field_index) = location else { unreachable!() };

        if matches!(part, AttrPart::Tag("default")) {
            struct_def.fields[field_index].clone_in.is_default = true;
            Ok(())
        } else {
            Err(())
        }
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::default_trait_access)]

            ///@@line_break
            use oxc_allocator::{Allocator, CloneIn};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => derive_struct(struct_def),
            StructOrEnum::Enum(enum_def) => derive_enum(enum_def, schema),
        }
    }
}

fn derive_struct(struct_def: &StructDef) -> TokenStream {
    let type_ident = struct_def.ident();

    let has_fields = !struct_def.fields.is_empty();
    let body = if has_fields {
        let fields = struct_def.fields.iter().map(|field| {
            let field_ident = field.ident();
            if field.clone_in.is_default {
                quote!( #field_ident: Default::default() )
            } else {
                quote!( #field_ident: CloneIn::clone_in(&self.#field_ident, allocator) )
            }
        });
        quote!( #type_ident { #(#fields),* } )
    } else {
        quote!( #type_ident )
    };

    generate_impl(&type_ident, &body, struct_def.has_lifetime, has_fields)
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let type_ident = enum_def.ident();

    let mut uses_allocator = false;
    let match_arms = enum_def.all_variants(schema).map(|variant| {
        let ident = variant.ident();
        if variant.is_fieldless() {
            quote!( Self::#ident => #type_ident::#ident )
        } else {
            uses_allocator = true;
            quote!( Self::#ident(it) => #type_ident::#ident(CloneIn::clone_in(it, allocator)) )
        }
    });

    let body = quote! {
        match self {
            #(#match_arms),*
        }
    };

    generate_impl(&type_ident, &body, enum_def.has_lifetime, uses_allocator)
}

fn generate_impl(
    type_ident: &Ident,
    body: &TokenStream,
    has_lifetime: bool,
    uses_allocator: bool,
) -> TokenStream {
    let alloc_ident = format_ident!("{}", if uses_allocator { "allocator" } else { "_" });

    if has_lifetime {
        quote! {
            impl<'new_alloc> CloneIn<'new_alloc> for #type_ident<'_> {
                type Cloned = #type_ident<'new_alloc>;
                fn clone_in(&self, #alloc_ident: &'new_alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    } else {
        quote! {
            impl<'alloc> CloneIn<'alloc> for #type_ident {
                type Cloned = #type_ident;
                fn clone_in(&self, #alloc_ident: &'alloc Allocator) -> Self::Cloned {
                    #body
                }
            }
        }
    }
}
