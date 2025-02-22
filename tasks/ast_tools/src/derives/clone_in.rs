//! Derive for `CloneIn` trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    Result,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef},
    utils::create_safe_ident,
};

use super::{
    AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum, attr_positions, define_derive,
};

/// Derive for `CloneIn` trait.
pub struct DeriveCloneIn;

define_derive!(DeriveCloneIn);

impl Derive for DeriveCloneIn {
    fn trait_name(&self) -> &'static str {
        "CloneIn"
    }

    fn trait_has_lifetime(&self) -> bool {
        true
    }

    fn crate_name(&self) -> &'static str {
        "oxc_allocator"
    }

    /// Register that accept `#[clone_in]` attr on structs, enums, or struct fields.
    /// Allow attr on structs and enums which don't derive this trait.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("clone_in", attr_positions!(StructMaybeDerived | EnumMaybeDerived | StructField))]
    }

    /// Parse `#[clone_in(default)]` on struct, enum, or struct field.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `clone_in`, because that's the only attribute this derive handles.
        if !matches!(part, AttrPart::Tag("default")) {
            return Err(());
        }

        match location {
            AttrLocation::Struct(struct_def) => struct_def.clone_in.is_default = true,
            AttrLocation::Enum(enum_def) => enum_def.clone_in.is_default = true,
            AttrLocation::StructField(struct_def, field_index) => {
                struct_def.fields[field_index].clone_in.is_default = true;
            }
            _ => return Err(()),
        }

        Ok(())
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
            StructOrEnum::Struct(struct_def) => derive_struct(struct_def, schema),
            StructOrEnum::Enum(enum_def) => derive_enum(enum_def, schema),
        }
    }
}

fn derive_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    assert!(
        !struct_def.clone_in.is_default,
        "Cannot derive `CloneIn` on a type which has a `#[clone_in(default)]` attribute: `{}`",
        struct_def.name()
    );

    let type_ident = struct_def.ident();
    let has_fields = !struct_def.fields.is_empty();
    let body = if has_fields {
        let fields = struct_def.fields.iter().map(|field| {
            let field_ident = field.ident();
            if struct_field_is_default(field, schema) {
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

/// Get if a struct field should be filled with default value when cloning.
///
/// This is that case if either:
/// 1. Struct field has `#[clone_in(default)]` attr. or
/// 2. The field's type has `#[clone_in(default)]` attr.
fn struct_field_is_default(field: &FieldDef, schema: &Schema) -> bool {
    if field.clone_in.is_default {
        true
    } else {
        let innermost_type = field.type_def(schema).innermost_type(schema);
        match innermost_type {
            TypeDef::Struct(struct_def) => struct_def.clone_in.is_default,
            TypeDef::Enum(enum_def) => enum_def.clone_in.is_default,
            _ => false,
        }
    }
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    assert!(
        !enum_def.clone_in.is_default,
        "Cannot derive `CloneIn` on a type which has a `#[clone_in(default)]` attribute: `{}`",
        enum_def.name()
    );

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
    let alloc_ident = create_safe_ident(if uses_allocator { "allocator" } else { "_" });

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
