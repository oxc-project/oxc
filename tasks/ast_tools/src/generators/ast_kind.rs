//! Generator of code related to `AstKind`.
//!
//! * `AstType` type definition.
//! * `AstKind` type definition.
//! * `AstKind::ty` method.
//! * `AstKind::as_*` methods.
//! * `GetSpan` impl for `AstKind`.
//! * `GetAddress` impl for `AstKind`.
//!
//! Variants of `AstKind` and `AstType` are created for:
//!
//! * All structs which are visited, and are not listed in `STRUCTS_BLACK_LIST` below.

use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
    utils::number_lit,
};

use super::define_generator;

/// Structs to omit creating an `AstKind` for.
///
/// Apart from this list, every struct with `#[ast(visit)]` attr gets an `AstKind`.
///
/// `BindingPattern` and `Span` are special cases:
///
/// * `Span` we don't want to have an `AstKind` because it's not an AST node.
///   Once we have `NodeId` stored in AST types, it won't need to be visited.
///   So then it won't get an `AstKind` automatically, and can be removed from this blacklist.
///
/// * `BindingPattern` we intend to change into an enum.
///   <https://github.com/oxc-project/oxc/issues/11489#issuecomment-2946791520>
///
/// These 2 should continue to be blacklisted for now.
///
/// See also: <https://github.com/oxc-project/oxc/issues/11490>
const STRUCTS_BLACK_LIST: &[&str] = &["BindingPattern", "Span"];

/// Generator for `AstKind`, `AstType`, and related code.
pub struct AstKindGenerator;

define_generator!(AstKindGenerator);

impl Generator for AstKindGenerator {
    /// Set `has_kind` for structs and enums.
    ///
    /// All visited structs have an `AstKind`, unless included in `STRUCTS_BLACK_LIST`.
    /// Enums do not have an `AstKind`, unless included in `ENUMS_WHITE_LIST`.
    fn prepare(&self, schema: &mut Schema, _codegen: &Codegen) {
        // Set `has_kind = true` for all visited structs
        for type_def in &mut schema.types {
            if let TypeDef::Struct(struct_def) = type_def {
                struct_def.kind.has_kind = struct_def.visit.has_visitor();
            }
        }

        // Set `has_kind = false` for structs in black list
        for &type_name in STRUCTS_BLACK_LIST {
            let type_def = schema.type_by_name_mut(type_name);
            let TypeDef::Struct(struct_def) = type_def else {
                panic!("Type which isn't a struct `{}` in `STRUCTS_BLACK_LIST`", type_def.name());
            };
            assert!(
                struct_def.kind.has_kind,
                "`{}` struct is not visited - doesn't need to be in `STRUCTS_BLACK_LIST`",
                struct_def.name()
            );
            struct_def.kind.has_kind = false;
        }
    }

    /// Generate `AstKind` etc definitions.
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        let mut type_variants = quote!();
        let mut kind_variants = quote!();
        let mut span_match_arms = quote!();
        let mut address_match_arms = quote!();
        let mut as_methods = quote!();

        let mut next_index = 0u16;
        for type_def in &schema.types {
            let has_kind = match type_def {
                TypeDef::Struct(struct_def) => struct_def.kind.has_kind,
                TypeDef::Enum(enum_def) => enum_def.kind.has_kind,
                _ => false,
            };
            if !has_kind {
                continue;
            }

            let type_ident = type_def.ident();
            let type_ty = type_def.ty(schema);

            assert!(u8::try_from(next_index).is_ok());
            let index = number_lit(next_index);
            type_variants.extend(quote!( #type_ident = #index, ));
            kind_variants.extend(quote!( #type_ident(&'a #type_ty) = AstType::#type_ident as u8, ));

            span_match_arms.extend(quote!( Self::#type_ident(it) => it.span(), ));

            let get_address = match type_def {
                TypeDef::Struct(_) => quote!(Address::from_ref(it)),
                TypeDef::Enum(_) => quote!(it.address()),
                _ => unreachable!(),
            };
            address_match_arms.extend(quote!( Self::#type_ident(it) => #get_address, ));

            let as_method_name = format_ident!("as_{}", type_def.snake_name());
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
            #![expect(missing_docs)] ///@ FIXME (in ast_tools/src/generators/ast_kind.rs)

            ///@@line_break
            use std::ptr;

            ///@@line_break
            use oxc_allocator::{Address, GetAddress};
            use oxc_span::{GetSpan, Span};

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
            }

            ///@@line_break
            impl GetSpan for AstKind<'_> {
                fn span(&self) -> Span {
                    match self {
                        #span_match_arms
                    }
                }
            }

            ///@@line_break
            impl GetAddress for AstKind<'_> {
                // TODO: Once only structs have `AstKind`s (https://github.com/oxc-project/oxc/issues/11490),
                // mark this method `#[inline]`, because then it'll be boiled down to a single instruction.
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
