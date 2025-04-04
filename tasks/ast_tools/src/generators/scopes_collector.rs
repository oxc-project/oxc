//! Generator for `ChildScopesCollector` visitor.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    Codegen, Generator, TRAVERSE_CRATE_PATH,
    output::{Output, output_path},
    schema::{Def, Schema, StructDef, TypeDef},
    utils::create_ident,
};

use super::define_generator;

/// Generator for `ChildScopesCollector` visitor.
pub struct ScopesCollectorGenerator;

define_generator!(ScopesCollectorGenerator);

impl Generator for ScopesCollectorGenerator {
    fn generate(&self, schema: &Schema, _codegen: &Codegen) -> Output {
        // Get `TypeDef` for `ScopeId`
        let scope_id_type = schema.type_by_name("ScopeId").as_struct().unwrap();

        let visit_methods = schema
            .types
            .iter()
            .filter_map(|type_def| generate_for_type(type_def, scope_id_type, schema));

        let output = quote! {
            use std::cell::Cell;

            ///@@line_break
            use oxc_ast::ast::*;
            use oxc_ast_visit::Visit;
            use oxc_syntax::scope::{ScopeFlags, ScopeId};

            ///@@line_break
            /// Visitor that locates all child scopes.
            /// NB: Child scopes only, not grandchild scopes.
            /// Does not do full traversal - stops each time it hits a node with a scope.
            pub struct ChildScopeCollector {
                pub(crate) scope_ids: Vec<ScopeId>,
            }

            ///@@line_break
            impl ChildScopeCollector {
                pub(crate) fn new() -> Self {
                    Self { scope_ids: vec![] }
                }

                ///@@line_break
                pub(crate) fn add_scope(&mut self, scope_id: &Cell<Option<ScopeId>>) {
                    self.scope_ids.push(scope_id.get().unwrap());
                }
            }

            ///@@line_break
            impl<'a> Visit<'a> for ChildScopeCollector {
                #(#visit_methods)*
            }
        };

        Output::Rust {
            path: output_path(TRAVERSE_CRATE_PATH, "scopes_collector.rs"),
            tokens: output,
        }
    }
}

fn generate_for_type(
    type_def: &TypeDef,
    scope_id_type: &StructDef,
    schema: &Schema,
) -> Option<TokenStream> {
    let struct_def = type_def.as_struct()?;

    // Find `ScopeId` field
    let field = struct_def.fields.iter().find(|field| {
        if let TypeDef::Cell(cell_def) = field.type_def(schema) {
            if let TypeDef::Option(option_def) = cell_def.inner_type(schema) {
                return option_def.inner_type_id == scope_id_type.id;
            }
        }
        false
    })?;

    // Generate visit method
    let struct_ty = struct_def.ty(schema);
    let field_ident = field.ident();
    let visit_method_ident = struct_def.visit.visitor_ident();

    let extra_params = struct_def
        .visit
        .visit_args
        .iter()
        .map(|(_, arg_type_name)| {
            let arg_type_ident = create_ident(arg_type_name);
            quote!( , _: #arg_type_ident )
        })
        .collect::<TokenStream>();

    let visit_method = quote! {
        ///@@line_break
        #[inline]
        fn #visit_method_ident(&mut self, it: &#struct_ty #extra_params) {
            self.add_scope(&it.#field_ident);
        }
    };
    Some(visit_method)
}
