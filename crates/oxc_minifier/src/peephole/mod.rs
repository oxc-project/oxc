//! Peephole optimizations for JavaScript minification.
//!
//! This module contains various local optimizations that examine small sections of code
//! (typically a few AST nodes) and replace them with more efficient equivalents.
//!
//! ## Optimization Categories
//!
//! ### Expression Optimizations
//! - [`fold_constants`]: Evaluates constant expressions at compile time
//! - [`minimize_conditions`]: Simplifies boolean expressions and conditions
//! - [`minimize_logical_expression`]: Optimizes logical AND/OR operations
//! - [`minimize_not_expression`]: Reduces negation operations
//!
//! ### Statement Optimizations  
//! - [`minimize_if_statement`]: Simplifies conditional statements
//! - [`minimize_for_statement`]: Optimizes loop structures
//! - [`remove_dead_code`]: Eliminates unreachable code
//! - [`remove_unused_declaration`]: Removes unused variable declarations
//!
//! ### Property Access Optimizations
//! - [`convert_to_dotted_properties`]: Converts bracket notation to dot notation when safe
//! - [`replace_known_methods`]: Replaces known method calls with shorter equivalents
//!
//! ### Cleanup Optimizations
//! - [`remove_unused_expression`]: Removes expressions without side effects
//! - [`substitute_alternate_syntax`]: Uses shorter syntax alternatives
//!
//! ## Fixed-Point Iteration
//!
//! Most optimizations are applied in a fixed-point loop using [`PeepholeOptimizations`],
//! which continues applying optimizations until no more changes are made.

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

/// Coordinates peephole optimizations in a fixed-point iteration.
///
/// This struct manages the application of various peephole optimizations,
/// continuing to apply them until no more changes are made to the AST.
/// This ensures that all possible optimizations are applied, even when
/// one optimization enables another.
///
/// ## Fixed-Point Algorithm
///
/// The optimizer runs in a loop where:
/// 1. All enabled optimizations are applied to the AST
/// 2. If any optimization made changes, repeat from step 1
/// 3. If no changes were made, optimization is complete
/// 4. A maximum iteration limit prevents infinite loops
pub struct PeepholeOptimizations {
    /// Walk the ast in a fixed point loop until no changes are made.
    /// `prev_function_changed`, `functions_changed` and `current_function` track changes
    /// in top level and each function. No minification code are run if the function is not changed
    /// in the previous walk.
    iteration: u8,
    changed: bool,
}

impl<'a> PeepholeOptimizations {
    /// Creates a new peephole optimization coordinator.
    pub fn new() -> Self {
        Self { iteration: 0, changed: false }
    }

    /// Applies optimizations once to the program.
    ///
    /// This performs a single pass of all peephole optimizations.
    /// Use [`run_in_loop`] for fixed-point iteration.
    ///
    /// [`run_in_loop`]: Self::run_in_loop
    pub fn build(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    /// Applies optimizations repeatedly until no more changes are made.
    ///
    /// This is the main entry point for peephole optimization. It runs
    /// optimizations in a loop until the AST reaches a fixed point where
    /// no more optimizations can be applied.
    ///
    /// # Arguments
    ///
    /// * `program` - The JavaScript AST to optimize (modified in-place)
    /// * `ctx` - Traversal context containing semantic information
    pub fn run_in_loop(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
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

    fn exit_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_call_expression(expr, &mut ctx);
    }

    fn exit_new_expression(&mut self, expr: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut ctx = Ctx::new(ctx);

        self.substitute_new_expression(expr, &mut ctx);
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
}

/// Changes that do not interfere with optimizations that are run inside the fixed-point loop,
/// which can be done as a last AST pass.
pub struct LatePeepholeOptimizations;

impl<'a> LatePeepholeOptimizations {
    pub fn new() -> Self {
        Self
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
