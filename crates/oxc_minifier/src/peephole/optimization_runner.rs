//! Optimization runner for coordinating and executing peephole optimizations.

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::ctx::Ctx;

/// Categories of optimizations that can be applied at different stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationCategory {
    /// Constant folding and evaluation optimizations.
    ConstantFolding,
    /// Condition minimization and boolean logic optimizations.
    ConditionMinimization,
    /// Dead code elimination and removal.
    DeadCodeElimination,
    /// Method call optimizations and known method replacements.
    MethodOptimization,
    /// Syntactic substitutions and alternate syntax.
    SyntaxSubstitution,
    /// Identifier inlining and reference optimization.
    IdentifierInlining,
    /// Statement-level optimizations and minimization.
    StatementOptimization,
}

/// Configuration for optimization execution.
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Categories of optimizations to enable.
    pub enabled_categories: std::vec::Vec<OptimizationCategory>,
    /// Maximum number of iterations for fixed-point optimizations.
    pub max_iterations: u8,
    /// Whether to apply optimizations conservatively (safer but less aggressive).
    pub conservative: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enabled_categories: std::vec![
                OptimizationCategory::ConstantFolding,
                OptimizationCategory::ConditionMinimization,
                OptimizationCategory::DeadCodeElimination,
                OptimizationCategory::MethodOptimization,
                OptimizationCategory::SyntaxSubstitution,
                OptimizationCategory::IdentifierInlining,
                OptimizationCategory::StatementOptimization,
            ],
            max_iterations: 10,
            conservative: false,
        }
    }
}

/// Result of applying optimizations.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Whether any changes were made.
    pub changed: bool,
    /// Number of iterations executed.
    pub iterations: u8,
    /// Categories of optimizations that were applied.
    pub applied_categories: std::vec::Vec<OptimizationCategory>,
}

/// Trait for optimization passes that can be applied to expressions.
pub trait ExpressionOptimization<'a> {
    /// Apply the optimization to an expression.
    /// Returns true if any changes were made.
    fn optimize_expression(&self, expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) -> bool;

    /// Get the category of this optimization.
    fn category(&self) -> OptimizationCategory;

    /// Check if this optimization should be applied given the configuration.
    fn should_apply(&self, config: &OptimizationConfig) -> bool {
        config.enabled_categories.contains(&self.category())
    }
}

/// Trait for optimization passes that can be applied to statements.
pub trait StatementOptimization<'a> {
    /// Apply the optimization to a statement.
    /// Returns true if any changes were made.
    fn optimize_statement(&self, stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) -> bool;

    /// Apply the optimization to a list of statements.
    /// Returns true if any changes were made.
    fn optimize_statements(
        &self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> bool;

    /// Get the category of this optimization.
    fn category(&self) -> OptimizationCategory;

    /// Check if this optimization should be applied given the configuration.
    fn should_apply(&self, config: &OptimizationConfig) -> bool {
        config.enabled_categories.contains(&self.category())
    }
}

/// Optimization runner that coordinates the execution of multiple optimization passes.
pub struct OptimizationRunner<'a> {
    config: OptimizationConfig,
    expression_optimizations: std::vec::Vec<Box<dyn ExpressionOptimization<'a> + 'a>>,
    statement_optimizations: std::vec::Vec<Box<dyn StatementOptimization<'a> + 'a>>,
}

impl<'a> OptimizationRunner<'a> {
    /// Create a new optimization runner with the given configuration.
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            expression_optimizations: std::vec::Vec::new(),
            statement_optimizations: std::vec::Vec::new(),
        }
    }

    /// Add an expression optimization to the runner.
    pub fn add_expression_optimization(
        &mut self,
        optimization: Box<dyn ExpressionOptimization<'a> + 'a>,
    ) {
        self.expression_optimizations.push(optimization);
    }

    /// Add a statement optimization to the runner.
    pub fn add_statement_optimization(
        &mut self,
        optimization: Box<dyn StatementOptimization<'a> + 'a>,
    ) {
        self.statement_optimizations.push(optimization);
    }

    /// Apply all enabled optimizations to an expression.
    pub fn optimize_expression(
        &self,
        expr: &mut Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> OptimizationResult {
        let mut result = OptimizationResult {
            changed: false,
            iterations: 0,
            applied_categories: std::vec::Vec::new(),
        };

        for optimization in &self.expression_optimizations {
            if !optimization.should_apply(&self.config) {
                continue;
            }

            let original_changed = ctx.state.changed;
            ctx.state.changed = false;

            if optimization.optimize_expression(expr, ctx) || ctx.state.changed {
                result.changed = true;
                let category = optimization.category();
                if !result.applied_categories.contains(&category) {
                    result.applied_categories.push(category);
                }
            }

            ctx.state.changed = original_changed || result.changed;
        }

        result.iterations = 1;
        result
    }

    /// Apply all enabled optimizations to a statement.
    pub fn optimize_statement(
        &self,
        stmt: &mut Statement<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> OptimizationResult {
        let mut result = OptimizationResult {
            changed: false,
            iterations: 0,
            applied_categories: std::vec::Vec::new(),
        };

        for optimization in &self.statement_optimizations {
            if !optimization.should_apply(&self.config) {
                continue;
            }

            let original_changed = ctx.state.changed;
            ctx.state.changed = false;

            if optimization.optimize_statement(stmt, ctx) || ctx.state.changed {
                result.changed = true;
                let category = optimization.category();
                if !result.applied_categories.contains(&category) {
                    result.applied_categories.push(category);
                }
            }

            ctx.state.changed = original_changed || result.changed;
        }

        result.iterations = 1;
        result
    }

    /// Apply all enabled optimizations to a list of statements.
    pub fn optimize_statements(
        &self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> OptimizationResult {
        let mut result = OptimizationResult {
            changed: false,
            iterations: 0,
            applied_categories: std::vec::Vec::new(),
        };

        for optimization in &self.statement_optimizations {
            if !optimization.should_apply(&self.config) {
                continue;
            }

            let original_changed = ctx.state.changed;
            ctx.state.changed = false;

            if optimization.optimize_statements(stmts, ctx) || ctx.state.changed {
                result.changed = true;
                let category = optimization.category();
                if !result.applied_categories.contains(&category) {
                    result.applied_categories.push(category);
                }
            }

            ctx.state.changed = original_changed || result.changed;
        }

        result.iterations = 1;
        result
    }
}

/// Helper trait for optimization passes that need both expression and statement optimization.
pub trait HybridOptimization<'a>: ExpressionOptimization<'a> + StatementOptimization<'a> {
    /// Apply optimizations to both expressions and statements in a coordinated manner.
    fn optimize_hybrid(
        &self,
        expr: &mut Expression<'a>,
        stmt: &mut Statement<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> bool {
        let expr_changed = self.optimize_expression(expr, ctx);
        let stmt_changed = self.optimize_statement(stmt, ctx);
        expr_changed || stmt_changed
    }
}
