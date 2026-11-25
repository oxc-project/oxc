//! Generator for `FullEq` impls, used in `oxc_coverage`.
//!
//! `FullEq` is a stricter version of `ContentEq`. It compares every field except `node_id`,
//! including spans, semantic IDs, and fields marked `#[content_eq(skip)]` (e.g. `raw`).
//!
//! The trait, and its impls for container types (`Option`, `Box`, `Vec`, `Cell`),
//! are in `tasks/coverage/src/full_eq.rs`.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    COVERAGE_CRATE_PATH, Codegen, Generator,
    codegen::DeriveId,
    output::{Output, output_path},
    schema::{Def, EnumDef, PrimitiveDef, Schema, StructDef, TypeDef, TypeId, Visibility},
};

use super::define_generator;

/// Generator for `FullEq` impls.
pub struct FullEqGenerator;

define_generator!(FullEqGenerator);

impl Generator for FullEqGenerator {
    /// Generate `FullEq` impls for all AST types.
    fn generate(&self, schema: &Schema, codegen: &Codegen) -> Output {
        Output::Rust {
            path: output_path(COVERAGE_CRATE_PATH, "derive_full_eq.rs"),
            tokens: generate(schema, codegen),
        }
    }
}

/// Generate `FullEq` impls.
///
/// - Structs and enums which derive `ContentEq` compare every field or variant with `FullEq`.
/// - Types which `ContentEq` ignores (`#[content_eq(skip)]`), structs with private fields,
///   fieldless enums, and primitives compare with `PartialEq`.
/// - `NodeId` is always equal, so `node_id` fields are ignored.
///
/// Generated code:
///
/// ```ignore
/// impl FullEq for StringLiteral<'_> {
///     fn full_eq(&self, other: &Self) -> bool {
///         FullEq::full_eq(&self.span, &other.span)
///             && FullEq::full_eq(&self.value, &other.value)
///             && FullEq::full_eq(&self.raw, &other.raw)
///     }
/// }
///
/// impl FullEq for Span {
///     fn full_eq(&self, other: &Self) -> bool {
///         self == other
///     }
/// }
/// ```
fn generate(schema: &Schema, codegen: &Codegen) -> TokenStream {
    let content_eq_derive_id = codegen.get_derive_id_by_name("ContentEq");
    let node_id_type_id = schema.type_names["NodeId"];

    let impls = schema.types.iter().filter_map(|type_def| {
        let body = match type_def {
            TypeDef::Struct(struct_def) => {
                generate_struct_body(struct_def, node_id_type_id, content_eq_derive_id)
            }
            TypeDef::Enum(enum_def) => generate_enum_body(enum_def, content_eq_derive_id, schema),
            TypeDef::Primitive(primitive_def) => generate_primitive_body(primitive_def),
            _ => None,
        }?;

        let ty = type_def.ty_anon(schema);
        Some(quote! {
            ///@@line_break
            impl FullEq for #ty {
                fn full_eq(&self, other: &Self) -> bool {
                    #body
                }
            }
        })
    });

    quote! {
        #![allow(clippy::match_same_arms, unused_variables)]

        ///@@line_break
        use std::num::NonZeroU32;

        ///@@line_break
        use oxc::{
            ast::ast::*,
            regular_expression::ast::*,
            span::FileExtension,
            str::JSStr,
            syntax::{
                label::LabelId, node::NodeId, reference::ReferenceId, scope::ScopeId,
                symbol::SymbolId,
            },
        };

        ///@@line_break
        use crate::full_eq::FullEq;

        #(#impls)*
    }
}

/// Generate body of `full_eq` method for a struct.
///
/// - `NodeId` is always equal, so `node_id` fields are ignored.
/// - Types which `ContentEq` ignores compare with `PartialEq`.
/// - Structs which derive `ContentEq` compare every field.
///   Or compare with `PartialEq` if they have private fields e.g. `bitflags!` types, `SourceType`,
///   because `oxc_coverage` can't access those fields.
///
/// Returns `None` for other structs.
///
/// Generated code:
///
/// ```ignore
/// FullEq::full_eq(&self.span, &other.span) && FullEq::full_eq(&self.value, &other.value)
/// ```
fn generate_struct_body(
    struct_def: &StructDef,
    node_id_type_id: TypeId,
    content_eq_derive_id: DeriveId,
) -> Option<TokenStream> {
    if struct_def.id == node_id_type_id {
        return Some(quote!(true));
    }
    if struct_def.content_eq.skip {
        return Some(quote!(self == other));
    }
    if !struct_def.generates_derive(content_eq_derive_id) {
        return None;
    }
    if struct_def.fields.iter().any(|field| field.visibility != Visibility::Public) {
        return Some(quote!(self == other));
    }

    let fields = struct_def
        .fields
        .iter()
        .map(|field| {
            let ident = field.ident();
            quote!( FullEq::full_eq(&self.#ident, &other.#ident) )
        })
        .collect::<Vec<_>>();

    assert!(!fields.is_empty(), "`FullEq` found no fields to compare in `{}`", struct_def.name());

    Some(quote!( #(#fields)&&* ))
}

/// Generate body of `full_eq` method for an enum.
///
/// - Types which `ContentEq` ignores, and fieldless enums, compare with `PartialEq`.
/// - Other enums which derive `ContentEq` compare every variant.
///
/// Returns `None` for other enums.
///
/// Generated code:
///
/// ```ignore
/// match (self, other) {
///     (Self::BooleanLiteral(a), Self::BooleanLiteral(b)) => FullEq::full_eq(a, b),
///     (Self::NullLiteral(a), Self::NullLiteral(b)) => FullEq::full_eq(a, b),
///     // ...
///     _ => false,
/// }
/// ```
fn generate_enum_body(
    enum_def: &EnumDef,
    content_eq_derive_id: DeriveId,
    schema: &Schema,
) -> Option<TokenStream> {
    if enum_def.content_eq.skip {
        return Some(quote!(self == other));
    }
    if !enum_def.generates_derive(content_eq_derive_id) {
        return None;
    }
    // Same as `ContentEq` derive, assume fieldless enums implement `PartialEq`
    if enum_def.is_fieldless() {
        return Some(quote!(self == other));
    }

    let matches = enum_def.all_variants(schema).map(|variant| {
        let ident = variant.ident();
        if variant.is_fieldless() {
            quote!( (Self::#ident, Self::#ident) => true )
        } else {
            quote!( (Self::#ident(a), Self::#ident(b)) => FullEq::full_eq(a, b) )
        }
    });

    Some(quote! {
        match (self, other) {
            #(#matches,)*
            _ => false,
        }
    })
}

/// Generate body of `full_eq` method for a primitive.
///
/// Compare with `PartialEq`, except floats, which compare as bits, same as `ContentEq` does.
/// Otherwise `NaN` would not equal itself.
///
/// Returns `None` for atomics, which don't implement `PartialEq`,
/// and `PointerAlign`, which is private to `oxc_span`.
///
/// Generated code:
///
/// ```ignore
/// self == other
/// ```
fn generate_primitive_body(primitive_def: &PrimitiveDef) -> Option<TokenStream> {
    match primitive_def.name() {
        // Private type
        "PointerAlign" => None,
        // `Atomic*` types do not implement `PartialEq`
        name if name.starts_with("Atomic") => None,
        // See `ContentEq` impl for why `.to_bits()`
        "f32" | "f64" => Some(quote!(self.to_bits() == other.to_bits())),
        _ => Some(quote!(self == other)),
    }
}
