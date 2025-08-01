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
mod optimization;
mod optimization_runner;
mod optimization_utils;
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
pub use self::optimization::Optimization;
pub use self::optimization_utils::OptimizationUtils;
// These will be used once integrated into the main optimization loop:
// pub use self::optimization_runner::{
//     ExpressionOptimization, 
//     HybridOptimization, 
//     OptimizationCategory, 
//     OptimizationConfig, 
//     OptimizationResult, 
//     OptimizationRunner, 
//     StatementOptimization
// };

/// Main peephole optimization coordinator.
/// 
/// This struct orchestrates the execution of various peephole optimizations
/// in a systematic and efficient manner. It implements a fixed-point iteration
/// strategy to ensure all possible optimizations are applied.
///
/// # Optimization Strategy
///
/// The coordinator uses a fixed-point loop approach:
/// 1. Apply all enabled optimizations to the AST
/// 2. Check if any changes were made
/// 3. If changes occurred, repeat from step 1
/// 4. Stop when no changes are made or max iterations reached
///
/// This ensures that optimizations that create new optimization opportunities
/// are fully exploited (e.g., constant folding enabling dead code elimination).
///
/// # Performance Considerations
///
/// - Tracks iteration count to prevent infinite loops
/// - Only continues if changes were made in the previous iteration
/// - Uses efficient AST traversal patterns
/// - Minimizes memory allocations during optimization
///
/// # Supported Optimizations
///
/// - **Constant folding**: Evaluate compile-time constants
/// - **Dead code elimination**: Remove unreachable code
/// - **Condition minimization**: Simplify boolean expressions
/// - **Expression minimization**: Reduce expression complexity
/// - **Statement optimization**: Optimize control flow structures
pub struct PeepholeOptimizations {
    /// Current iteration count for the fixed-point loop.
    /// Used to prevent infinite optimization cycles.
    iteration: u8,
    /// Whether any changes were made in the current iteration.
    /// Controls whether to continue the fixed-point loop.
    changed: bool,
}

impl<'a> PeepholeOptimizations {
    /// Creates a new peephole optimization coordinator.
    ///
    /// Initializes the coordinator with default settings for a fresh
    /// optimization session.
    ///
    /// # Returns
    /// A new `PeepholeOptimizations` instance ready to process AST nodes.
    pub fn new() -> Self {
        Self { iteration: 0, changed: false }
    }

    /// Executes a single pass of optimizations through the AST.
    ///
    /// This method performs one complete traversal of the AST, applying
    /// all enabled optimizations. It's used internally by the fixed-point
    /// loop but can also be called directly for single-pass optimization.
    ///
    /// # Arguments
    /// * `program` - The JavaScript/TypeScript program AST to optimize
    /// * `ctx` - The traversal context with minifier state and settings
    ///
    /// # Side Effects
    /// - Modifies the AST in place when optimizations are applied
    /// - Updates the `changed` flag to indicate if any modifications occurred
    pub fn build(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    /// Runs optimizations in a fixed-point loop until convergence.
    /// 
    /// This is the main entry point for comprehensive peephole optimization.
    /// It repeatedly applies optimizations until no further changes can be made,
    /// ensuring that cascading optimizations are fully realized.
    ///
    /// # Fixed-Point Algorithm
    /// 1. Reset change tracking for the new iteration
    /// 2. Apply all optimizations via `build()`
    /// 3. If changes were made, increment iteration counter and repeat
    /// 4. Stop when no changes occur or maximum iterations reached
    ///
    /// # Arguments
    /// * `program` - The JavaScript/TypeScript program AST to optimize
    /// * `ctx` - The traversal context with minifier state and settings
    ///
    /// # Performance Notes
    /// - Uses a maximum iteration limit to prevent infinite loops
    /// - Only continues iterations when the previous pass made changes
    /// - Typical programs converge within 2-3 iterations
    ///
    /// # Examples of Cascading Optimizations
    /// - Constant folding enables dead code elimination: `if (true) { a(); }` → `a();`
    /// - Dead code removal enables further constant folding
    /// - Expression simplification creates new inlining opportunities
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

    /// Apply an optimization function and update state if changed.
    fn apply_optimization<T, F>(&self, target: &mut T, ctx: &mut Ctx<'a, '_>, optimizer: F) -> bool
    where
        F: FnOnce(&mut T, &mut Ctx<'a, '_>) -> bool,
    {
        let original_changed = ctx.state.changed;
        ctx.state.changed = false;
        
        let result = optimizer(target, ctx);
        
        if ctx.state.changed || result {
            ctx.state.changed = true;
            true
        } else {
            ctx.state.changed = original_changed;
            false
        }
    }

    /// Apply multiple optimizations to an expression in sequence.
    fn optimize_expression(&self, expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let original_changed = ctx.state.changed;
        let mut any_changes = false;
        
        // Apply optimizations and track changes
        if self.apply_optimization(expr, ctx, |e, c| {
            self.fold_constants_exit_expression(e, c);
            false // fold_constants updates ctx.state.changed internally
        }) {
            any_changes = true;
        }
        
        if self.apply_optimization(expr, ctx, |e, c| {
            self.minimize_conditions_exit_expression(e, c);
            false // minimize_conditions updates ctx.state.changed internally
        }) {
            any_changes = true;
        }
        
        if self.apply_optimization(expr, ctx, |e, c| {
            self.remove_dead_code_exit_expression(e, c);
            false // remove_dead_code updates ctx.state.changed internally
        }) {
            any_changes = true;
        }
        
        if self.apply_optimization(expr, ctx, |e, c| {
            self.replace_known_methods_exit_expression(e, c);
            false // replace_known_methods updates ctx.state.changed internally
        }) {
            any_changes = true;
        }
        
        if self.apply_optimization(expr, ctx, |e, c| {
            self.substitute_exit_expression(e, c);
            false // substitute updates ctx.state.changed internally
        }) {
            any_changes = true;
        }
        
        if self.apply_optimization(expr, ctx, |e, c| {
            self.inline_identifier_reference(e, c);
            false // inline updates ctx.state.changed internally
        }) {
            any_changes = true;
        }
        
        // Restore original changed state or mark as changed if we made changes
        ctx.state.changed = original_changed || any_changes;
    }

    /// Utility function for handling commutative binary operations.
    /// 
    /// This function tries to apply two checker functions to a pair of values
    /// in both orders (commutative), returning the first successful match.
    /// This is useful for optimizations that can work with operands in either order.
    pub fn commutative_pair<'x, A, F, G, RetF: 'x, RetG: 'x>(
        pair: (&'x A, &'x A),
        check_a: F,
        check_b: G,
    ) -> Option<(RetF, RetG)>
    where
        F: Fn(&'x A) -> Option<RetF>,
        G: Fn(&'x A) -> Option<RetG>,
    {
        // Try first order: check_a(pair.0) && check_b(pair.1)
        if let Some(a) = check_a(pair.0) {
            if let Some(b) = check_b(pair.1) {
                return Some((a, b));
            }
        }
        
        // Try second order: check_a(pair.1) && check_b(pair.0)
        if let Some(a) = check_a(pair.1) {
            if let Some(b) = check_b(pair.0) {
                return Some((a, b));
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
        self.optimize_expression(expr, &mut ctx);
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

/// Late peephole optimizations that don't interfere with the fixed-point loop.
/// 
/// These optimizations are applied as a final pass after the main optimization loop
/// has completed. They include transformations that:
/// - Don't benefit from iterative application
/// - Are best applied after all other optimizations are complete
/// - May conflict with optimizations in the main loop
pub struct LatePeepholeOptimizations;

impl<'a> LatePeepholeOptimizations {
    /// Create a new late peephole optimization coordinator.
    pub fn new() -> Self {
        Self
    }

    /// Apply late optimizations to the program.
    /// 
    /// This runs a single pass of optimizations that should be applied
    /// after the main optimization loop has converged.
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

/// Specialized dead code elimination optimization.
/// 
/// This optimization focuses specifically on removing dead code and unused expressions.
/// It uses a subset of the main peephole optimizations but with different iteration
/// and application strategies optimized for dead code elimination.
pub struct DeadCodeElimination {
    inner: PeepholeOptimizations,
}

impl<'a> DeadCodeElimination {
    /// Create a new dead code elimination optimizer.
    pub fn new() -> Self {
        Self { inner: PeepholeOptimizations::new() }
    }

    /// Apply dead code elimination to the program.
    /// 
    /// This applies a focused set of optimizations specifically designed
    /// to eliminate dead code and unused expressions.
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

/// Reference counter for tracking identifier references during optimization.
/// 
/// This visitor counts all identifier references in the AST, which is used
/// to update the semantic information after optimizations have potentially
/// removed references.
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
