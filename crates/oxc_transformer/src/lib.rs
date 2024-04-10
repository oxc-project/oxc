#![allow(clippy::wildcard_imports)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
// Plugins: <https://babeljs.io/docs/plugins-list>
mod decorators;
mod react;
mod typescript;

use oxc_allocator::Allocator;
use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

pub use crate::{
    compiler_assumptions::CompilerAssumptions,
    decorators::{Decorators, DecoratorsOptions},
    react::{
        React, ReactDisplayName, ReactDisplayNameOptions, ReactJsx, ReactJsxSelf, ReactJsxSource,
        ReactJsxSourceOptions, ReactOptions,
    },
    typescript::{TypeScript, TypeScriptOptions},
};

#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [proposal-decorators](https://babeljs.io/docs/babel-plugin-proposal-decorators)
    pub decorators: DecoratorsOptions,

    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub react: ReactOptions,
}

#[allow(unused)]
pub struct Transformer<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
    semantic: Semantic<'a>,
    options: TransformOptions,

    // NOTE: all callbacks must run in order.
    x0_typescript: TypeScript,
    x1_react: React,
    x2_decorators: Decorators,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        options: TransformOptions,
    ) -> Self {
        Self {
            allocator,
            source_type,
            semantic,
            options,
            x0_typescript: TypeScript::default(),
            x1_react: React::default(),
            x2_decorators: Decorators::default(),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        self.visit_program(program);
        Ok(())
    }
}

impl<'a> VisitMut<'a> for Transformer<'a> {
    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.x0_typescript.transform_statement(stmt);
        self.x2_decorators.transform_statement(stmt);
        walk_mut::walk_statement_mut(self, stmt);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.x1_react.transform_expression(expr);
        walk_mut::walk_expression_mut(self, expr);
    }
}
