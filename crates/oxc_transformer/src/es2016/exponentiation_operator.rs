//! ES2016: Exponentiation Operator
//!
//! This plugin transforms the exponentiation operator (`**`) to `Math.pow`.
//!
//! > This plugin is included in `preset-env`, in ES2016
//!
//! ## Example
//!
//! Input:
//! ```js
//! let x = 10 ** 2;
//!
//! x **= 3;
//! ```
//!
//! Output:
//! ```js
//! let x = Math.pow(10, 2);
//!
//! x = Math.pow(x, 3);
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-exponentiation-operator](https://babel.dev/docs/babel-plugin-transform-exponentiation-operator).
//!
//! ## References:
//!
//! * Babel plugin implementation:
//!   <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-exponentiation-operator>
//!   <https://github.com/babel/babel/tree/main/packages/babel-helper-builder-binary-assignment-operator-visitor>
//! * Exponentiation operator TC39 proposal: <https://github.com/tc39/proposal-exponentiation-operator>
//! * Exponentiation operator specification: <https://tc39.es/ecma262/#sec-exp-operator>

use oxc_allocator::{CloneIn, Vec};
use oxc_ast::{ast::*, NONE};
use oxc_semantic::{ReferenceFlags, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};
use oxc_traverse::{ast_operations::get_var_name_from_node, Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct ExponentiationOperator<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

#[derive(Debug)]
struct Exploded<'a> {
    reference: AssignmentTarget<'a>,
    uid: Expression<'a>,
}

impl<'a, 'ctx> ExponentiationOperator<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for ExponentiationOperator<'a, 'ctx> {
    // Note: Do not transform to `Math.pow` with BigInt arguments - that's a runtime error
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            // `left ** right`
            Expression::BinaryExpression(binary_expr) => {
                if binary_expr.operator != BinaryOperator::Exponential
                    || binary_expr.left.is_big_int_literal()
                    || binary_expr.right.is_big_int_literal()
                {
                    return;
                }

                Self::convert_binary_expression(expr, ctx);
            }
            // `left **= right`
            Expression::AssignmentExpression(assign_expr) => {
                if assign_expr.operator != AssignmentOperator::Exponential
                    || assign_expr.right.is_big_int_literal()
                {
                    return;
                }

                match &assign_expr.left {
                    AssignmentTarget::AssignmentTargetIdentifier(_) => {
                        self.convert_assignment_to_identifier(expr, ctx);
                    }
                    // Note: We do not match `AssignmentTarget::PrivateFieldExpression` here.
                    // From Babel: "We can't generate property ref for private name, please install
                    // `@babel/plugin-transform-class-properties`".
                    // TODO: Ensure this plugin interacts correctly with class private properties
                    // transform, so the property is transformed before this transform.
                    AssignmentTarget::StaticMemberExpression(_)
                    | AssignmentTarget::ComputedMemberExpression(_) => {
                        self.convert_assignment_to_member_expression(expr, ctx);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl<'a, 'ctx> ExponentiationOperator<'a, 'ctx> {
    /// Convert `BinaryExpression`.
    /// `left ** right` -> `Math.pow(left, right)`
    fn convert_binary_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let binary_expr = match ctx.ast.move_expression(expr) {
            Expression::BinaryExpression(binary_expr) => binary_expr.unbox(),
            _ => unreachable!(),
        };
        *expr = Self::math_pow(binary_expr.left, binary_expr.right, ctx);
    }

    /// Convert `AssignmentExpression` where assignee is an identifier.
    ///
    /// `left **= right` transformed to:
    /// * If `left` is a bound symbol:
    ///   -> `left = Math.pow(left, right)`
    /// * If `left` is unbound:
    ///   -> `var _left; _left = left, left = Math.pow(_left, right);`
    ///
    /// Temporary variable `_left` is to avoid side-effects of getting `left` from running twice.
    fn convert_assignment_to_identifier(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let assign_target = &mut assign_expr.left;
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = assign_target else {
            unreachable!()
        };

        let mut nodes = ctx.ast.vec();

        let symbol_id = ctx.symbols().get_reference(ident.reference_id().unwrap()).symbol_id();
        // Make sure side-effects of evaluating `left` only happen once
        let uid = if let Some(symbol_id) = symbol_id {
            // This variable is declared in scope so evaluating it multiple times can't trigger a getter.
            // No need for a temp var.
            ctx.ast.expression_from_identifier_reference(ctx.create_bound_reference_id(
                SPAN,
                ident.name.clone(),
                symbol_id,
                ReferenceFlags::Write,
            ))
        } else {
            // Unbound reference. Could possibly trigger a getter so we need to only evaluate it once.
            // Assign to a temp var.
            let reference = ctx.ast.expression_from_identifier_reference(
                ctx.create_unbound_reference_id(SPAN, ident.name.clone(), ReferenceFlags::Read),
            );
            let name = ident.name.as_str();
            self.add_new_reference(reference, name, &mut nodes, ctx)
        };

        let reference = ctx.ast.move_assignment_target(assign_target);

        *expr = Self::create_replacement(assign_expr, reference, uid, nodes, ctx);
    }

    /// Convert `AssignmentExpression` where assignee is a member expression.
    ///
    /// `obj.prop **= right`
    /// * If `obj` is a bound symbol:
    ///   -> `obj["prop"] = Math.pow(obj["prop"], right)`
    /// * If `obj` is unbound:
    ///   -> `var _obj; _obj = obj, _obj["prop"] = Math.pow(_obj["prop"], right)`
    ///
    /// `obj[name] **= right`
    /// * If `obj` is a bound symbol:
    ///   -> `var _name; _name = name, obj[_name] = Math.pow(obj[_name], 2)`
    /// * If `obj` is unbound:
    ///   -> `var _obj, _name; _obj = obj, _name = name, _obj[_name] = Math.pow(_obj[_name], 2)`
    ///
    /// Temporary variables are to avoid side-effects of getting `obj` or `name` being run twice.
    ///
    /// TODO(improve-on-babel):
    /// 1. If `name` is bound, it doesn't need a temp variable `_name`.
    /// 2. `obj.prop` does not need to be transformed to `obj["prop"]`.
    /// We currently aim to produce output that exactly matches Babel, but we can improve this in future
    /// when we no longer need to match exactly.
    fn convert_assignment_to_member_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };

        let mut nodes = ctx.ast.vec();
        let Exploded { reference, uid } =
            self.explode_member_expression(&mut assign_expr.left, &mut nodes, ctx);

        *expr = Self::create_replacement(assign_expr, reference, uid, nodes, ctx);
    }

    fn create_replacement(
        assign_expr: &mut AssignmentExpression<'a>,
        reference: AssignmentTarget<'a>,
        uid: Expression<'a>,
        mut nodes: Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let right = ctx.ast.move_expression(&mut assign_expr.right);
        let right = Self::math_pow(uid, right, ctx);
        let assign_expr =
            ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, reference, right);
        nodes.push(assign_expr);

        ctx.ast.expression_sequence(SPAN, nodes)
    }

    fn explode_member_expression(
        &mut self,
        node: &mut AssignmentTarget<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Exploded<'a> {
        let member_expr = node.to_member_expression_mut();

        // Make sure side-effects of evaluating `obj` of `obj.ref` and `obj[ref]` only happen once
        let obj = match member_expr {
            MemberExpression::ComputedMemberExpression(e) => &mut e.object,
            MemberExpression::StaticMemberExpression(e) => &mut e.object,
            // This possibility is ruled out in `enter_expression`
            MemberExpression::PrivateFieldExpression(_) => unreachable!(),
        };
        let mut obj = ctx.ast.move_expression(obj);
        // If the object reference that we need to save is locally declared, evaluating it multiple times
        // will not trigger getters or setters. `super` cannot be directly assigned, so use it directly too.
        // TODO(improve-on-babel): We could also skip creating a temp var for `this.x **= 2`.
        let needs_temp_var = match &obj {
            Expression::Super(_) => false,
            Expression::Identifier(ident) => {
                !ctx.symbols().has_binding(ident.reference_id().unwrap())
            }
            _ => true,
        };
        if needs_temp_var {
            let name = get_var_name_from_node(&obj);
            obj = self.add_new_reference(obj, &name, nodes, ctx);
        }

        let computed = member_expr.is_computed();
        let prop = self.get_prop_ref(member_expr, nodes, ctx);
        let optional = false;
        let obj_clone = Self::clone_expression(&obj, ctx);
        let (reference, uid) = match &prop {
            Expression::Identifier(ident) if !computed => {
                let ident = IdentifierName::new(SPAN, ident.name.clone());
                (
                    // TODO:
                    // Both of these are the same, but it's in order to avoid after cloning without reference_id.
                    // Related: https://github.com/oxc-project/oxc/issues/4804
                    ctx.ast.member_expression_static(SPAN, obj_clone, ident.clone(), optional),
                    ctx.ast.member_expression_static(SPAN, obj, ident, optional),
                )
            }
            _ => {
                let prop_clone = Self::clone_expression(&prop, ctx);
                (
                    ctx.ast.member_expression_computed(SPAN, obj_clone, prop_clone, optional),
                    ctx.ast.member_expression_computed(SPAN, obj, prop, optional),
                )
            }
        };
        Exploded {
            reference: AssignmentTarget::from(
                ctx.ast.simple_assignment_target_member_expression(reference),
            ),
            uid: Expression::from(uid),
        }
    }

    fn clone_expression(expr: &Expression<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match expr {
            Expression::Identifier(ident) => ctx.ast.expression_from_identifier_reference(
                ctx.clone_identifier_reference(ident, ReferenceFlags::Read),
            ),
            _ => expr.clone_in(ctx.ast.allocator),
        }
    }

    /// `Math.pow(left, right)`
    fn math_pow(
        left: Expression<'a>,
        right: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let math_symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), "Math");
        let ident_math =
            ctx.create_reference_id(SPAN, Atom::from("Math"), math_symbol_id, ReferenceFlags::Read);
        let object = ctx.ast.expression_from_identifier_reference(ident_math);
        let property = ctx.ast.identifier_name(SPAN, "pow");
        let callee =
            Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false));
        let mut arguments = ctx.ast.vec_with_capacity(2);
        arguments.push(Argument::from(left));
        arguments.push(Argument::from(right));
        ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
    }

    /// Make sure side-effects of evaluating `ref` of `obj.ref` and `obj[ref]` only happen once.
    fn get_prop_ref(
        &mut self,
        node: &mut MemberExpression<'a>,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match node {
            MemberExpression::ComputedMemberExpression(expr) => {
                let expr = ctx.ast.move_expression(&mut expr.expression);
                if expr.is_literal() {
                    return expr;
                }
                let name = get_var_name_from_node(&expr);
                self.add_new_reference(expr, &name, nodes, ctx)
            }
            MemberExpression::StaticMemberExpression(expr) => {
                ctx.ast.expression_string_literal(SPAN, expr.property.name.clone())
            }
            // This possibility is ruled out in `enter_expression`
            MemberExpression::PrivateFieldExpression(_) => unreachable!(),
        }
    }

    fn add_new_reference(
        &mut self,
        expr: Expression<'a>,
        name: &str,
        nodes: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let binding = ctx.generate_uid_in_current_scope(name, SymbolFlags::FunctionScopedVariable);

        // var _name;
        self.ctx.var_declarations.insert(&binding, None, ctx);

        // Add new reference `_name = name` to nodes
        let left = ctx.ast.simple_assignment_target_from_identifier_reference(
            binding.create_write_reference(ctx),
        );
        let op = AssignmentOperator::Assign;
        nodes.push(ctx.ast.expression_assignment(SPAN, op, AssignmentTarget::from(left), expr));

        ctx.ast.expression_from_identifier_reference(binding.create_read_reference(ctx))
    }
}
