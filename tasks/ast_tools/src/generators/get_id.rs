//! Generator for ID getter/setter methods on all structs with semantic ID fields
//! (`scope_id`, `symbol_id`, `reference_id`) and `node_id`.
//!
//! e.g. Generates `scope_id` and `set_scope_id` methods on all types with a `scope_id` field.
//! Also generates `node_id` and `set_node_id` methods on all types with a `node_id` field.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, EnumDef, Schema, StructDef, TypeId},
};

use super::define_generator;

/// Semantic ID types.
const SEMANTIC_ID_TYPES: [&str; 4] = ["NodeId", "ScopeId", "SymbolId", "ReferenceId"];

/// Generator for methods to get/set semantic IDs on structs which have them.
pub struct GetIdGenerator;

define_generator!(GetIdGenerator);

impl Generator for GetIdGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        // Get `TypeId`s for semantic ID types
        let mut semantic_id_type_ids = [TypeId::DUMMY; SEMANTIC_ID_TYPES.len()];
        for (index, &type_name) in SEMANTIC_ID_TYPES.iter().enumerate() {
            semantic_id_type_ids[index] = schema.type_names[type_name];
        }

        let struct_impls = schema.structs().filter_map(|struct_def| {
            generate_for_struct(struct_def, &semantic_id_type_ids, schema)
        });

        let enum_impls = schema.enums().filter_map(|enum_def| generate_for_enum(enum_def, schema));

        let output = quote! {
            #![expect(clippy::inline_always)]
            #![expect(clippy::match_same_arms)]

            use oxc_syntax::{node::NodeId, reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

            ///@@line_break
            use crate::ast::*;

            #(#struct_impls)*

            #(#enum_impls)*
        };

        Output::Rust { path: output_path(AST_CRATE_PATH, "get_id.rs"), tokens: output }
    }
}

fn generate_for_struct(
    struct_def: &StructDef,
    semantic_id_type_ids: &[TypeId; SEMANTIC_ID_TYPES.len()],
    schema: &Schema,
) -> Option<TokenStream> {
    let struct_name = struct_def.name();

    // Generate semantic ID getters/setters (`node_id`, `scope_id`, `symbol_id`, `reference_id`)
    let methods = struct_def
        .fields
        .iter()
        .filter_map(|field| {
            let field_type = field.type_def(schema);

            let mut inner_type = field_type.as_cell()?.inner_type(schema);
            let mut is_option = false;
            if let Some(option_def) = inner_type.as_option() {
                inner_type = option_def.inner_type(schema);
                is_option = true;
            }

            if !semantic_id_type_ids.contains(&inner_type.id()) {
                return None;
            }

            let field_name = field.name();
            let field_ident = field.ident();
            let inner_type_ident = inner_type.ident();

            // Generate getter + setter methods
            let inner_type_name = inner_type.name();
            let get_doc1 = format!(" Get [`{inner_type_name}`] of [`{struct_name}`].");
            let get_doc2 = format!(" Only use this method on a post-semantic AST where [`{inner_type_name}`]s are always defined.");
            let mut get_doc = quote! {
                #[doc = #get_doc1]
                ///
                #[doc = #get_doc2]
            };

            let (get_body, set_body) = if is_option {
                let get_doc3 = format!(" Panics if `{field_name}` is [`None`].");
                get_doc.extend(quote! {
                    ///
                    /// # Panics
                    #[doc = #get_doc3]
                });

                (
                    quote!( self.#field_ident.get().unwrap() ),
                    quote!( self.#field_ident.set(Some(#field_ident)); ),
                )
            } else {
                (
                    quote!( self.#field_ident.get() ),
                    quote!( self.#field_ident.set(#field_ident); ),
                )
            };

            let set_method_ident = format_ident!("set_{field_name}");
            let set_doc = format!(" Set [`{inner_type_name}`] of [`{struct_name}`].");
            let set_doc = quote!( #[doc = #set_doc] );

            Some(quote! {
                ///@@line_break
                #get_doc
                #[inline]
                pub fn #field_ident(&self) -> #inner_type_ident {
                    #get_body
                }

                ///@@line_break
                #set_doc
                #[inline]
                pub fn #set_method_ident(&self, #field_ident: #inner_type_ident) {
                    #set_body
                }
            })
        })
        .collect::<TokenStream>();

    if methods.is_empty() {
        return None;
    }

    let struct_ty = struct_def.ty_anon(schema);
    Some(quote! {
        ///@@line_break
        impl #struct_ty {
            #methods
        }
    })
}

fn generate_for_enum(enum_def: &EnumDef, schema: &Schema) -> Option<TokenStream> {
    // Check all variants are structs with a `NodeId` field (`has_kind` is only `true` for structs that do).
    // Also check if all variants are consistently boxed or consistently unboxed.
    let mut all_variants_boxed = true;
    let mut all_variants_unboxed = true;

    for variant in enum_def.all_variants(schema) {
        let mut field_type = variant.field_type(schema)?;
        if let Some(box_def) = field_type.as_box() {
            field_type = box_def.inner_type(schema);
            all_variants_unboxed = false;
        } else {
            all_variants_boxed = false;
        }

        let struct_def = field_type.as_struct()?;
        if !struct_def.kind.has_kind {
            return None;
        }
    }

    // If all variants are consistently boxed or consistently unboxed, add `#[inline(always)]` to the method.
    // `NodeId` field is in a consistent position in all AST structs, so if all variants have the same shape,
    // the method should boil down to a single instruction.
    let maybe_inline = if all_variants_boxed || all_variants_unboxed {
        quote! {
            ///@ `#[inline(always)]` because this should boil down to a single instruction.
            #[inline(always)]
        }
    } else {
        quote!()
    };

    let matches = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();
        quote!( Self::#variant_ident(it) => it.node_id() )
    });

    let enum_ty = enum_def.ty_anon(schema);
    let get_doc = format!(" Get [`NodeId`] of [`{}`].", enum_def.name());

    Some(quote! {
        ///@@line_break
        impl #enum_ty {
            #[doc = #get_doc]
            #maybe_inline
            pub fn node_id(&self) -> NodeId {
                match self {
                    #(#matches),*
                }
            }
        }
    })
}
