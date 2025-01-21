use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_syntax::es_target::ESTarget;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

mod collapse_variable_declarations;
mod convert_to_dotted_properties;
mod minimize_exit_points;
mod normalize;
mod peephole_fold_constants;
mod peephole_minimize_conditions;
mod peephole_remove_dead_code;
mod peephole_replace_known_methods;
mod peephole_substitute_alternate_syntax;
mod statement_fusion;

pub use normalize::{Normalize, NormalizeOptions};

pub struct PeepholeOptimizations {
    target: ESTarget,
    changed: bool,
    /// `in_fixed_loop`: Do not compress syntaxes that are hard to analyze inside the fixed loop.
    /// Opposite of `late` in Closure Compiler.
    in_fixed_loop: bool,
}

impl<'a> PeepholeOptimizations {
    pub fn new(target: ESTarget, in_fixed_loop: bool) -> Self {
        Self { target, changed: false, in_fixed_loop }
    }

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn run_in_loop(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        let mut i = 0;
        loop {
            self.changed = false;
            self.build(program, ctx);
            if !self.changed {
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

impl<'a> Traverse<'a> for PeepholeOptimizations {
    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.statement_fusion_exit_statements(stmts, ctx);
        self.collapse_variable_declarations(stmts, ctx);
        self.minimize_conditions_exit_statements(stmts, ctx);
        self.remove_dead_code_exit_statements(stmts, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.minimize_conditions_exit_statement(stmt, ctx);
        self.remove_dead_code_exit_statement(stmt, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.substitute_return_statement(stmt, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.minimize_exit_points(body, ctx);
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.remove_dead_code_exit_class_body(body, ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.substitute_variable_declaration(decl, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.fold_constants_exit_expression(expr, ctx);
        self.minimize_conditions_exit_expression(expr, ctx);
        self.remove_dead_code_exit_expression(expr, ctx);
        self.replace_known_methods_exit_expression(expr, ctx);
        self.substitute_exit_expression(expr, ctx);
    }

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.substitute_call_expression(expr, ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.convert_to_dotted_properties(expr, ctx);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        self.substitute_object_property(prop, ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.substitute_assignment_target_property_property(prop, ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        self.substitute_binding_property(prop, ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.substitute_method_definition(prop, ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.substitute_property_definition(prop, ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.substitute_accessor_property(prop, ctx);
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        self.substitute_catch_clause(catch, ctx);
    }
}

pub struct DeadCodeElimination {
    inner: PeepholeOptimizations,
}

impl<'a> DeadCodeElimination {
    pub fn new() -> Self {
        Self { inner: PeepholeOptimizations::new(ESTarget::ESNext, false) }
    }

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for DeadCodeElimination {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.inner.remove_dead_code_exit_statement(stmt, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.inner.remove_dead_code_exit_statements(stmts, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.inner.fold_constants_exit_expression(expr, ctx);
        self.inner.remove_dead_code_exit_expression(expr, ctx);
    }
}
