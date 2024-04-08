//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
mod options;
// Plugins: <https://babeljs.io/docs/plugins-list>
mod decorators;
mod react;
mod typescript;

pub use crate::options::{
    CompilerAssumptions, DecoratorsOptions, ReactOptions, TransformOptions, TypeScriptOptions,
};
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
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        mut options: TransformOptions,
    ) -> Self {
        options.validate();
        Self { allocator, source_type, semantic, options }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(self, _program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        Ok(())
    }
}
