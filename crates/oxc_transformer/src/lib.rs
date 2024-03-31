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
mod react_display_name;
mod react_jsx;
mod react_jsx_self;
mod react_jsx_source;
mod typescript;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

pub use crate::{
    compiler_assumptions::CompilerAssumptions,
    decorators::{Decorators, DecoratorsOptions},
    react_display_name::{ReactDisplayName, ReactDisplayNameOptions},
    react_jsx::{ReactJsx, ReactJsxOptions},
    react_jsx_self::{ReactJsxSelf, ReactJsxSelfOptions},
    react_jsx_source::{ReactJsxSource, ReactJsxSourceOptions},
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
    pub decorators: DecoratorsOptions,
    pub typescript: TypeScriptOptions,
    pub react_jsx: ReactJsxOptions,
    pub react_display_name: ReactDisplayNameOptions,
    pub react_jsx_self: ReactJsxSelfOptions,
    pub react_jsx_source: ReactJsxSourceOptions,
}

#[allow(unused)]
pub struct Transformer<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
    semantic: Semantic<'a>,
    options: TransformOptions,

    // Stage 3
    decorators: Decorators,
    // [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    typescript: TypeScript,
    // [preset-react](https://babeljs.io/docs/babel-preset-react)
    react_display_name: ReactDisplayName,
    react_jsx: ReactJsx,
    react_jsx_self: ReactJsxSelf,
    react_jsx_source: ReactJsxSource,
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
            react_display_name: ReactDisplayName::default(),
            react_jsx: ReactJsx::default(),
            react_jsx_self: ReactJsxSelf::default(),
            react_jsx_source: ReactJsxSource::default(),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(self, _program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        Ok(())
    }
}
