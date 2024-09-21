use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{ScopeTree, SemanticBuilder, SymbolTable};
use oxc_traverse::TraverseCtx;

use crate::{
    ast_passes::{
        CollapseVariableDeclarations, ExploitAssigns, PeepholeFoldConstants,
        PeepholeMinimizeConditions, PeepholeRemoveDeadCode, PeepholeSubstituteAlternateSyntax,
        RemoveSyntax, StatementFusion,
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
        self.remove_syntax(program, &mut ctx);

        if self.options.dead_code_elimination {
            self.dead_code_elimination(program, &mut ctx);
            return;
        }

        self.fold_constants(program, &mut ctx);
        self.minimize_conditions(program, &mut ctx);
        self.remove_dead_code(program, &mut ctx);
        // self.statement_fusion(program, &mut ctx);
        self.substitute_alternate_syntax(program, &mut ctx);
        self.collapse_variable_declarations(program, &mut ctx);
        self.exploit_assigns(program, &mut ctx);
    }

    fn dead_code_elimination(self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fold_constants(program, ctx);
        self.minimize_conditions(program, ctx);
        self.remove_dead_code(program, ctx);
    }

    fn remove_syntax(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        RemoveSyntax::new(self.options).build(program, ctx);
    }

    fn minimize_conditions(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        PeepholeMinimizeConditions::new().build(program, ctx);
    }

    fn fold_constants(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        PeepholeFoldConstants::new().with_evaluate(self.options.evaluate).build(program, ctx);
    }

    fn substitute_alternate_syntax(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        PeepholeSubstituteAlternateSyntax::new(self.options).build(program, ctx);
    }

    fn remove_dead_code(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        PeepholeRemoveDeadCode::new().build(program, ctx);
    }

    #[allow(unused)]
    fn statement_fusion(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        StatementFusion::new().build(program, ctx);
    }

    fn collapse_variable_declarations(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        CollapseVariableDeclarations::new(self.options).build(program, ctx);
    }

    fn exploit_assigns(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        ExploitAssigns::new().build(program, ctx);
    }
}
