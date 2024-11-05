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
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-nullish-coalescing-operator>
//! * Nullish coalescing TC39 proposal: <https://github.com/tc39-transfer/proposal-nullish-coalescing>

use oxc_allocator::CloneIn;
use oxc_ast::{ast::*, NONE};
use oxc_semantic::{ReferenceFlags, ScopeFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct NullishCoalescingOperator<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> NullishCoalescingOperator<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for NullishCoalescingOperator<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // left ?? right
        if !matches!(expr, Expression::LogicalExpression(logical_expr) if logical_expr.operator == LogicalOperator::Coalesce)
        {
            return;
        }

        // Take ownership of the `LogicalExpression`
        let logical_expr = match ctx.ast.move_expression(expr) {
            Expression::LogicalExpression(logical_expr) => logical_expr.unbox(),
            _ => unreachable!(),
        };

        // skip creating extra reference when `left` is static
        if ctx.is_static(&logical_expr.left) {
            *expr = Self::create_conditional_expression(
                Self::clone_expression(&logical_expr.left, ctx),
                logical_expr.left,
                logical_expr.right,
                ctx,
            );
            return;
        }

        // ctx.ancestor(0) is AssignmentPattern
        // ctx.ancestor(1) is BindingPattern
        // ctx.ancestor(2) is FormalParameter
        let is_parent_formal_parameter =
            matches!(ctx.ancestor(2), Ancestor::FormalParameterPattern(_));

        let current_scope_id = if is_parent_formal_parameter {
            ctx.create_child_scope_of_current(ScopeFlags::Arrow | ScopeFlags::Function)
        } else {
            ctx.current_scope_id()
        };

        // Add `var _name` to scope
        let binding = ctx.generate_uid_based_on_node(
            &logical_expr.left,
            current_scope_id,
            SymbolFlags::FunctionScopedVariable,
        );

        let reference = binding.create_read_expression(ctx);
        let assignment = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            binding.create_read_write_target(ctx),
            logical_expr.left,
        );
        let mut new_expr =
            Self::create_conditional_expression(reference, assignment, logical_expr.right, ctx);

        if is_parent_formal_parameter {
            // Replace `function (a, x = a.b ?? c) {}` to `function (a, x = (() => a.b ?? c)() ){}`
            // so the temporary variable can be injected in correct scope
            let id = binding.create_binding_pattern(ctx);
            let param = ctx.ast.formal_parameter(SPAN, ctx.ast.vec(), id, None, false, false);
            let params = ctx.ast.formal_parameters(
                SPAN,
                FormalParameterKind::ArrowFormalParameters,
                ctx.ast.vec1(param),
                NONE,
            );
            let body = ctx.ast.function_body(
                SPAN,
                ctx.ast.vec(),
                ctx.ast.vec1(ctx.ast.statement_expression(SPAN, new_expr)),
            );
            let arrow_function = Expression::ArrowFunctionExpression(
                ctx.ast.alloc_arrow_function_expression_with_scope_id(
                    SPAN,
                    true,
                    false,
                    NONE,
                    params,
                    NONE,
                    body,
                    current_scope_id,
                ),
            );
            // `(x) => x;` -> `((x) => x)();`
            new_expr = ctx.ast.expression_call(SPAN, arrow_function, NONE, ctx.ast.vec(), false);
        } else {
            self.ctx.var_declarations.insert_var(&binding, None, ctx);
        }

        *expr = new_expr;
    }
}

impl<'a, 'ctx> NullishCoalescingOperator<'a, 'ctx> {
    fn clone_expression(expr: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => Expression::Identifier(
                ctx.ast.alloc(ctx.clone_identifier_reference(ident, ReferenceFlags::Read)),
            ),
            _ => expr.clone_in(ctx.ast.allocator),
        }
    }

    /// Create a conditional expression
    ///
    /// ```js
    /// // Input
    /// bar ?? "qux"
    ///
    /// // Output
    /// qux = bar !== null && bar !== void 0 ? bar : "qux"
    /// //    ^^^ assignment  ^^^ reference           ^^^ default
    /// ```
    ///
    /// reference and assignment are the same in this case, but they can be different
    fn create_conditional_expression(
        reference: Expression<'a>,
        assignment: Expression<'a>,
        default: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let op = BinaryOperator::StrictInequality;
        let null = ctx.ast.expression_null_literal(SPAN);
        let left = ctx.ast.expression_binary(SPAN, assignment, op, null);
        let right = ctx.ast.expression_binary(
            SPAN,
            Self::clone_expression(&reference, ctx),
            op,
            ctx.ast.void_0(SPAN),
        );
        let test = ctx.ast.expression_logical(SPAN, left, LogicalOperator::And, right);

        ctx.ast.expression_conditional(SPAN, test, reference, default)
    }
}
