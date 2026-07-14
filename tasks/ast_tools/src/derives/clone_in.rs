//! Derive for `CloneIn` trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    Result,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef},
    utils::create_ident,
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
            #![allow(unused_imports, unused_variables, clippy::default_trait_access, clippy::inline_always)]

            ///@@line_break
            use std::cell::Cell;

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

    let (body, body_uses_flag) = if struct_def.clone_in.is_default {
        (quote!(Default::default()), false)
    } else if struct_def.fields.is_empty() {
        (quote!( #type_ident ), false)
    } else {
        let fields = struct_def.fields.iter().map(|field| {
            let field_ident = field.ident();
            if struct_field_is_default(field, schema) {
                // Fields (or fields whose type is) marked `#[clone_in(default)]` always reset to
                // their default. Semantic id `Cell`s are *not* marked — they thread the
                // `with_semantic_ids` flag through their `CloneIn` impl instead (see `oxc_allocator`).
                quote!( #field_ident: Default::default() )
            } else {
                quote!( #field_ident: CloneIn::clone_in_impl(&self.#field_ident, with_semantic_ids, allocator) )
            }
        });
        (quote!( #type_ident { #(#fields),* } ), true)
    };

    generate_impl(&type_ident, &body, struct_def.has_lifetime, false, body_uses_flag)
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

    let (body, body_uses_flag) = if enum_def.clone_in.is_default {
        (quote!(Default::default()), false)
    } else if enum_def.is_fieldless() {
        // Fieldless enums are always `Copy`
        (quote!(*self), false)
    } else {
        (derive_enum_body(enum_def, &type_ident, schema), true)
    };

    // Add `#[inline(always)]` to methods for fieldless enums, because they're no-ops
    let inline_always = enum_def.is_fieldless();

    generate_impl(&type_ident, &body, enum_def.has_lifetime, inline_always, body_uses_flag)
}

/// Generate the `match` body for an enum's `clone_in_impl` method.
///
/// Own variants are cloned arm-by-arm. Variants inherited via `INHERIT` are *not* expanded individually.
/// Instead they are delegated to the inherited enum's own `CloneIn` impl, using `to_*` reference cast
/// (narrow `&Self` to `&Inherited`), and the generated `From` impl (widen the clone back to `Self`).
/// Both casts are zero-cost because the inherited variants share discriminants and layout with the parent enum.
///
/// This avoids re-emitting the parent enum's variant arms (e.g. all of `Expression`'s ~30 variants)
/// in every inheriting enum (`Argument`, `PropertyKey`, ...), which is a large source of binary bloat.
///
/// `with_semantic_ids` is threaded through as a runtime flag so a single traversal serves both
/// `clone_in` and `clone_in_with_semantic_ids`.
fn derive_enum_body(enum_def: &EnumDef, type_ident: &Ident, schema: &Schema) -> TokenStream {
    let own_arms = enum_def.variants.iter().map(|variant| {
        let ident = variant.ident();
        if variant.is_fieldless() {
            quote!( Self::#ident => #type_ident::#ident )
        } else {
            quote!( Self::#ident(it) => #type_ident::#ident(CloneIn::clone_in_impl(it, with_semantic_ids, allocator)) )
        }
    });

    let inherited_arms = enum_def.inherits.iter().map(|&inherits_id| {
        let inherited = schema.enum_def(inherits_id);

        let patterns = inherited.all_variants(schema).map(|variant| {
            let ident = variant.ident();
            if variant.is_fieldless() { quote!( Self::#ident ) } else { quote!( Self::#ident(_) ) }
        });

        let to_inherited = create_ident(&format!("to_{}", inherited.snake_name()));
        quote! {
            #(#patterns)|* => {
                #type_ident::from(CloneIn::clone_in_impl(self.#to_inherited(), with_semantic_ids, allocator))
            }
        }
    });

    quote! {
        match self {
            #(#own_arms,)*
            #(#inherited_arms,)*
        }
    }
}

fn generate_impl(
    type_ident: &Ident,
    clone_in_impl_body: &TokenStream,
    has_lifetime: bool,
    inline_always: bool,
    body_uses_flag: bool,
) -> TokenStream {
    let (from_lifetime, to_lifetime) =
        if has_lifetime { (quote!( <'_> ), quote!( <'new_alloc> )) } else { (quote!(), quote!()) };

    let inline = if inline_always { quote!( #[inline(always)] ) } else { quote!() };

    // A single `clone_in_impl` traversal serves both public methods. Name the flag `_`-prefixed when
    // the body ignores it (default structs, empty structs, fieldless enums) to avoid an unused warning.
    let flag_param =
        if body_uses_flag { quote!(with_semantic_ids) } else { quote!(_with_semantic_ids) };

    quote! {
        impl<'new_alloc> CloneIn<'new_alloc> for #type_ident #from_lifetime {
            type Cloned = #type_ident #to_lifetime;

            ///@@line_break
            #inline
            fn clone_in_impl(
                &self,
                #flag_param: bool,
                allocator: &'new_alloc Allocator,
            ) -> Self::Cloned {
                #clone_in_impl_body
            }
        }
    }
}
