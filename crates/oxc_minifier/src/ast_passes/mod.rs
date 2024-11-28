mod collapse_variable_declarations;
mod exploit_assigns;
mod peephole_fold_constants;
mod peephole_minimize_conditions;
mod peephole_remove_dead_code;
mod peephole_replace_known_methods;
mod peephole_substitute_alternate_syntax;
mod remove_syntax;
mod statement_fusion;

pub use collapse_variable_declarations::CollapseVariableDeclarations;
pub use exploit_assigns::ExploitAssigns;
pub use peephole_fold_constants::PeepholeFoldConstants;
pub use peephole_minimize_conditions::PeepholeMinimizeConditions;
pub use peephole_remove_dead_code::PeepholeRemoveDeadCode;
pub use peephole_replace_known_methods::PeepholeReplaceKnownMethods;
pub use peephole_substitute_alternate_syntax::PeepholeSubstituteAlternateSyntax;
pub use remove_syntax::RemoveSyntax;
pub use statement_fusion::StatementFusion;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

pub trait CompressorPass<'a>: Traverse<'a> {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>);
}

// See `peepholeOptimizationsOnce`

// For pass:
// ```
//     if (options.collapseVariableDeclarations) {
//     passes.maybeAdd(exploitAssign);
//     passes.maybeAdd(collapseVariableDeclarations);
//   }
// ```
pub struct CollapsePass {
    _x0_exploit_assigns: ExploitAssigns,
    x1_collapse_variable_declarations: CollapseVariableDeclarations,
}

impl CollapsePass {
    pub fn new() -> Self {
        Self {
            _x0_exploit_assigns: ExploitAssigns::new(),
            x1_collapse_variable_declarations: CollapseVariableDeclarations::new(),
        }
    }
}

impl<'a> CompressorPass<'a> for CollapsePass {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for CollapsePass {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.x1_collapse_variable_declarations.enter_statements(stmts, ctx);
    }
}

// See `latePeepholeOptimizations`
pub struct LatePeepholeOptimizations {
    x0_statement_fusion: StatementFusion,
    x1_peephole_remove_dead_code: PeepholeRemoveDeadCode,
    // TODO: MinimizeExitPoints
    x2_peephole_minimize_conditions: PeepholeMinimizeConditions,
    x3_peephole_substitute_alternate_syntax: PeepholeSubstituteAlternateSyntax,
    x4_peephole_replace_known_methods: PeepholeReplaceKnownMethods,
    x5_peephole_fold_constants: PeepholeFoldConstants,
}

impl LatePeepholeOptimizations {
    pub fn new() -> Self {
        Self {
            x0_statement_fusion: StatementFusion::new(),
            x1_peephole_remove_dead_code: PeepholeRemoveDeadCode::new(),
            x2_peephole_minimize_conditions: PeepholeMinimizeConditions::new(),
            x3_peephole_substitute_alternate_syntax: PeepholeSubstituteAlternateSyntax::new(
                /* in_fixed_loop */ true,
            ),
            x4_peephole_replace_known_methods: PeepholeReplaceKnownMethods::new(),
            x5_peephole_fold_constants: PeepholeFoldConstants::new(),
        }
    }

    fn reset_changed(&mut self) {
        self.x0_statement_fusion.changed = false;
        self.x1_peephole_remove_dead_code.changed = false;
        self.x2_peephole_minimize_conditions.changed = false;
        self.x3_peephole_substitute_alternate_syntax.changed = false;
        self.x4_peephole_replace_known_methods.changed = false;
        self.x5_peephole_fold_constants.changed = false;
    }

    fn changed(&self) -> bool {
        self.x0_statement_fusion.changed
            || self.x1_peephole_remove_dead_code.changed
            || self.x2_peephole_minimize_conditions.changed
            || self.x3_peephole_substitute_alternate_syntax.changed
            || self.x4_peephole_replace_known_methods.changed
            || self.x5_peephole_fold_constants.changed
    }

    pub fn run_in_loop<'a>(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a>,
    ) {
        let mut i = 0;
        loop {
            self.reset_changed();
            self.build(program, ctx);
            if !self.changed() {
                break;
            }
            if i > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            i += 1;
        }
    }
}

impl<'a> CompressorPass<'a> for LatePeepholeOptimizations {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for LatePeepholeOptimizations {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x1_peephole_remove_dead_code.enter_statement(stmt, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x1_peephole_remove_dead_code.exit_statement(stmt, ctx);
        self.x2_peephole_minimize_conditions.exit_statement(stmt, ctx);
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_statement_fusion.exit_program(program, ctx);
        self.x1_peephole_remove_dead_code.exit_program(program, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_statement_fusion.exit_function_body(body, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.x1_peephole_remove_dead_code.exit_statements(stmts, ctx);
    }

    fn exit_block_statement(&mut self, block: &mut BlockStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_statement_fusion.exit_block_statement(block, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.exit_return_statement(stmt, ctx);
    }

    fn enter_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x3_peephole_substitute_alternate_syntax.enter_variable_declaration(decl, ctx);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.enter_expression(expr, ctx);
        self.x4_peephole_replace_known_methods.enter_expression(expr, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x1_peephole_remove_dead_code.exit_expression(expr, ctx);
        self.x2_peephole_minimize_conditions.exit_expression(expr, ctx);
        self.x3_peephole_substitute_alternate_syntax.exit_expression(expr, ctx);
        self.x5_peephole_fold_constants.exit_expression(expr, ctx);
    }

    fn enter_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.enter_call_expression(expr, ctx);
    }

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.exit_call_expression(expr, ctx);
    }

    fn enter_binary_expression(
        &mut self,
        expr: &mut BinaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x3_peephole_substitute_alternate_syntax.enter_binary_expression(expr, ctx);
    }
}

// See `createPeepholeOptimizationsPass`
pub struct PeepholeOptimizations {
    // TODO: MinimizeExitPoints
    x2_peephole_minimize_conditions: PeepholeMinimizeConditions,
    x3_peephole_substitute_alternate_syntax: PeepholeSubstituteAlternateSyntax,
    x4_peephole_replace_known_methods: PeepholeReplaceKnownMethods,
    x5_peephole_remove_dead_code: PeepholeRemoveDeadCode,
    x6_peephole_fold_constants: PeepholeFoldConstants,
}

impl PeepholeOptimizations {
    pub fn new() -> Self {
        Self {
            x2_peephole_minimize_conditions: PeepholeMinimizeConditions::new(),
            x3_peephole_substitute_alternate_syntax: PeepholeSubstituteAlternateSyntax::new(
                /* in_fixed_loop */ false,
            ),
            x4_peephole_replace_known_methods: PeepholeReplaceKnownMethods::new(),
            x5_peephole_remove_dead_code: PeepholeRemoveDeadCode::new(),
            x6_peephole_fold_constants: PeepholeFoldConstants::new(),
        }
    }
}

impl<'a> CompressorPass<'a> for PeepholeOptimizations {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeOptimizations {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x5_peephole_remove_dead_code.enter_statement(stmt, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x2_peephole_minimize_conditions.exit_statement(stmt, ctx);
        self.x5_peephole_remove_dead_code.exit_statement(stmt, ctx);
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x5_peephole_remove_dead_code.exit_program(program, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.x5_peephole_remove_dead_code.exit_statements(stmts, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.exit_return_statement(stmt, ctx);
    }

    fn enter_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x3_peephole_substitute_alternate_syntax.enter_variable_declaration(decl, ctx);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.enter_expression(expr, ctx);
        self.x4_peephole_replace_known_methods.enter_expression(expr, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x2_peephole_minimize_conditions.exit_expression(expr, ctx);
        self.x3_peephole_substitute_alternate_syntax.exit_expression(expr, ctx);
        self.x5_peephole_remove_dead_code.exit_expression(expr, ctx);
        self.x6_peephole_fold_constants.exit_expression(expr, ctx);
    }

    fn enter_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.enter_call_expression(expr, ctx);
    }

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_peephole_substitute_alternate_syntax.exit_call_expression(expr, ctx);
    }

    fn enter_binary_expression(
        &mut self,
        expr: &mut BinaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x3_peephole_substitute_alternate_syntax.enter_binary_expression(expr, ctx);
    }
}
