mod collapse_variable_declarations;
mod convert_to_dotted_properties;
mod fold_constants;
mod minimize_conditional_expression;
mod minimize_conditions;
mod minimize_exit_points;
mod minimize_expression_in_boolean_context;
mod minimize_if_statement;
mod minimize_not_expression;
mod minimize_statements;
mod normalize;
mod remove_dead_code;
mod replace_known_methods;
mod statement_fusion;
mod substitute_alternate_syntax;

use rustc_hash::FxHashSet;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_syntax::{es_target::ESTarget, scope::ScopeId};
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::ctx::Ctx;

pub use self::normalize::{Normalize, NormalizeOptions};

pub struct PeepholeOptimizations {
    target: ESTarget,

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
    pub fn new(target: ESTarget) -> Self {
        Self {
            target,
            iteration: 0,
            prev_functions_changed: FxHashSet::default(),
            functions_changed: FxHashSet::default(),
            current_function: NonEmptyStack::new((ScopeId::new(0), true, false)),
        }
    }

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn run_in_loop(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
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
        if let Some(a) = check_a(pair.0) {
            if let Some(b) = check_b(pair.1) {
                return Some((a, b));
            }
        } else if let Some(a) = check_a(pair.1) {
            if let Some(b) = check_b(pair.0) {
                return Some((a, b));
            }
        }
        None
    }
}

impl<'a> Traverse<'a> for PeepholeOptimizations {
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
        let ctx = Ctx(ctx);
        self.minimize_statements(stmts, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, traverse_ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        Self::try_fold_stmt_in_boolean_context(stmt, Ctx(traverse_ctx));
        self.remove_dead_code_exit_statement(stmt, Ctx(traverse_ctx));
        if let Statement::IfStatement(if_stmt) = stmt {
            if let Some(folded_stmt) = self.try_minimize_if(if_stmt, traverse_ctx) {
                *stmt = folded_stmt;
                self.mark_current_function_as_changed();
            }
        }
        self.substitute_exit_statement(stmt, Ctx(traverse_ctx));
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_return_statement(stmt, ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_variable_declaration(decl, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.fold_constants_exit_expression(expr, ctx);
        self.minimize_conditions_exit_expression(expr, ctx);
        self.remove_dead_code_exit_expression(expr, ctx);
        self.replace_known_methods_exit_expression(expr, ctx);
        self.substitute_exit_expression(expr, ctx);
    }

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_call_expression(expr, ctx);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_object_property(prop, ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_assignment_target_property_property(prop, ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_binding_property(prop, ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_method_definition(prop, ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_property_definition(prop, ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.is_prev_function_changed() {
            return;
        }
        let ctx = Ctx(ctx);
        self.substitute_accessor_property(prop, ctx);
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

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for LatePeepholeOptimizations {
    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::convert_to_dotted_properties(expr, Ctx(ctx));
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::remove_dead_code_exit_class_body(body, Ctx(ctx));
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::substitute_exit_expression(expr, Ctx(ctx));
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        self.substitute_catch_clause(catch, Ctx(ctx));
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
        Self { inner: PeepholeOptimizations::new(ESTarget::ESNext) }
    }

    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for DeadCodeElimination {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.inner.remove_dead_code_exit_statement(stmt, Ctx(ctx));
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.inner.remove_dead_code_exit_statements(stmts, Ctx(ctx));
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.inner.fold_constants_exit_expression(expr, Ctx(ctx));
        self.inner.remove_dead_code_exit_expression(expr, Ctx(ctx));
    }
}
