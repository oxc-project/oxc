//! Generator for ID getter/setter methods on all structs with semantic ID fields
//! (`scope_id`, `symbol_id`, `reference_id`).
//!
//! e.g. Generates `scope_id` and `set_scope_id` methods on all types with a `scope_id` field.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    AST_CRATE_PATH, Codegen, Generator,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef, TypeId},
};

use super::define_generator;

/// Semantic ID types.
const SEMANTIC_ID_TYPES: [&str; 3] = ["ScopeId", "SymbolId", "ReferenceId"];

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

        let impls = schema
            .types
            .iter()
            .filter_map(|type_def| generate_for_type(type_def, &semantic_id_type_ids, schema));

        let output = quote! {
            use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

            ///@@line_break
            use crate::ast::*;

            #(#impls)*
        };

        Output::Rust { path: output_path(AST_CRATE_PATH, "get_id.rs"), tokens: output }
    }
}

fn generate_for_type(
    type_def: &TypeDef,
    semantic_id_type_ids: &[TypeId; SEMANTIC_ID_TYPES.len()],
    schema: &Schema,
) -> Option<TokenStream> {
    let TypeDef::Struct(struct_def) = type_def else { return None };

    let struct_name = struct_def.name();

    let methods = struct_def
        .fields
        .iter()
        .filter_map(|field| {
            let field_type = field.type_def(schema);
            let inner_type = field_type.as_cell()?.inner_type(schema).as_option()?.inner_type(schema);
            if !semantic_id_type_ids.contains(&inner_type.id()) {
                return None;
            }

            let field_name = field.name();
            let field_ident = field.ident();
            let inner_type_ident = inner_type.ident();

            // Generate getter method
            let inner_type_name = inner_type.name();
            let get_doc1 = format!(" Get [`{inner_type_name}`] of [`{struct_name}`].");
            let get_doc2 = format!(" Only use this method on a post-semantic AST where [`{inner_type_name}`]s are always defined.");
            let get_doc3 = format!(" Panics if `{field_name}` is [`None`].");

            let get_method = quote! {
                #[doc = #get_doc1]
                ///
                #[doc = #get_doc2]
                ///
                /// # Panics
                #[doc = #get_doc3]
                #[inline]
                pub fn #field_ident(&self) -> #inner_type_ident {
                    self.#field_ident.get().unwrap()
                }
            };

            // Generate setter method
            let set_method_ident = format_ident!("set_{field_name}");
            let set_doc = format!(" Set [`{inner_type_name}`] of [`{struct_name}`].");
            let set_method = quote! {
                #[doc = #set_doc]
                #[inline]
                pub fn #set_method_ident(&self, #field_ident: #inner_type_ident) {
                    self.#field_ident.set(Some(#field_ident));
                }
            };

            Some(quote! {
                ///@@line_break
                #get_method

                ///@@line_break
                #set_method
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
