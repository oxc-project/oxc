use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{ScopeTree, SemanticBuilder, SymbolTable};
use oxc_traverse::TraverseCtx;

use crate::{
    ast_passes::{
        EarlyPass, LatePass, PeepholeFoldConstants, PeepholeMinimizeConditions,
        PeepholeRemoveDeadCode, RemoveSyntax,
    },
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
        let mut ctx = TraverseCtx::new(scopes, symbols, self.allocator);
        RemoveSyntax::new(self.options).build(program, &mut ctx);

        if self.options.dead_code_elimination {
            Self::dead_code_elimination(program, &mut ctx);
            return;
        }

        let mut i = 0;
        loop {
            let mut changed = false;
            let mut pass = EarlyPass::new();
            pass.build(program, &mut ctx);
            if pass.changed() {
                changed = true;
            }
            if !changed {
                break;
            }
            if i > 50 {
                debug_assert!(false, "Ran in a infinite loop.");
                break;
            }
            i += 1;
        }

        LatePass::new().build(program, &mut ctx);
    }

    fn dead_code_elimination(program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        PeepholeFoldConstants::new().build(program, ctx);
        PeepholeMinimizeConditions::new().build(program, ctx);
        PeepholeRemoveDeadCode::new().build(program, ctx);
    }
}
