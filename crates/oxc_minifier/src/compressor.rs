use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{ScopeTree, SemanticBuilder, SymbolTable};
use oxc_traverse::ReusableTraverseCtx;

use crate::{
    ast_passes::{DeadCodeElimination, Normalize, PeepholeOptimizations, RemoveSyntax},
    CompressOptions, CompressorPass,
};

pub struct Compressor<'a> {
    allocator: &'a Allocator,
    options: CompressOptions,
}

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator, options: CompressOptions) -> Self {
        Self { allocator, options }
    }

    pub fn build(self, program: &mut Program<'a>) {
        let (symbols, scopes) =
            SemanticBuilder::new().build(program).semantic.into_symbol_table_and_scope_tree();
        self.build_with_symbols_and_scopes(symbols, scopes, program);
    }

    pub fn build_with_symbols_and_scopes(
        self,
        symbols: SymbolTable,
        scopes: ScopeTree,
        program: &mut Program<'a>,
    ) {
        let mut ctx = ReusableTraverseCtx::new(scopes, symbols, self.allocator);
        RemoveSyntax::new(self.options).build(program, &mut ctx);
        // RemoveUnusedCode::new(self.options).build(program, &mut ctx);
        Normalize::new().build(program, &mut ctx);
        PeepholeOptimizations::new(self.options.target, true, self.options)
            .run_in_loop(program, &mut ctx);
        PeepholeOptimizations::new(self.options.target, false, self.options)
            .build(program, &mut ctx);
    }

    pub fn dead_code_elimination(self, program: &mut Program<'a>) {
        let (symbols, scopes) =
            SemanticBuilder::new().build(program).semantic.into_symbol_table_and_scope_tree();
        self.dead_code_elimination_with_symbols_and_scopes(symbols, scopes, program);
    }

    pub fn dead_code_elimination_with_symbols_and_scopes(
        self,
        symbols: SymbolTable,
        scopes: ScopeTree,
        program: &mut Program<'a>,
    ) {
        let mut ctx = ReusableTraverseCtx::new(scopes, symbols, self.allocator);
        RemoveSyntax::new(self.options).build(program, &mut ctx);
        DeadCodeElimination::new().build(program, &mut ctx);
    }
}
