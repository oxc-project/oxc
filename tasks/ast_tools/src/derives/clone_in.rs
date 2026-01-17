//! Derive for `CloneIn` trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    Result,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef},
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
            #![allow(unused_variables, clippy::default_trait_access, clippy::inline_always)]

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
    let type_ident = struct_def.ident();

    let (clone_in_body, clone_in_with_semantic_ids_body) = if struct_def.clone_in.is_default {
        let clone_in_body = quote!(Default::default());
        (clone_in_body.clone(), clone_in_body)
    } else {
        let has_fields = !struct_def.fields.is_empty();
        let clone_in_body = if has_fields {
            let fields = struct_def.fields.iter().map(|field| {
                let field_ident = field.ident();
                // Special case: node_id uses NodeId::DUMMY when cloning
                if field.name() == "node_id" {
                    quote!( #field_ident: oxc_syntax::node::NodeId::DUMMY )
                } else if struct_field_is_default(field, schema) {
                    quote!( #field_ident: Default::default() )
                } else {
                    quote!( #field_ident: CloneIn::clone_in(&self.#field_ident, allocator) )
                }
            });
            quote!( #type_ident { #(#fields),* } )
        } else {
            quote!( #type_ident )
        };

        let clone_in_with_semantic_ids_body = if has_fields {
            let fields = struct_def.fields.iter().map(|field| {
                let field_ident = field.ident();
                // Special case: node_id is copied directly to preserve semantic IDs
                if field.name() == "node_id" {
                    quote!( #field_ident: self.#field_ident )
                } else {
                    quote!( #field_ident: CloneIn::clone_in_with_semantic_ids(&self.#field_ident, allocator) )
                }
            });
            quote!( #type_ident { #(#fields),* } )
        } else {
            quote!( #type_ident )
        };

        (clone_in_body, clone_in_with_semantic_ids_body)
    };

    generate_impl(
        &type_ident,
        &clone_in_body,
        &clone_in_with_semantic_ids_body,
        struct_def.has_lifetime,
        false,
    )
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
    let type_ident = enum_def.ident();

    let (clone_in_body, clone_in_with_semantic_ids_body) = if enum_def.clone_in.is_default {
        let clone_in_body = quote!(Default::default());
        (clone_in_body.clone(), clone_in_body)
    } else if enum_def.is_fieldless() {
        // Fieldless enums are always `Copy`
        let clone_in_body = quote!(*self);
        (clone_in_body.clone(), clone_in_body)
    } else {
        let match_arms = enum_def.all_variants(schema).map(|variant| {
            let ident = variant.ident();
            if variant.is_fieldless() {
                quote!( Self::#ident => #type_ident::#ident )
            } else {
                quote!( Self::#ident(it) => #type_ident::#ident(CloneIn::clone_in(it, allocator)) )
            }
        });
        let clone_in_body = quote! {
            match self {
                #(#match_arms),*
            }
        };

        let match_arms = enum_def.all_variants(schema).map(|variant| {
            let ident = variant.ident();
            if variant.is_fieldless() {
                quote!( Self::#ident => #type_ident::#ident )
            } else {
                quote!( Self::#ident(it) => #type_ident::#ident(CloneIn::clone_in_with_semantic_ids(it, allocator)) )
            }
        });
        let clone_in_with_semantic_ids_body = quote! {
            match self {
                #(#match_arms),*
            }
        };

        (clone_in_body, clone_in_with_semantic_ids_body)
    };

    // Note: Add `#[inline(always)]` to methods for fieldless enums, because they're no-ops
    generate_impl(
        &type_ident,
        &clone_in_body,
        &clone_in_with_semantic_ids_body,
        enum_def.has_lifetime,
        enum_def.is_fieldless(),
    )
}

fn generate_impl(
    type_ident: &Ident,
    clone_in_body: &TokenStream,
    clone_in_with_semantic_ids_body: &TokenStream,
    has_lifetime: bool,
    inline_always: bool,
) -> TokenStream {
    let (from_lifetime, to_lifetime) =
        if has_lifetime { (quote!( <'_> ), quote!( <'new_alloc> )) } else { (quote!(), quote!()) };

    let inline = if inline_always { quote!( #[inline(always)] ) } else { quote!() };

    quote! {
        impl<'new_alloc> CloneIn<'new_alloc> for #type_ident #from_lifetime {
            type Cloned = #type_ident #to_lifetime;

            ///@@line_break
            #inline
            fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
                #clone_in_body
            }

            ///@@line_break
            #inline
            fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
                #clone_in_with_semantic_ids_body
            }
        }
    }
}
