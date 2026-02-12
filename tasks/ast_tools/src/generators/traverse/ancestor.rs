//! Generator for `ancestor.rs` in `oxc_traverse` crate.
//!
//! Generates:
//! * `AncestorType` enum — `#[repr(u16)]` with auto-incrementing discriminants
//! * Offset constants — `pub const OFFSET_X_Y: usize = offset_of!(X, y);`
//! * `Ancestor` enum — `#[repr(C, u16)]` with `Without` struct payloads
//! * `Without` structs — one per visited field, with accessor methods for sibling fields
//! * `is_*` methods on `Ancestor` — one per struct type
//! * `is_parent_of_*` methods on `Ancestor` — one per enum type
//! * `GetAddress` impl on `Ancestor`

use convert_case::{Case, Casing};
use cow_utils::CowUtils;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    schema::{Def, FieldDef, Schema, StructDef, TypeDef},
    utils::upper_case_first,
};

/// Generate all ancestor-related code.
pub fn generate_ancestor(schema: &Schema) -> TokenStream {
    let mut ancestor_type_variants = quote!();
    let mut ancestor_enum_variants = quote!();
    let mut is_functions = quote!();
    let mut is_parent_of_functions = quote!();
    let mut ancestor_types = quote!();
    let mut address_match_arms = quote!();

    // Map from enum name -> list of ancestor variant names that are parents of that enum
    let mut variant_names_for_enums: Vec<(String, Vec<syn::Ident>)> = Vec::new();

    let mut discriminant = 1u16;

    for type_def in &schema.types {
        let TypeDef::Struct(struct_def) = type_def else { continue };
        if !struct_def.visit.has_visitor() {
            continue;
        }

        // Get visited fields (fields whose innermost type has a visitor)
        let visited_fields: Vec<(usize, &FieldDef)> = struct_def
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field_is_visited(field, schema))
            .collect();

        if visited_fields.is_empty() {
            continue;
        }

        let type_snake_name = struct_def.snake_name();
        let type_screaming_name = type_snake_name.cow_to_ascii_uppercase();
        let struct_ident = struct_def.ident();
        let struct_ty = struct_def.ty(schema);
        let has_lifetime = struct_def.has_lifetime;

        // Generate offset constants for ALL fields (not just visited)
        let mut offset_code = quote!();
        for field in &struct_def.fields {
            let const_name = format_ident!(
                "OFFSET_{}_{}",
                type_screaming_name,
                field.name().cow_to_ascii_uppercase()
            );
            let field_ident = field.ident();
            offset_code.extend(quote! {
                pub(crate) const #const_name: usize = offset_of!(#struct_ident, #field_ident);
            });
        }

        // Generate Without structs, ancestor variants, etc for each visited field
        let mut variant_names: Vec<syn::Ident> = Vec::new();
        let mut without_structs = quote!();

        for (_, field) in &visited_fields {
            let field_camel_name = upper_case_first(&field.camel_name()).into_owned();
            let variant_name = format_ident!("{}{}", struct_def.name(), field_camel_name);
            let without_struct_name =
                format_ident!("{}Without{}", struct_def.name(), field_camel_name);

            // Generate accessor methods for all OTHER fields
            let mut methods_code = quote!();
            for other_field in &struct_def.fields {
                if other_field.name() == field.name() {
                    continue;
                }

                let other_field_ident = other_field.ident();
                let other_field_ty = other_field.type_def(schema).ty(schema);
                let other_screaming = format_ident!(
                    "OFFSET_{}_{}",
                    type_screaming_name,
                    other_field.name().cow_to_ascii_uppercase()
                );

                methods_code.extend(quote! {
                    ///@@line_break
                    #[inline]
                    pub fn #other_field_ident(self) -> &'t #other_field_ty {
                        unsafe {
                            &*((self.0 as *const u8).add(#other_screaming) as *const #other_field_ty)
                        }
                    }
                });
            }

            // Lifetimes for Without struct
            let lifetimes = if has_lifetime { quote!(<'a, 't>) } else { quote!(<'t>) };

            without_structs.extend(quote! {
                ///@@line_break
                #[repr(transparent)]
                #[derive(Clone, Copy, Debug)]
                pub struct #without_struct_name #lifetimes (
                    pub(crate) *const #struct_ty,
                    pub(crate) PhantomData<&'t ()>,
                );

                ///@@line_break
                impl #lifetimes #without_struct_name #lifetimes {
                    #methods_code
                }

                ///@@line_break
                impl #lifetimes GetAddress for #without_struct_name #lifetimes {
                    #[inline]
                    fn address(&self) -> Address {
                        unsafe { Address::from_ptr(self.0) }
                    }
                }
            });

            // AncestorType variant
            let disc_lit = crate::utils::number_lit(discriminant);
            ancestor_type_variants.extend(quote! {
                #variant_name = #disc_lit,
            });

            // Ancestor variant
            ancestor_enum_variants.extend(quote! {
                #variant_name(#without_struct_name #lifetimes) = AncestorType::#variant_name as u16,
            });

            // GetAddress match arm
            address_match_arms.extend(quote! {
                Self::#variant_name(a) => a.address(),
            });

            // Track parent-of-enum relationships
            let inner_type = field.type_def(schema).innermost_type(schema);
            if let TypeDef::Enum(enum_def) = inner_type
                && enum_def.visit.has_visitor()
            {
                let enum_name = enum_def.name().to_string();
                if let Some(entry) =
                    variant_names_for_enums.iter_mut().find(|(name, _)| name == &enum_name)
                {
                    entry.1.push(variant_name.clone());
                } else {
                    variant_names_for_enums.push((enum_name, vec![variant_name.clone()]));
                }
            }

            variant_names.push(variant_name);
            discriminant += 1;
        }

        ancestor_types.extend(quote! {
            ///@@line_break
            #offset_code
            ///@@line_break
            #without_structs
        });

        // Generate is_* method for this struct
        let is_method_name = format_ident!("is_{}", type_snake_name);
        let variant_patterns: Vec<_> = variant_names.iter().map(|v| quote!(Self::#v(_))).collect();
        is_functions.extend(quote! {
            ///@@line_break
            #[inline]
            pub fn #is_method_name(self) -> bool {
                matches!(self, #(#variant_patterns)|*)
            }
        });
    }

    // Generate is_parent_of_* methods for enums
    for (enum_name, variant_names) in &variant_names_for_enums {
        let method_name = format_ident!("is_parent_of_{}", enum_name.to_case(Case::Snake));
        let variant_patterns: Vec<_> = variant_names.iter().map(|v| quote!(Self::#v(_))).collect();
        is_parent_of_functions.extend(quote! {
            ///@@line_break
            #[inline]
            pub fn #method_name(self) -> bool {
                matches!(self, #(#variant_patterns)|*)
            }
        });
    }

    quote! {
        #![expect(
            clippy::cast_ptr_alignment,
            clippy::elidable_lifetime_names,
            clippy::ptr_as_ptr,
            clippy::ref_option,
            clippy::undocumented_unsafe_blocks,
        )]
        #![allow(clippy::redundant_pub_crate)]

        ///@@line_break
        use std::{cell::Cell, marker::PhantomData, mem::offset_of};

        ///@@line_break
        use oxc_allocator::{Address, Box, GetAddress, Vec};
        use oxc_ast::ast::*;
        use oxc_syntax::{node::NodeId, scope::ScopeId};

        ///@@line_break
        /// Type of [`Ancestor`].
        /// Used in [`crate::TraverseCtx::retag_stack`].
        #[repr(u16)]
        #[derive(Clone, Copy)]
        pub(crate) enum AncestorType {
            None = 0,
            #ancestor_type_variants
        }

        ///@@line_break
        /// Ancestor type used in AST traversal.
        ///
        /// Encodes both the type of the parent, and child's location in the parent.
        /// i.e. variants for `BinaryExpressionLeft` and `BinaryExpressionRight`, not just `BinaryExpression`.
        ///
        /// `'a` is lifetime of AST nodes.
        /// `'t` is lifetime of the `Ancestor` (which inherits lifetime from `&'t TraverseCtx'`).
        /// i.e. `Ancestor`s can only exist within the body of `enter_*` and `exit_*` methods
        /// and cannot "escape" from them.
        ///@
        ///@ SAFETY
        ///@ * This type must be `#[repr(u16)]`.
        ///@ * Variant discriminants must correspond to those in `AncestorType`.
        ///@
        ///@ These invariants make it possible to set the discriminant of an `Ancestor` without altering
        ///@ the "payload" pointer with:
        ///@ `*(ancestor as *mut _ as *mut AncestorType) = AncestorType::Program`.
        ///@ `TraverseCtx::retag_stack` uses this technique.
        #[repr(C, u16)]
        #[derive(Clone, Copy, Debug)]
        pub enum Ancestor<'a, 't> {
            None = AncestorType::None as u16,
            #ancestor_enum_variants
        }

        ///@@line_break
        impl<'a, 't> Ancestor<'a, 't> {
            #is_functions
            ///@@line_break
            #is_parent_of_functions
        }

        ///@@line_break
        impl<'a, 't> GetAddress for Ancestor<'a, 't> {
            /// Get memory address of node represented by `Ancestor` in the arena.
            ///@ Compiler should reduce this down to only a couple of assembly operations.
            #[inline]
            fn address(&self) -> Address {
                match self {
                    Self::None => Address::DUMMY,
                    #address_match_arms
                }
            }
        }

        ///@@line_break
        #ancestor_types
    }
}

/// Check if a field is "visited" - i.e. its innermost type is an AST type with a visitor.
///
/// This corresponds to the JS logic: `field.innerTypeName in types`.
/// The JS scripts only consider types from `crates/oxc_ast/src/ast/{js,jsx,literal,ts}.rs`,
/// so we filter to types in the `oxc_ast` crate with import path starting with `::ast::`.
fn field_is_visited(field: &FieldDef, schema: &Schema) -> bool {
    let inner_type = field.type_def(schema).innermost_type(schema);
    is_ast_type_with_visitor(inner_type, schema)
}

/// Check if a type is an AST type (from `oxc_ast::ast::*`) with a visitor.
pub fn is_ast_type_with_visitor(type_def: &TypeDef, schema: &Schema) -> bool {
    match type_def {
        TypeDef::Struct(s) => s.visit.has_visitor() && is_ast_file(s.file(schema)),
        TypeDef::Enum(e) => e.visit.has_visitor() && is_ast_file(e.file(schema)),
        _ => false,
    }
}

/// Check if a file is one of the AST definition files.
fn is_ast_file(file: &crate::schema::File) -> bool {
    file.krate() == "oxc_ast" && file.import_path().starts_with("::ast::")
}

/// Determine the visited fields of a struct (reusable by walk.rs).
pub fn get_visited_fields<'a>(
    struct_def: &'a StructDef,
    schema: &'a Schema,
) -> Vec<(usize, &'a FieldDef)> {
    struct_def
        .fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field_is_visited(field, schema))
        .collect()
}
