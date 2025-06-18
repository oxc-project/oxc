#![allow(clippy::unused_self)]

mod collapse_variable_declarations;
mod convert_to_dotted_properties;
mod fold_constants;
mod minimize_conditional_expression;
mod minimize_conditions;
mod minimize_exit_points;
mod minimize_expression_in_boolean_context;
mod minimize_for_statement;
mod minimize_if_statement;
mod minimize_logical_expression;
mod minimize_not_expression;
mod minimize_statements;
mod normalize;
mod remove_dead_code;
mod remove_unused_expression;
mod replace_known_methods;
mod statement_fusion;
mod substitute_alternate_syntax;

use rustc_hash::FxHashSet;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_syntax::{es_target::ESTarget, scope::ScopeId};
use oxc_traverse::{ReusableTraverseCtx, Traverse, traverse_mut_with_ctx};

use crate::{
    ctx::{Ctx, MinifierState, TraverseCtx},
    options::CompressOptionsKeepNames,
};

pub use self::normalize::{Normalize, NormalizeOptions};

#[derive(Debug, Default, Clone, Copy)]
pub struct State {
    pub changed: bool,
}

pub struct PeepholeOptimizations {
    target: ESTarget,
    keep_names: CompressOptionsKeepNames,

    /// Walk the ast in a fixed point loop until no changes are made.
    /// `prev_function_changed`, `functions_changed` and `current_function` track changes
    /// in top level and each function. No minification code are run if the function is not changed
    /// in the previous walk.
    iteration: u8,
    prev_functions_changed: FxHashSet<ScopeId>,
    functions_changed: FxHashSet<ScopeId>,
    /// Track the current function as a stack.
    current_function:
        NonEmptyStack<(ScopeId, /* prev changed */ bool, /* current changed */ bool)>,
}

impl<'a> PeepholeOptimizations {
    pub fn new(target: ESTarget, keep_names: CompressOptionsKeepNames) -> Self {
        Self {
            target,
            keep_names,
            iteration: 0,
            prev_functions_changed: FxHashSet::default(),
            functions_changed: FxHashSet::default(),
            current_function: NonEmptyStack::new((ScopeId::new(0), true, false)),
        }
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
    ) {
        loop {
            self.build(program, ctx);
            if self.functions_changed.is_empty() {
                break;
            }
            self.prev_functions_changed.clear();
            std::mem::swap(&mut self.prev_functions_changed, &mut self.functions_changed);
            if self.iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            self.iteration += 1;
        }
    }

    fn mark_current_function_as_changed(&mut self) {
        let (_scope_id, _prev_changed, current_changed) = self.current_function.last_mut();
        *current_changed = true;
    }

    #[inline]
    fn is_prev_function_changed(&self) -> bool {
        let (_, prev_changed, _) = self.current_function.last();
        *prev_changed
    }

    fn enter_program_or_function(&mut self, scope_id: ScopeId) {
        self.current_function.push((
            scope_id,
            self.iteration == 0 || self.prev_functions_changed.contains(&scope_id),
            false,
        ));
    }

    fn exit_program_or_function(&mut self) {
        let (scope_id, _, changed) = self.current_function.pop();
        if changed {
            self.functions_changed.insert(scope_id);
        }
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
    fn enter_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.enter_program_or_function(program.scope_id());
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.exit_program_or_function();
    }

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.enter_program_or_function(func.scope_id());
    }

    fn exit_function(&mut self, _: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.exit_program_or_function();
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.minimize_statements(stmts, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.try_fold_stmt_in_boolean_context(stmt, &mut ctx);
        self.remove_dead_code_exit_statement(stmt, &mut state, &mut ctx);
        if let Statement::IfStatement(if_stmt) = stmt {
            if let Some(folded_stmt) = self.try_minimize_if(if_stmt, &mut state, &mut ctx) {
                *stmt = folded_stmt;
                self.mark_current_function_as_changed();
            }
        }
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut state = State::default();
        let mut ctx = Ctx::new(ctx);
        self.minimize_for_statement(stmt, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_return_statement(stmt, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_variable_declaration(decl, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.fold_constants_exit_expression(expr, &mut state, &mut ctx);
        self.minimize_conditions_exit_expression(expr, &mut state, &mut ctx);
        self.remove_dead_code_exit_expression(expr, &mut state, &mut ctx);
        self.replace_known_methods_exit_expression(expr, &mut state, &mut ctx);
        self.substitute_exit_expression(expr, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);

        if expr.operator.is_not()
            && self.try_fold_expr_in_boolean_context(&mut expr.argument, &mut ctx)
        {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_call_expression(expr, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_new_expression(&mut self, expr: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_new_expression(expr, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_object_property(prop, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_assignment_target_property(
        &mut self,
        node: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_assignment_target_property(node, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_assignment_target_property_property(prop, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_binding_property(prop, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_method_definition(prop, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_property_definition(prop, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let mut ctx = Ctx::new(ctx);
        let mut state = State::default();
        self.substitute_accessor_property(prop, &mut state, &mut ctx);
        if state.changed {
            self.mark_current_function_as_changed();
        }
    }
}

/// Changes that do not interfere with optimizations that are run inside the fixed-point loop,
/// which can be done as a last AST pass.
pub struct LatePeepholeOptimizations {
    target: ESTarget,
}

impl<'a> LatePeepholeOptimizations {
    pub fn new(target: ESTarget) -> Self {
        Self { target }
    }

    pub fn build(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a, MinifierState<'a>> for LatePeepholeOptimizations {
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

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        Self::substitute_exit_expression(expr, &mut ctx);
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);
        self.substitute_catch_clause(catch, &mut ctx);
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, _ctx: &mut TraverseCtx<'a>) {
        Self::remove_empty_spread_arguments(&mut e.arguments);
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, _ctx: &mut TraverseCtx<'a>) {
        Self::remove_empty_spread_arguments(&mut e.arguments);
    }
}

pub struct DeadCodeElimination {
    inner: PeepholeOptimizations,
}

impl<'a> DeadCodeElimination {
    pub fn new() -> Self {
        Self {
            inner: PeepholeOptimizations::new(
                ESTarget::ESNext,
                CompressOptionsKeepNames::all_true(),
            ),
        }
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
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut state = State::default();
        let mut ctx = Ctx::new(ctx);
        self.inner.remove_dead_code_exit_statement(stmt, &mut state, &mut ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        let mut state = State::default();
        let mut ctx = Ctx::new(ctx);
        self.inner.remove_dead_code_exit_statements(stmts, &mut state, &mut ctx);
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut state = State::default();
        let mut ctx = Ctx::new(ctx);
        self.inner.fold_constants_exit_expression(expr, &mut state, &mut ctx);
        self.inner.remove_dead_code_exit_expression(expr, &mut state, &mut ctx);
    }
}
