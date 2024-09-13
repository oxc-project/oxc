use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{ScopeTree, SemanticBuilder, SymbolTable};
use oxc_traverse::TraverseCtx;

use crate::{
    ast_passes::{
        Collapse, FoldConstants, MinimizeConditions, RemoveDeadCode, RemoveSyntax,
        SubstituteAlternateSyntax,
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
            SemanticBuilder::new("").build(program).semantic.into_symbol_table_and_scope_tree();
        self.build_with_symbols_and_scopes(symbols, scopes, program);
    }

    pub fn build_with_symbols_and_scopes(
        self,
        symbols: SymbolTable,
        scopes: ScopeTree,
        program: &mut Program<'a>,
    ) {
        let mut ctx = TraverseCtx::new(scopes, symbols, self.allocator);
        // Run separate AST passes
        // TODO: inline variables
        self.remove_syntax(program, &mut ctx);
        self.fold_constants(program, &mut ctx);
        self.minimize_conditions(program, &mut ctx);
        self.remove_dead_code(program, &mut ctx);
        // TODO: StatementFusion
        self.substitute_alternate_syntax(program, &mut ctx);
        self.collapse(program, &mut ctx);
    }

    fn remove_syntax(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.remove_syntax {
            RemoveSyntax::new(ctx.ast, self.options).build(program, ctx);
        }
    }

    fn minimize_conditions(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.minimize_conditions {
            MinimizeConditions::new(ctx.ast).build(program, ctx);
        }
    }

    fn fold_constants(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.fold_constants {
            FoldConstants::new(ctx.ast).with_evaluate(self.options.evaluate).build(program, ctx);
        }
    }

    fn substitute_alternate_syntax(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.substitute_alternate_syntax {
            SubstituteAlternateSyntax::new(ctx.ast, self.options).build(program, ctx);
        }
    }

    fn remove_dead_code(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.remove_dead_code {
            RemoveDeadCode::new(ctx.ast).build(program, ctx);
        }
    }

    fn collapse(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.collapse {
            Collapse::new(ctx.ast, self.options).build(program, ctx);
        }
    }
}
