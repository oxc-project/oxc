//! ECMAScript Minifier

mod compressor;
mod ctx;
mod keep_var;
mod options;
mod peephole;

#[cfg(test)]
mod tester;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_mangler::Mangler;
use oxc_semantic::{SemanticBuilder, Stats, SymbolTable};

pub use oxc_mangler::MangleOptions;

pub use crate::{compressor::Compressor, options::CompressOptions};

#[derive(Debug, Clone, Copy)]
pub struct MinifierOptions {
    pub mangle: Option<MangleOptions>,
    pub compress: Option<CompressOptions>,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self { mangle: Some(MangleOptions::default()), compress: Some(CompressOptions::default()) }
    }
}

pub struct MinifierReturn {
    pub symbol_table: Option<SymbolTable>,
}

pub struct Minifier {
    options: MinifierOptions,
}

impl Minifier {
    pub fn new(options: MinifierOptions) -> Self {
        Self { options }
    }

    pub fn build<'a>(self, allocator: &'a Allocator, program: &mut Program<'a>) -> MinifierReturn {
        let stats = if let Some(compress) = self.options.compress {
            let semantic = SemanticBuilder::new().build(program).semantic;
            let stats = semantic.stats();
            let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
            Compressor::new(allocator, compress)
                .build_with_symbols_and_scopes(symbols, scopes, program);
            stats
        } else {
            Stats::default()
        };
        let symbol_table = self.options.mangle.map(|options| {
            let semantic = SemanticBuilder::new()
                .with_stats(stats)
                .with_scope_tree_child_ids(true)
                .build(program)
                .semantic;
            Mangler::default().with_options(options).build_with_semantic(semantic, program)
        });
        MinifierReturn { symbol_table }
    }
}
