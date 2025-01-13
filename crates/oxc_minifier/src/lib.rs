//! ECMAScript Minifier

mod ast_passes;
mod compressor;
mod ctx;
mod keep_var;
mod options;

#[cfg(test)]
mod tester;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::Mangler;
use oxc_semantic::SemanticBuilder;

pub use oxc_mangler::MangleOptions;

pub use crate::{ast_passes::CompressorPass, compressor::Compressor, options::CompressOptions};

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: Option<MangleOptions>,
    pub compress: CompressOptions,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: Some(MangleOptions::default()), compress: CompressOptions::default() }
    }
}

pub struct MinifierReturn {
    pub mangler: Option<Mangler>,
}

pub struct Minifier {
    options: MinifierOptions,
}

impl Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn build<'a>(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        let semantic = SemanticBuilder::new().build(program).semantic;
        let stats = semantic.stats();
        let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
        Compressor::new(allocator, self.options.compress)
            .build_with_symbols_and_scopes(symbols, scopes, program);
        let mangler = self.options.mangle.map(|options| {
            let semantic = SemanticBuilder::new().with_stats(stats).build(program).semantic;
            let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
            Mangler::default()
                .with_options(options)
                .build_with_symbols_and_scopes(symbols, &scopes, program)
        });
        MinifierReturn { mangler }
    }
}
