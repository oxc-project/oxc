//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
pub mod options;
// Plugins: <https://babeljs.io/docs/plugins-list>
mod decorators;
mod es2020;
mod es2021;
mod es2022;
mod es2024;
mod react;
mod typescript;

use crate::options::TransformOptions;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

#[allow(unused)]
pub struct Transformer<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
    semantic: Semantic<'a>,
    options: TransformOptions,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        options: TransformOptions,
    ) -> Self {
        Self { allocator, source_type, semantic, options }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(self, _program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        Ok(())
    }
}
