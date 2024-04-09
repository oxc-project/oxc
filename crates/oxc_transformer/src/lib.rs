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
use oxc_ast::ast::Program;
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

    decorators: Decorators,
    typescript: TypeScript,
    react: React,
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
            decorators: Decorators::default(),
            typescript: TypeScript::default(),
            react: React::default(),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(self, _program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        Ok(())
    }
}
