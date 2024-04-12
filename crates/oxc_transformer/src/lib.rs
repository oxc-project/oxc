#![allow(clippy::wildcard_imports)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
mod context;
mod options;
// Presets: <https://babel.dev/docs/presets>
mod decorators;
mod react;
mod typescript;

use std::{path::Path, rc::Rc};

use oxc_allocator::{Allocator, Vec};
use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;

pub use crate::{
    compiler_assumptions::CompilerAssumptions, decorators::DecoratorsOptions,
    options::TransformOptions, react::ReactOptions, typescript::TypeScriptOptions,
};

use crate::{
    context::{Ctx, TransformCtx},
    decorators::Decorators,
    react::React,
    typescript::TypeScript,
};

pub struct Transformer<'a> {
    ctx: Ctx<'a>,
    // NOTE: all callbacks must run in order.
    x0_typescript: TypeScript<'a>,
    x1_react: React<'a>,
    x2_decorators: Decorators<'a>,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_path: &Path,
        semantic: Semantic<'a>,
        options: TransformOptions,
    ) -> Self {
        let ctx = Rc::new(TransformCtx::new(allocator, source_path, semantic));
        Self {
            ctx: Rc::clone(&ctx),
            x0_typescript: TypeScript::new(options.typescript, &ctx),
            x1_react: React::new(options.react, &ctx),
            x2_decorators: Decorators::new(options.decorators, &ctx),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &mut Program<'a>) -> Result<(), std::vec::Vec<Error>> {
        self.visit_program(program);
        let errors = self.ctx.take_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> VisitMut<'a> for Transformer<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.x0_typescript.transform_statements(stmts);
        walk_mut::walk_statements_mut(self, stmts);
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.x2_decorators.transform_statement(stmt);
        walk_mut::walk_statement_mut(self, stmt);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.x1_react.transform_expression(expr);
        walk_mut::walk_expression_mut(self, expr);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        self.x1_react.transform_variable_declarator(declarator);
        walk_mut::walk_variable_declarator_mut(self, declarator);
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        self.x1_react.transform_object_property(prop);
        walk_mut::walk_object_property_mut(self, prop);
    }

    fn visit_export_default_declaration(&mut self, decl: &mut ExportDefaultDeclaration<'a>) {
        self.x1_react.transform_export_default_declaration(decl);
        walk_mut::walk_export_default_declaration_mut(self, decl);
    }

    fn visit_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        self.x1_react.transform_jsx_opening_element(elem);
        walk_mut::walk_jsx_opening_element_mut(self, elem);
    }
}
