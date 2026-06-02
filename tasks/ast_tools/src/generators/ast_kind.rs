//! Generator of code related to `AstKind`.
//!
//! * `AstType` type definition.
//! * `AstKind` type definition.
//! * `AstKind::ty` method.
//! * `AstKind::node_id` & `AstKind::set_node_id` methods.
//! * `AstKind::as_*` methods.
//! * `GetSpan` impl for `AstKind`.
//! * `GetAddress` impl for `AstKind`.
//!
//! Variants of `AstKind` and `AstType` are created for all structs which have a `NodeId` field.

use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, Schema},
    utils::number_lit,
};

use super::define_generator;

/// Generator for `AstKind`, `AstType`, and related code.
pub struct AstKindGenerator;

define_generator!(AstKindGenerator);

impl Generator for AstKindGenerator {
    /// Set `has_kind` for structs and enums.
    ///
    /// All structs with a `NodeId` have an `AstKind`.
    /// Enums do not have an `AstKind`.
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {
        // Set `has_kind = true` for structs with a `NodeId`
        let node_id_cell_type_id =
            schema.type_by_name("NodeId").as_struct().unwrap().containers.cell_id.unwrap();

        for struct_def in schema.structs_mut() {
            if struct_def
                .fields
                .iter()
                .any(|field| field.type_id == node_id_cell_type_id && field.name == "node_id")
            {
                struct_def.kind.has_kind = true;
            }
        }
    }

    /// Generate `AstKind` etc definitions.
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let mut type_variants = quote!();
        let mut kind_variants = quote!();
        let mut span_match_arms = quote!();
        let mut address_match_arms = quote!();
        let mut node_id_match_arms = quote!();
        let mut set_node_id_match_arms = quote!();
        let mut as_methods = quote!();

        let mut next_index = 0u16;
        for struct_def in schema.structs() {
            if !struct_def.kind.has_kind {
                continue;
            }

            let type_ident = struct_def.ident();
            let type_ty = struct_def.ty(schema);

            assert!(u8::try_from(next_index).is_ok());
            let index = number_lit(next_index);
            type_variants.extend(quote!( #type_ident = #index, ));
            kind_variants.extend(quote!( #type_ident(&'a #type_ty) = AstType::#type_ident as u8, ));

            span_match_arms.extend(quote!( Self::#type_ident(it) => it.span(), ));

            address_match_arms.extend(quote!( Self::#type_ident(it) => it.unstable_address(), ));

            node_id_match_arms.extend(quote!( Self::#type_ident(it) => it.node_id(), ));
            set_node_id_match_arms
                .extend(quote!( Self::#type_ident(it) => it.set_node_id(node_id), ));

            let as_method_name = format_ident!("as_{}", struct_def.snake_name());
            as_methods.extend(quote! {
                ///@@line_break
                #[inline]
                pub fn #as_method_name(self) -> Option<&'a #type_ty> {
                    if let Self::#type_ident(v) = self {
                        Some(v)
                    } else {
                        None
                    }
                }
            });

            next_index += 1;
        }

        let ast_type_max = number_lit(next_index - 1);

        let output = quote! {
            ///@@line_break
            use std::ptr;

            ///@@line_break
            use oxc_allocator::{Address, GetAddress, UnstableAddress};
            use oxc_span::{GetSpan, Span};
            use oxc_syntax::node::NodeId;

            ///@@line_break
            use crate::ast::*;

            ///@@line_break
            /// The largest integer value that can be mapped to an `AstType`/`AstKind` enum variant.
            pub const AST_TYPE_MAX: u8 = #ast_type_max;

            ///@@line_break
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[repr(u8)]
            pub enum AstType {
                #type_variants
            }

            ///@@line_break
            /// Untyped AST Node Kind
            #[derive(Debug, Clone, Copy)]
            #[repr(C, u8)]
            pub enum AstKind<'a> {
                #kind_variants
            }

            ///@@line_break
            impl AstKind<'_> {
                /// Get the [`AstType`] of an [`AstKind`].
                #[inline]
                pub fn ty(&self) -> AstType {
                    ///@ SAFETY: `AstKind` is `#[repr(C, u8)]`, so discriminant is stored in first byte,
                    ///@ and it's valid to read it.
                    ///@ `AstType` is also `#[repr(u8)]` and `AstKind` and `AstType` both have the same
                    ///@ discriminants, so it's valid to read `AstKind`'s discriminant as `AstType`.
                    unsafe { *ptr::from_ref(self).cast::<AstType>().as_ref().unwrap_unchecked() }
                }

                ///@@line_break
                /// Get [`NodeId`] of an [`AstKind`].
                ///@ `node_id` field is in consistent position in all AST structs, so this boils down to 1 instruction.
                #[inline]
                pub fn node_id(&self) -> NodeId {
                    match self {
                        #node_id_match_arms
                    }
                }

                ///@@line_break
                /// Set [`NodeId`] of an [`AstKind`].
                ///@ `node_id` field is in consistent position in all AST structs, so this boils down to 1 instruction.
                #[inline]
                pub fn set_node_id(&self, node_id: NodeId) {
                    match self {
                        #set_node_id_match_arms
                    }
                }
            }

            ///@@line_break
            impl GetSpan for AstKind<'_> {
                ///@@line_break
                /// Get [`Span`] of an [`AstKind`].
                ///@ `span` field is in consistent position in all AST structs, so this boils down to 1 instruction.
                #[inline]
                fn span(&self) -> Span {
                    match self {
                        #span_match_arms
                    }
                }
            }

            ///@@line_break
            impl GetAddress for AstKind<'_> {
                ///@@line_break
                /// Get [`Address`] of an [`AstKind`].
                ///@ This boils down to 1 instruction.
                ///@ In all cases, it gets the pointer from the reference in the `AstKind`.
                #[inline]
                fn address(&self) -> Address {
                    match *self {
                        #address_match_arms
                    }
                }
            }

            ///@@line_break
            impl<'a> AstKind<'a> {
                #as_methods
            }
        };

        Output::Rust { path: output_path(AST_CRATE_PATH, "ast_kind.rs"), tokens: output }
    }
}
