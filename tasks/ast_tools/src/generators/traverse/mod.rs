//! Generator for `oxc_traverse` crate.
//!
//! Generates 3 files:
//! * `traverse.rs` - `Traverse` trait with `enter_*` / `exit_*` methods.
//! * `walk.rs` - Unsafe `walk_*` functions for AST traversal.
//! * `ancestor.rs` - Ancestor tracking types and offset constants.

mod ancestor;
mod walk;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    Codegen, Generator, TRAVERSE_CRATE_PATH,
    output::{Output, output_path},
    schema::{Def, Schema, TypeDef},
};

use self::ancestor::is_ast_type_with_visitor;

use super::define_generator;

pub struct TraverseGenerator;

define_generator!(TraverseGenerator);

impl Generator for TraverseGenerator {
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        vec![
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "traverse.rs"),
                tokens: generate_traverse_trait(schema),
            },
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "walk.rs"),
                tokens: walk::generate_walk(schema),
            },
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "ancestor.rs"),
                tokens: ancestor::generate_ancestor(schema),
            },
        ]
    }
}

/// Generate `Traverse` trait with `enter_*` / `exit_*` methods for each visited type.
fn generate_traverse_trait(schema: &Schema) -> TokenStream {
    let mut methods = quote!();

    for type_def in &schema.types {
        if !is_ast_type_with_visitor(type_def, schema) {
            continue;
        }

        let (visitor_names, ty) = match type_def {
            TypeDef::Struct(s) => (s.visit.visitor_names.as_ref().unwrap(), s.ty(schema)),
            TypeDef::Enum(e) => (e.visit.visitor_names.as_ref().unwrap(), e.ty(schema)),
            _ => continue,
        };

        let snake_name = traverse_snake_name(visitor_names);
        let enter_ident = quote::format_ident!("enter_{snake_name}");
        let exit_ident = quote::format_ident!("exit_{snake_name}");

        methods.extend(quote! {
            ///@@line_break
            #[inline]
            fn #enter_ident(&mut self, node: &mut #ty, ctx: &mut TraverseCtx<'a, State>) {}
            #[inline]
            fn #exit_ident(&mut self, node: &mut #ty, ctx: &mut TraverseCtx<'a, State>) {}
        });
    }

    // Special "Statements" type: Vec<'a, Statement<'a>>
    methods.extend(quote! {
        ///@@line_break
        #[inline]
        fn enter_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, State>) {}
        #[inline]
        fn exit_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a, State>) {}
    });

    quote! {
        use oxc_allocator::Vec;
        use oxc_ast::ast::*;

        ///@@line_break
        use crate::TraverseCtx;

        ///@@line_break
        #[expect(unused_variables)]
        pub trait Traverse<'a, State> {
            #methods
        }
    }
}

/// Extract snake_name from visitor names (strip `visit_` prefix).
///
/// e.g. `VisitorNames { visit: "visit_program", .. }` -> `"program"`.
pub(super) fn traverse_snake_name(
    visitor_names: &crate::schema::extensions::visit::VisitorNames,
) -> &str {
    visitor_names.visit.strip_prefix("visit_").unwrap()
}
