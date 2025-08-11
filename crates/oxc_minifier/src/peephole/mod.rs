#![allow(clippy::unused_self)]

mod convert_to_dotted_properties;
mod fold_constants;
mod inline;
mod minimize_conditional_expression;
mod minimize_conditions;
mod minimize_expression_in_boolean_context;
mod minimize_for_statement;
mod minimize_if_statement;
mod minimize_logical_expression;
mod minimize_not_expression;
mod minimize_statements;
mod normalize;
mod remove_dead_code;
mod remove_unused_declaration;
mod remove_unused_expression;
mod replace_known_methods;
mod substitute_alternate_syntax;

use oxc_ast_visit::Visit;
use oxc_semantic::ReferenceId;
use rustc_hash::FxHashSet;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{ReusableTraverseCtx, Traverse, traverse_mut_with_ctx};

use crate::{
    ctx::{Ctx, TraverseCtx},
    state::MinifierState,
};

pub use self::normalize::{Normalize, NormalizeOptions};

pub struct PeepholeOptimizations {
    /// Walk the ast in a fixed point loop until no changes are made.
    /// `prev_function_changed`, `functions_changed` and `current_function` track changes
    /// in top level and each function. No minification code are run if the function is not changed
    /// in the previous walk.
    iteration: u8,
    changed: bool,
}

impl<'a> PeepholeOptimizations {
    pub fn new() -> Self {
        Self { iteration: 0, changed: false }
    }

    pub fn build(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn run_in_loop(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) -> u8 {
        loop {
            self.changed = false;
            self.build(program, ctx);
            if !self.changed {
                break;
            }
            if self.iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            self.iteration += 1;
        }
        self.iteration
    }

    pub fn commutative_pair<'x, A, F, G, RetF: 'x, RetG: 'x>(
        pair: (&'x A, &'x A),
        check_a: F,
        check_b: G,
    ) -> Option<(RetF, RetG)>
    where
        F: Fn(&'x A) -> Option<RetF>,
        G: Fn(&'x A) -> Option<RetG>,
    {
        match check_a(pair.0) {
            Some(a) => {
                if let Some(b) = check_b(pair.1) {
                    return Some((a, b));
                }
            }
            _ => {
                if let Some(a) = check_a(pair.1) {
                    if let Some(b) = check_b(pair.0) {
                        return Some((a, b));
                    }
                }
            }
        }
        None
    }
}

impl<'a> Traverse<'a, MinifierState<'a>> for PeepholeOptimizations {
    fn enter_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.symbol_values.clear();
        ctx.state.changed = false;
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Remove unused references by visiting the AST again and diff the collected references.
        let refs_before =
            ctx.scoping().resolved_references().flatten().copied().collect::<FxHashSet<_>>();
        let mut counter = ReferencesCounter::default();
        counter.visit_program(program);
        for reference_id_to_remove in refs_before.difference(&counter.refs) {
            ctx.scoping_mut().delete_reference(*reference_id_to_remove);
        }
        self.changed = ctx.state.changed;
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.minimize_statements(stmts, &mut ctx);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = &mut Ctx::new(ctx);
        Self::keep_track_of_empty_functions(stmt, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.try_fold_stmt_in_boolean_context(stmt, &mut ctx);
        self.remove_dead_code_exit_statement(stmt, &mut ctx);
        if let Statement::IfStatement(if_stmt) = stmt {
            if let Some(folded_stmt) = self.try_minimize_if(if_stmt, &mut ctx) {
                *stmt = folded_stmt;
                ctx.state.changed = true;
            }
        }
    }

    fn exit_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.minimize_for_statement(stmt, &mut ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_return_statement(stmt, &mut ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);
        self.substitute_variable_declaration(decl, &mut ctx);
    }

    fn exit_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);
        self.init_symbol_value(decl, &mut ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.fold_constants_exit_expression(expr, &mut ctx);
        self.minimize_conditions_exit_expression(expr, &mut ctx);
        self.remove_dead_code_exit_expression(expr, &mut ctx);
        self.replace_known_methods_exit_expression(expr, &mut ctx);
        self.substitute_exit_expression(expr, &mut ctx);
        self.inline_identifier_reference(expr, &mut ctx);
    }

    fn exit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        if expr.operator.is_not()
            && self.try_fold_expr_in_boolean_context(&mut expr.argument, &mut ctx)
        {
            ctx.state.changed = true;
        }
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.substitute_call_expression(e, &mut ctx);
        Self::remove_empty_spread_arguments(&mut e.arguments);
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.substitute_new_expression(e, &mut ctx);
        Self::remove_empty_spread_arguments(&mut e.arguments);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_object_property(prop, &mut ctx);
    }

    fn exit_assignment_target_property(
        &mut self,
        node: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_assignment_target_property(node, &mut ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_assignment_target_property_property(prop, &mut ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_binding_property(prop, &mut ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_method_definition(prop, &mut ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_property_definition(prop, &mut ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_accessor_property(prop, &mut ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);
        Self::convert_to_dotted_properties(expr, &mut ctx);
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        Self::remove_dead_code_exit_class_body(body, &mut ctx);
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.substitute_catch_clause(catch, &mut ctx);
    }
}

pub struct DeadCodeElimination {
    inner: PeepholeOptimizations,
}

impl<'a> DeadCodeElimination {
    pub fn new() -> Self {
        Self { inner: PeepholeOptimizations::new() }
    }

    pub fn build(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a, MinifierState<'a>> for DeadCodeElimination {
    fn exit_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut ctx = Ctx::new(ctx);
        self.inner.init_symbol_value(decl, &mut ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.inner.remove_dead_code_exit_statement(stmt, &mut ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        let changed = ctx.state.changed;
        ctx.state.changed = false;
        self.inner.minimize_statements(stmts, &mut ctx);
        if ctx.state.changed {
            self.inner.minimize_statements(stmts, &mut ctx);
        } else {
            ctx.state.changed = changed;
        }
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.inner.fold_constants_exit_expression(expr, &mut ctx);
        self.inner.remove_dead_code_exit_expression(expr, &mut ctx);
    }
}

#[derive(Default)]
struct ReferencesCounter {
    refs: FxHashSet<ReferenceId>,
}

impl<'a> Visit<'a> for ReferencesCounter {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if let Some(reference_id) = it.reference_id.get() {
            self.refs.insert(reference_id);
        }
    }
}
