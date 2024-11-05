//! ES2021: Logical Assignment Operators
//!
//! This plugin transforms logical assignment operators (`&&=`, `||=`, `??=`)
//! to a series of logical expressions.
//!
//! > This plugin is included in `preset-env`, in ES2021
//!
//! ## Example
//!
//! Input:
//! ```js
//! a ||= b;
//! obj.a.b ||= c;
//!
//! a &&= b;
//! obj.a.b &&= c;
//! ```
//!
//! Output:
//! ```js
//! var _obj$a, _obj$a2;
//!
//! a || (a = b);
//! (_obj$a = obj.a).b || (_obj$a.b = c);
//!
//! a && (a = b);
//! (_obj$a2 = obj.a).b && (_obj$a2.b = c);
//! ```
//!
//! ### With Nullish Coalescing
//!
//! > While using the [nullish-coalescing-operator](https://github.com/oxc-project/oxc/blob/main/crates/oxc_transformer/src/es2020/nullish_coalescing_operator.rs) plugin (included in `preset-env``)
//!
//! Input:
//! ```js
//! a ??= b;
//! obj.a.b ??= c;
//! ```
//!
//! Output:
//! ```js
//! var _a, _obj$a, _obj$a$b;
//!
//! (_a = a) !== null && _a !== void 0 ? _a : (a = b);
//! (_obj$a$b = (_obj$a = obj.a).b) !== null && _obj$a$b !== void 0
//! ? _obj$a$b
//! : (_obj$a.b = c);
//! ```
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-logical-assignment-operators](https://babel.dev/docs/babel-plugin-transform-logical-assignment-operators).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-logical-assignment-operators>
//! * Logical Assignment TC39 proposal: <https://github.com/tc39/proposal-logical-assignment>

use oxc_allocator::CloneIn;
use oxc_ast::ast::*;
use oxc_semantic::{ReferenceFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct LogicalAssignmentOperators<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> LogicalAssignmentOperators<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for LogicalAssignmentOperators<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::AssignmentExpression(assignment_expr) = expr else { return };

        // `&&=` `||=` `??=`
        let operator = match assignment_expr.operator {
            AssignmentOperator::LogicalAnd => LogicalOperator::And,
            AssignmentOperator::LogicalOr => LogicalOperator::Or,
            AssignmentOperator::LogicalNullish => LogicalOperator::Coalesce,
            _ => return,
        };

        // `a &&= c` -> `a && (a = c);`
        //               ^     ^ assign_target
        //               ^ left_expr

        // TODO: Add tests, cover private identifier
        let (left_expr, assign_target) = match &mut assignment_expr.left {
            // `a &&= c` -> `a && (a = c)`
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                Self::convert_identifier(ident, ctx)
            }
            // `a.b &&= c` -> `var _a; (_a = a).b && (_a.b = c)`
            AssignmentTarget::StaticMemberExpression(static_expr) => {
                self.convert_static_member_expression(static_expr, ctx)
            }
            // `a[b.y] &&= c;` ->
            // `var _a, _b$y; (_a = a)[_b$y = b.y] && (_a[_b$y] = c);`
            AssignmentTarget::ComputedMemberExpression(computed_expr) => {
                self.convert_computed_member_expression(computed_expr, ctx)
            }
            // TODO
            #[allow(clippy::match_same_arms)]
            AssignmentTarget::PrivateFieldExpression(_) => return,
            // All other are TypeScript syntax.

            // It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.
            // So safe to return here.
            _ => return,
        };

        let assign_op = AssignmentOperator::Assign;
        let right = ctx.ast.move_expression(&mut assignment_expr.right);
        let right = ctx.ast.expression_assignment(SPAN, assign_op, assign_target, right);

        let logical_expr = ctx.ast.expression_logical(SPAN, left_expr, operator, right);

        *expr = logical_expr;
    }
}

impl<'a, 'ctx> LogicalAssignmentOperators<'a, 'ctx> {
    fn convert_identifier(
        ident: &IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, AssignmentTarget<'a>) {
        let reference = ctx.symbols_mut().get_reference_mut(ident.reference_id());
        *reference.flags_mut() = ReferenceFlags::Read;
        let symbol_id = reference.symbol_id();
        let left_expr = Expression::Identifier(ctx.alloc(ident.clone()));

        let ident = ctx.create_reference_id(
            SPAN,
            ident.name.clone(),
            symbol_id,
            ReferenceFlags::read_write(),
        );
        let assign_target = AssignmentTarget::AssignmentTargetIdentifier(ctx.alloc(ident));
        (left_expr, assign_target)
    }

    fn convert_static_member_expression(
        &mut self,
        static_expr: &mut StaticMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, AssignmentTarget<'a>) {
        if let Some(ident) = self.maybe_generate_memoised(&static_expr.object, ctx) {
            // (_o = o).a
            let right = ctx.ast.move_expression(&mut static_expr.object);
            let target = ident.create_read_write_target(ctx);
            let object =
                ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, target, right);
            let left_expr = Expression::from(ctx.ast.member_expression_static(
                SPAN,
                object,
                static_expr.property.clone_in(ctx.ast.allocator),
                false,
            ));

            // (_o.a = 1)
            let assign_expr = ctx.ast.member_expression_static(
                SPAN,
                ident.create_read_expression(ctx),
                static_expr.property.clone_in(ctx.ast.allocator),
                false,
            );
            let assign_target = AssignmentTarget::from(assign_expr);

            (left_expr, assign_target)
        } else {
            // transform `obj.x ||= 1` to `obj.x || (obj.x = 1)`
            let object = ctx.ast.move_expression(&mut static_expr.object);

            // TODO: We should use static_expr.clone_in instead of cloning the properties,
            // but currently clone_in will get rid of IdentifierReference's reference_id
            let static_expr_cloned = ctx.ast.alloc_static_member_expression(
                static_expr.span,
                Self::clone_expression(&object, ctx),
                static_expr.property.clone_in(ctx.ast.allocator),
                static_expr.optional,
            );
            let left_expr = Expression::StaticMemberExpression(static_expr_cloned);

            let member_expr_moved = ctx.ast.member_expression_static(
                static_expr.span,
                object,
                static_expr.property.clone_in(ctx.ast.allocator),
                static_expr.optional,
            );

            let assign_target = AssignmentTarget::from(member_expr_moved);

            (left_expr, assign_target)
        }
    }

    fn convert_computed_member_expression(
        &mut self,
        computed_expr: &mut ComputedMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Expression<'a>, AssignmentTarget<'a>) {
        if let Some(ident) = self.maybe_generate_memoised(&computed_expr.object, ctx) {
            // (_o = object)
            let right = ctx.ast.move_expression(&mut computed_expr.object);
            let target = ident.create_read_write_target(ctx);
            let object =
                ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, target, right);

            let mut expression = ctx.ast.move_expression(&mut computed_expr.expression);

            // _b = expression
            let property = self.maybe_generate_memoised(&expression, ctx);

            if let Some(property) = &property {
                expression = ctx.ast.expression_assignment(
                    SPAN,
                    AssignmentOperator::Assign,
                    property.create_read_write_target(ctx),
                    expression,
                );
            }

            // _o[_b]
            let assign_target = AssignmentTarget::from(ctx.ast.member_expression_computed(
                SPAN,
                ident.create_read_expression(ctx),
                property.map_or_else(
                    || expression.clone_in(ctx.ast.allocator),
                    |ident| ident.create_read_expression(ctx),
                ),
                false,
            ));

            let left_expr = Expression::from(
                ctx.ast.member_expression_computed(SPAN, object, expression, false),
            );

            (left_expr, assign_target)
        } else {
            // transform `obj[++key] ||= 1` to `obj[_key = ++key] || (obj[_key] = 1)`
            let property_ident = self.maybe_generate_memoised(&computed_expr.expression, ctx);

            let object = ctx.ast.move_expression(&mut computed_expr.object);
            let mut expression = ctx.ast.move_expression(&mut computed_expr.expression);

            // TODO: ideally we should use computed_expr.clone_in instead of cloning the properties,
            // but currently clone_in will get rid of IdentifierReference's reference_id
            let new_compute_expr = ctx.ast.alloc_computed_member_expression(
                computed_expr.span,
                Self::clone_expression(&object, ctx),
                {
                    // _key = ++key
                    if let Some(property_ident) = &property_ident {
                        ctx.ast.expression_assignment(
                            SPAN,
                            AssignmentOperator::Assign,
                            property_ident.create_read_write_target(ctx),
                            ctx.ast.move_expression(&mut expression),
                        )
                    } else {
                        Self::clone_expression(&expression, ctx)
                    }
                },
                computed_expr.optional,
            );

            let left_expr = Expression::ComputedMemberExpression(new_compute_expr);

            // obj[_key] = 1
            let new_compute_expr = ctx.ast.alloc_computed_member_expression(
                computed_expr.span,
                object,
                {
                    if let Some(property_ident) = property_ident {
                        property_ident.create_read_expression(ctx)
                    } else {
                        expression
                    }
                },
                computed_expr.optional,
            );

            let assign_target = AssignmentTarget::ComputedMemberExpression(new_compute_expr);

            (left_expr, assign_target)
        }
    }

    /// Clone an expression
    ///
    /// If it is an identifier, clone the identifier by [TraverseCtx::clone_identifier_reference], otherwise, use [CloneIn].
    ///
    /// TODO: remove this once <https://github.com/oxc-project/oxc/issues/4804> is resolved.
    fn clone_expression(expr: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => Expression::Identifier(
                ctx.ast.alloc(ctx.clone_identifier_reference(ident, ReferenceFlags::Read)),
            ),
            _ => expr.clone_in(ctx.ast.allocator),
        }
    }

    fn maybe_generate_memoised(
        &mut self,
        expr: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<BoundIdentifier<'a>> {
        if ctx.is_static(expr) {
            return None;
        }

        // var _name;
        let binding = ctx
            .generate_uid_in_current_scope_based_on_node(expr, SymbolFlags::FunctionScopedVariable);
        self.ctx.var_declarations.insert_var(&binding, None, ctx);

        Some(binding)
    }
}
