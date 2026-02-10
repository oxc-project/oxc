//! Generator for `oxc_traverse` crate.
//!
//! Generates 3 files:
//! * `traverse.rs` - `Traverse` trait with `enter_*` / `exit_*` methods.
//! * `walk.rs` - Unsafe `walk_*` functions for AST traversal.
//! * `ancestor.rs` - Ancestor tracking types and offset constants.

pub(super) mod ancestor;
pub(super) mod walk;

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
        let config = TraverseTraitConfig::traverse();
        vec![
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "traverse.rs"),
                tokens: generate_traverse_trait(schema, &config),
            },
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "walk.rs"),
                tokens: generate_walk_traverse(schema),
            },
            Output::Rust {
                path: output_path(TRAVERSE_CRATE_PATH, "ancestor.rs"),
                tokens: ancestor::generate_ancestor(schema),
            },
        ]
    }
}

pub(super) fn generate_walk_traverse(schema: &Schema) -> TokenStream {
    walk::generate_walk(schema, &walk::WalkConfig::traverse())
}

pub(super) fn generate_walk_minifier(schema: &Schema) -> TokenStream {
    walk::generate_walk(schema, &walk::WalkConfig::minifier())
}

pub(super) fn generate_ancestor(schema: &Schema) -> TokenStream {
    ancestor::generate_ancestor(schema)
}

pub(super) struct TraverseTraitConfig {
    pub trait_ident: syn::Ident,
    pub trait_generics: TokenStream,
    pub ctx_ty: TokenStream,
    pub ctx_use: TokenStream,
}

impl TraverseTraitConfig {
    pub fn traverse() -> Self {
        Self {
            trait_ident: quote::format_ident!("Traverse"),
            trait_generics: quote! { <'a, State> },
            ctx_ty: quote! { TraverseCtx<'a, State> },
            ctx_use: quote! { use crate::TraverseCtx; },
        }
    }

    pub fn minifier() -> Self {
        Self {
            trait_ident: quote::format_ident!("Traverse"),
            trait_generics: quote! { <'a> },
            ctx_ty: quote! { TraverseCtx<'a> },
            ctx_use: quote! { use crate::TraverseCtx; },
        }
    }
}

/// Generate `Traverse` trait with `enter_*` / `exit_*` methods for each visited type.
pub(super) fn generate_traverse_trait(
    schema: &Schema,
    config: &TraverseTraitConfig,
) -> TokenStream {
    let ctx_ty = &config.ctx_ty;
    let trait_ident = &config.trait_ident;
    let trait_generics = &config.trait_generics;
    let ctx_use = &config.ctx_use;
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
            fn #enter_ident(&mut self, node: &mut #ty, ctx: &mut #ctx_ty) {}
            #[inline]
            fn #exit_ident(&mut self, node: &mut #ty, ctx: &mut #ctx_ty) {}
        });
    }

    // Special "Statements" type: Vec<'a, Statement<'a>>
    methods.extend(quote! {
        ///@@line_break
        #[inline]
        fn enter_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &mut #ctx_ty) {}
        #[inline]
        fn exit_statements(&mut self, node: &mut Vec<'a, Statement<'a>>, ctx: &mut #ctx_ty) {}
    });

    quote! {
        use oxc_allocator::Vec;
        use oxc_ast::ast::*;

        ///@@line_break
        #ctx_use

        ///@@line_break
        #[expect(unused_variables)]
        pub trait #trait_ident #trait_generics {
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
