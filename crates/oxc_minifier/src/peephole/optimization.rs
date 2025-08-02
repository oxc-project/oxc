//! Common optimization traits and utilities for peephole optimizations.

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::ctx::Ctx;

/// Common interface for optimization implementations.
pub trait Optimization<'a> {
    /// Apply this optimization to an expression, returning a new expression if changed.
    fn optimize_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let _ = (expr, ctx);
        false
    }

    /// Apply this optimization to a statement, returning true if changed.
    fn optimize_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) -> bool {
        let _ = (stmt, ctx);
        false
    }

    /// Apply this optimization to a list of statements, returning true if changed.
    fn optimize_statements(
        &mut self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> bool {
        let _ = (stmts, ctx);
        false
    }
}

/// Utility functions for optimization implementations.
pub struct OptimizationUtils;

impl OptimizationUtils {
    /// Mark context as changed and return true for convenience.
    pub fn mark_changed(ctx: &mut Ctx<'_, '_>) -> bool {
        ctx.state.changed = true;
        true
    }

    /// Check if an expression has side effects according to the context.
    pub fn has_side_effects<'a>(expr: &Expression<'a>, ctx: &Ctx<'a, '_>) -> bool {
        use oxc_ecmascript::side_effects::MayHaveSideEffects;
        expr.may_have_side_effects(ctx)
    }

    /// Check if a statement is empty (EmptyStatement).
    pub fn is_empty_statement(stmt: &Statement<'_>) -> bool {
        matches!(stmt, Statement::EmptyStatement(_))
    }
}

/// Context wrapper that provides optimization-specific utilities.
pub struct OptimizationContext<'a, 'b> {
    pub ctx: &'b mut Ctx<'a, 'b>,
    changes_made: bool,
}

impl<'a, 'b> OptimizationContext<'a, 'b> {
    pub fn new(ctx: &'b mut Ctx<'a, 'b>) -> Self {
        Self { ctx, changes_made: false }
    }

    /// Mark the context as changed.
    pub fn mark_changed(&mut self) {
        self.ctx.state.changed = true;
        self.changes_made = true;
    }

    /// Check if changes were made in this optimization context.
    pub fn has_changes(&self) -> bool {
        self.changes_made
    }

    /// Check if the underlying context has been marked as changed.
    pub fn has_changed(&self) -> bool {
        self.ctx.state.changed
    }

    /// Apply an optimization function that returns an optional replacement.
    pub fn try_optimize<T, F>(&mut self, target: &mut T, optimizer: F) -> bool
    where
        F: FnOnce(&mut T, &mut Ctx<'a, '_>) -> Option<T>,
    {
        if let Some(replacement) = optimizer(target, self.ctx) {
            *target = replacement;
            self.mark_changed();
            true
        } else {
            false
        }
    }

    /// Apply an optimization function that may modify the target in place.
    pub fn try_optimize_in_place<T, F>(&mut self, target: &mut T, optimizer: F) -> bool
    where
        F: FnOnce(&mut T, &mut Ctx<'a, '_>) -> bool,
    {
        let previous_changed = self.ctx.state.changed;
        self.ctx.state.changed = false;

        let result = optimizer(target, self.ctx);

        if self.ctx.state.changed {
            self.changes_made = true;
            self.ctx.state.changed = previous_changed || true;
            true
        } else {
            self.ctx.state.changed = previous_changed;
            result
        }
    }
}
