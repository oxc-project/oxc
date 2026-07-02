//! ES2020: Nullish Coalescing Operator
//!
//! This plugin transforms nullish coalescing operators (`??`) to a series of ternary expressions.
//!
//! > This plugin is included in `preset-env`, in ES2020
//!
//! ## Example
//!
//! Input:
//! ```js
//! var foo = object.foo ?? "default";
//! ```
//!
//! Output:
//! ```js
//! var _object$foo;
//! var foo =
//! (_object$foo = object.foo) !== null && _object$foo !== void 0
//!   ? _object$foo
//!   : "default";
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-nullish-coalescing-operator](https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/v7.26.2/packages/babel-plugin-transform-nullish-coalescing-operator>
//! * Nullish coalescing TC39 proposal: <https://github.com/tc39-transfer/proposal-nullish-coalescing>

use oxc_allocator::{ArenaBox, ArenaVec, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_semantic::{ScopeFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};

use crate::{context::TraverseCtx, state::TransformState};

pub struct NullishCoalescingOperator;

impl NullishCoalescingOperator {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for NullishCoalescingOperator {
    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // left ?? right
        if matches!(expr, Expression::LogicalExpression(logical_expr) if logical_expr.operator == LogicalOperator::Coalesce)
        {
            Self::transform_logical_expression(expr, ctx);
        }
    }
}

impl<'a> NullishCoalescingOperator {
    #[cold] // Most `Expression`s are not `??` `LogicalExpression`s
    fn transform_logical_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Take ownership of the `LogicalExpression`
        let Expression::LogicalExpression(logical_expr) = expr.take_in(ctx) else { unreachable!() };

        *expr = Self::transform_logical_expression_impl(logical_expr, ctx);
    }

    fn transform_logical_expression_impl(
        logical_expr: ArenaBox<'a, LogicalExpression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let logical_expr = logical_expr.unbox();

        // Skip creating extra reference when `left` is static
        match &logical_expr.left {
            Expression::ThisExpression(this) => {
                let this_span = this.span;
                return Self::create_conditional_expression(
                    logical_expr.left,
                    Expression::new_this_expression(this_span, ctx),
                    Expression::new_this_expression(this_span, ctx),
                    logical_expr.right,
                    logical_expr.span,
                    ctx,
                );
            }
            Expression::Identifier(ident) => {
                let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id();
                if let Some(symbol_id) = symbol_id {
                    // Check binding is not mutated.
                    // TODO(improve-on-babel): Remove this check. Whether binding is mutated or not is not relevant.
                    if ctx.scoping().get_resolved_references(symbol_id).all(|r| !r.is_write()) {
                        let binding = BoundIdentifier::new(ident.name, symbol_id);
                        let ident_span = ident.span;
                        return Self::create_conditional_expression(
                            logical_expr.left,
                            binding.create_spanned_read_expression(ident_span, ctx),
                            binding.create_spanned_read_expression(ident_span, ctx),
                            logical_expr.right,
                            logical_expr.span,
                            ctx,
                        );
                    }
                }
            }
            _ => {}
        }

        // After the binding pattern refactor, initializers are directly on FormalParameter
        // So ctx.ancestor(0) is FormalParameterInitializer when the nullish coalescing
        // is directly in a parameter's default value
        let is_parent_formal_parameter =
            matches!(ctx.ancestor(0), Ancestor::FormalParameterInitializer(_));

        let current_scope_id = if is_parent_formal_parameter {
            ctx.create_child_scope_of_current(ScopeFlags::Arrow | ScopeFlags::Function)
        } else {
            ctx.current_hoist_scope_id()
        };

        // Add `var _name` to scope
        let binding = ctx.generate_uid_based_on_node(
            &logical_expr.left,
            current_scope_id,
            SymbolFlags::FunctionScopedVariable,
        );

        let assignment = Expression::new_assignment_expression(
            SPAN,
            AssignmentOperator::Assign,
            binding.create_write_target(ctx),
            logical_expr.left,
            ctx,
        );
        let mut new_expr = Self::create_conditional_expression(
            assignment,
            binding.create_read_expression(ctx),
            binding.create_read_expression(ctx),
            logical_expr.right,
            logical_expr.span,
            ctx,
        );

        if is_parent_formal_parameter {
            // Replace `function (a, x = a.b ?? c) {}` to `function (a, x = (() => a.b ?? c)() ){}`
            // so the temporary variable can be injected in correct scope
            let id = binding.create_binding_pattern(ctx);
            let param = FormalParameter::new(
                SPAN,
                ArenaVec::new_in(ctx),
                id,
                NONE,
                NONE,
                false,
                None,
                false,
                false,
                ctx,
            );
            let params = FormalParameters::new(
                SPAN,
                FormalParameterKind::ArrowFormalParameters,
                ArenaVec::from_value_in(param, ctx),
                NONE,
                ctx,
            );
            let body = FunctionBody::new(
                SPAN,
                ArenaVec::new_in(ctx),
                ArenaVec::from_value_in(
                    Statement::new_expression_statement(SPAN, new_expr, ctx),
                    ctx,
                ),
                ctx,
            );
            let arrow_function =
                Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
                    SPAN,
                    true,
                    false,
                    NONE,
                    params,
                    NONE,
                    body,
                    current_scope_id,
                    false,
                    false,
                    ctx,
                );
            // `(x) => x;` -> `((x) => x)();`
            new_expr = Expression::new_call_expression(
                SPAN,
                arrow_function,
                NONE,
                ArenaVec::new_in(ctx),
                false,
                ctx,
            );
        } else {
            ctx.state.var_declarations.insert_var(&binding, &ctx.ast);
        }

        new_expr
    }

    /// Create a conditional expression.
    ///
    /// ```js
    /// // Input
    /// foo = bar ?? "qux"
    ///
    /// // Output
    /// foo = bar !== null && bar !== void 0 ? bar : "qux"
    /// //    ^^^ assignment  ^^^ reference1         ^^^^^ default
    /// //                                     ^^^ reference2
    /// ```
    ///
    /// ```js
    /// // Input
    /// foo = bar.x ?? "qux"
    ///
    /// // Output
    /// foo = (_bar$x = bar.x) !== null && _bar$x !== void 0 ? _bar$x : "qux"
    /// //    ^^^^^^^^^^^^^^^^ assignment  ^^^^^^ reference1            ^^^^^ default
    /// //                                                     ^^^^^^ reference2
    /// ```
    fn create_conditional_expression(
        assignment: Expression<'a>,
        reference1: Expression<'a>,
        reference2: Expression<'a>,
        default: Expression<'a>,
        span: Span,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let op = BinaryOperator::StrictInequality;
        let null = Expression::new_null_literal(SPAN, ctx);
        let left = Expression::new_binary_expression(SPAN, assignment, op, null, ctx);
        let right = Expression::new_binary_expression(
            SPAN,
            reference1,
            op,
            Expression::new_void_0(SPAN, ctx),
            ctx,
        );
        let test = Expression::new_logical_expression(SPAN, left, LogicalOperator::And, right, ctx);

        Expression::new_conditional_expression(span, test, reference2, default, ctx)
    }
}
