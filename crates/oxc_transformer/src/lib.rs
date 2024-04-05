//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
pub mod options;
mod preset_plugin;
// Plugins: <https://babeljs.io/docs/plugins-list>
mod decorators;
mod es2020;
mod es2021;
mod es2022;
mod es2024;
mod react;
mod typescript;

pub use crate::options::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;
use preset_plugin::BoxedTransformation;

#[allow(unused)]
pub struct Transformer<'a> {
    allocator: &'a Allocator,
    source_type: SourceType,
    semantic: Semantic<'a>,
    options: TransformOptions,
    presets: Vec<BoxedTransformation>,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        mut options: TransformOptions,
    ) -> Self {
        options.validate();

        let mut presets: Vec<BoxedTransformation> = vec![];

        // Order here is very important! We start off by stripping syntax
        // that isn't valid in a standard JS file, and then we apply
        // transforms spec-by-spec for each ES version.

        if let Some(opts) = &options.typescript {
            presets.push(Box::new(crate::typescript::TypeScript::new(
                opts.to_owned(),
                options.jsx.as_ref().unwrap().to_owned(),
            )));
        }

        if let Some(opts) = &options.react {
            presets.push(Box::new(crate::react::React::new(
                opts.to_owned(),
                options.jsx.as_ref().unwrap().to_owned(),
            )));
        }

        if options.target < TransformTarget::ES2024 {
            presets.push(Box::new(crate::es2024::Es2024::new(options.es2024.clone())));
        }

        if options.target < TransformTarget::ES2020 {
            presets.push(Box::new(crate::es2020::Es2020::new(options.es2020.clone())));
        }

        Self { allocator, source_type, semantic, options, presets }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(self, program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        for mut preset in self.presets {
            preset.transform(program);
        }

        Ok(())
    }
}
