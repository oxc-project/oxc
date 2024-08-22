//! ES2021: Logical Assignment Operators
//!
//! This plugin transform logical assignment operators `&&=`, `||=`, and `??=` to a series of logical expressions.
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

use std::cell::Cell;

use oxc_allocator::{CloneIn, Vec};
use oxc_ast::ast::*;
use oxc_semantic::{ReferenceFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::Ctx;

pub struct LogicalAssignmentOperators<'a> {
    _ctx: Ctx<'a>,
    var_declarations: std::vec::Vec<Vec<'a, VariableDeclarator<'a>>>,
}

impl<'a> LogicalAssignmentOperators<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { _ctx: ctx, var_declarations: vec![] }
    }
}

impl<'a> Traverse<'a> for LogicalAssignmentOperators<'a> {
    fn enter_statements(&mut self, _stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.var_declarations.push(ctx.ast.vec());
    }

    fn exit_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(declarations) = self.var_declarations.pop() {
            if declarations.is_empty() {
                return;
            }
            let variable = ctx.ast.alloc_variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                declarations,
                false,
            );
            statements.insert(0, Statement::VariableDeclaration(variable));
        }
    }

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

        let left_expr: Expression<'a>;
        let assign_target: AssignmentTarget;

        // TODO: refactor this block, add tests, cover private identifier
        match &mut assignment_expr.left {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                left_expr = ctx.ast.expression_from_identifier_reference(
                    ctx.clone_identifier_reference(ident, ReferenceFlags::Read),
                );
                assign_target = AssignmentTarget::from(
                    ctx.ast.simple_assignment_target_from_identifier_reference(
                        ctx.clone_identifier_reference(ident, ReferenceFlags::Write),
                    ),
                );
            }
            left @ match_member_expression!(AssignmentTarget) => {
                let member_expr = left.to_member_expression_mut();
                let op = AssignmentOperator::Assign;

                // `a.b &&= c` -> `var _a; (_a = a).b && (_a.b = c)`
                match member_expr {
                    MemberExpression::StaticMemberExpression(static_expr) => {
                        if let Some(ident) = self.maybe_generate_memoised(&static_expr.object, ctx)
                        {
                            // (_o = o).a
                            let right = ctx.ast.move_expression(&mut static_expr.object);
                            let target = AssignmentTarget::from(
                                ctx.ast.simple_assignment_target_from_identifier_reference(
                                    ctx.clone_identifier_reference(&ident, ReferenceFlags::Write),
                                ),
                            );
                            let object = ctx.ast.expression_assignment(SPAN, op, target, right);
                            left_expr = Expression::from(ctx.ast.member_expression_static(
                                SPAN,
                                object,
                                static_expr.property.clone_in(ctx.ast.allocator),
                                false,
                            ));

                            // (_o.a = 1)
                            let assign_expr = ctx.ast.member_expression_static(
                                SPAN,
                                ctx.ast.expression_from_identifier_reference(ident),
                                static_expr.property.clone_in(ctx.ast.allocator),
                                false,
                            );
                            assign_target = AssignmentTarget::from(
                                ctx.ast.simple_assignment_target_member_expression(assign_expr),
                            );
                        } else {
                            // transform `obj.x ||= 1` to `obj.x || (obj.x = 1)`
                            let object = ctx.ast.move_expression(&mut static_expr.object);

                            // TODO: We should use static_expr.clone_in instead of cloning the properties,
                            // but currently clone_in will get rid of IdentifierReference's reference_id
                            let static_expr_cloned = ctx.ast.static_member_expression(
                                static_expr.span,
                                Self::clone_expression(&object, ctx),
                                static_expr.property.clone_in(ctx.ast.allocator),
                                static_expr.optional,
                            );

                            left_expr = ctx.ast.expression_member(
                                ctx.ast.member_expression_from_static(static_expr_cloned),
                            );

                            let member_expr_moved = ctx.ast.member_expression_static(
                                static_expr.span,
                                object,
                                static_expr.property.clone_in(ctx.ast.allocator),
                                static_expr.optional,
                            );

                            assign_target = AssignmentTarget::from(
                                ctx.ast
                                    .simple_assignment_target_member_expression(member_expr_moved),
                            );
                        };
                    }
                    // `a[b.y] &&= c;` ->
                    // `var _a, _b$y; (_a = a)[_b$y = b.y] && (_a[_b$y] = c);`
                    MemberExpression::ComputedMemberExpression(computed_expr) => {
                        if let Some(ident) =
                            self.maybe_generate_memoised(&computed_expr.object, ctx)
                        {
                            // (_o = object)
                            let right = ctx.ast.move_expression(&mut computed_expr.object);
                            let target = AssignmentTarget::from(
                                ctx.ast.simple_assignment_target_from_identifier_reference(
                                    ctx.clone_identifier_reference(&ident, ReferenceFlags::Write),
                                ),
                            );
                            let object = ctx.ast.expression_assignment(SPAN, op, target, right);

                            let mut expression =
                                ctx.ast.move_expression(&mut computed_expr.expression);

                            // _b = expression
                            let property = self.maybe_generate_memoised(&expression, ctx);

                            if let Some(ref property) = property {
                                let left = AssignmentTarget::from(
                                    ctx.ast.simple_assignment_target_from_identifier_reference(
                                        ctx.clone_identifier_reference(
                                            property,
                                            ReferenceFlags::Write,
                                        ),
                                    ),
                                );
                                expression =
                                    ctx.ast.expression_assignment(SPAN, op, left, expression);
                            }

                            // _o[_b]
                            assign_target =
                                AssignmentTarget::from(ctx.ast.member_expression_computed(
                                    SPAN,
                                    ctx.ast.expression_from_identifier_reference(
                                        ctx.clone_identifier_reference(
                                            &ident,
                                            ReferenceFlags::Read,
                                        ),
                                    ),
                                    property.map_or_else(
                                        || expression.clone_in(ctx.ast.allocator),
                                        |ident| ctx.ast.expression_from_identifier_reference(ident),
                                    ),
                                    false,
                                ));

                            left_expr = Expression::from(
                                ctx.ast.member_expression_computed(SPAN, object, expression, false),
                            );
                        } else {
                            // transform `obj[++key] ||= 1` to `obj[_key = ++key] || (obj[_key] = 1)`
                            let property_ident =
                                self.maybe_generate_memoised(&computed_expr.expression, ctx);

                            let object = ctx.ast.move_expression(&mut computed_expr.object);
                            let mut expression =
                                ctx.ast.move_expression(&mut computed_expr.expression);

                            // TODO: ideally we should use computed_expr.clone_in instead of cloning the properties,
                            // but currently clone_in will get rid of IdentifierReference's reference_id
                            let new_compute_expr = ctx.ast.computed_member_expression(
                                computed_expr.span,
                                Self::clone_expression(&object, ctx),
                                {
                                    // _key = ++key
                                    if let Some(property_ident) = &property_ident {
                                        let left = AssignmentTarget::from(
                                            ctx.ast
                                                .simple_assignment_target_from_identifier_reference(
                                                    ctx.clone_identifier_reference(
                                                        property_ident,
                                                        ReferenceFlags::Write,
                                                    ),
                                                ),
                                        );
                                        ctx.ast.expression_assignment(
                                            SPAN,
                                            op,
                                            left,
                                            ctx.ast.move_expression(&mut expression),
                                        )
                                    } else {
                                        Self::clone_expression(&expression, ctx)
                                    }
                                },
                                computed_expr.optional,
                            );

                            left_expr = ctx.ast.expression_member(
                                ctx.ast.member_expression_from_computed(new_compute_expr),
                            );

                            // obj[_key] = 1
                            let new_compute_expr = ctx.ast.computed_member_expression(
                                computed_expr.span,
                                object,
                                {
                                    if let Some(property_ident) = property_ident {
                                        ctx.ast.expression_from_identifier_reference(property_ident)
                                    } else {
                                        expression
                                    }
                                },
                                computed_expr.optional,
                            );

                            assign_target = AssignmentTarget::from(
                                ctx.ast.simple_assignment_target_member_expression(
                                    ctx.ast.member_expression_from_computed(new_compute_expr),
                                ),
                            );
                        };
                    }
                    MemberExpression::PrivateFieldExpression(_) => return,
                }
            }
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

impl<'a> LogicalAssignmentOperators<'a> {
    /// Clone an expression
    ///
    /// If it is an identifier, clone the identifier by [TraverseCtx::clone_identifier_reference], otherwise, use [CloneIn].
    ///
    /// TODO: remove this until <https://github.com/oxc-project/oxc/issues/4804> is resolved.
    pub fn clone_expression(expr: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => ctx.ast.expression_from_identifier_reference(
                ctx.clone_identifier_reference(ident, ReferenceFlags::Read),
            ),
            _ => expr.clone_in(ctx.ast.allocator),
        }
    }

    pub fn maybe_generate_memoised(
        &mut self,
        expr: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<IdentifierReference<'a>> {
        if ctx.symbols().is_static(expr) {
            return None;
        }

        let symbol_id = ctx
            .generate_uid_in_current_scope_based_on_node(expr, SymbolFlags::FunctionScopedVariable);
        let symbol_name = ctx.ast.atom(ctx.symbols().get_name(symbol_id));

        // var _name;
        let binding_identifier = BindingIdentifier {
            span: SPAN,
            name: symbol_name.clone(),
            symbol_id: Cell::new(Some(symbol_id)),
        };
        let kind = VariableDeclarationKind::Var;
        let id = ctx.ast.binding_pattern_kind_from_binding_identifier(binding_identifier);
        let id = ctx.ast.binding_pattern(id, None::<TSTypeAnnotation>, false);
        self.var_declarations
            .last_mut()
            .unwrap()
            .push(ctx.ast.variable_declarator(SPAN, kind, id, None, false));

        // _name = name
        Some(ctx.create_reference_id(SPAN, symbol_name, Some(symbol_id), ReferenceFlags::Write))
    }
}
