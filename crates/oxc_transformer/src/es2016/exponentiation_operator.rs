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
//! x **= 3;
//! obj.prop **= 4;
//! ```
//!
//! Output:
//! ```js
//! let x = Math.pow(10, 2);
//! x = Math.pow(x, 3);
//! obj["prop"] = Math.pow(obj["prop"], 4);
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
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct ExponentiationOperator<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
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
                        self.convert_identifier_assignment(expr, ctx);
                    }
                    AssignmentTarget::StaticMemberExpression(_) => {
                        self.convert_static_member_expression_assignment(expr, ctx);
                    }
                    AssignmentTarget::ComputedMemberExpression(_) => {
                        self.convert_computed_member_expression_assignment(expr, ctx);
                    }
                    // Babel refuses to transform this: "We can't generate property ref for private name,
                    // please install `@babel/plugin-transform-class-properties`".
                    // But there's no reason not to.
                    AssignmentTarget::PrivateFieldExpression(_) => {
                        self.convert_private_field_assignment(expr, ctx);
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
    ///
    /// `left ** right` -> `Math.pow(left, right)`
    //
    // `#[inline]` so compiler knows `expr` is a `BinaryExpression`
    #[inline]
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
    ///   -> `var _left; _left = left, left = Math.pow(_left, right)`
    ///
    /// Temporary variable `_left` is to avoid side-effects of getting `left` from running twice.
    //
    // `#[inline]` so compiler knows `expr` is an `AssignmentExpression` with `IdentifierReference` on left
    #[inline]
    fn convert_identifier_assignment(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &mut assign_expr.left else {
            unreachable!()
        };

        let (pow_left, temp_var_inits) = self.get_pow_left_identifier(ident, ctx);
        Self::convert_assignment(assign_expr, pow_left, ctx);
        Self::revise_expression(expr, temp_var_inits, ctx);
    }

    /// Get left side of `Math.pow(pow_left, ...)` for identifier
    fn get_pow_left_identifier(
        &mut self,
        ident: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (
        // Left side of `Math.pow(pow_left, ...)`
        Expression<'a>,
        // Temporary var initializations
        Vec<'a, Expression<'a>>,
    ) {
        let mut temp_var_inits = ctx.ast.vec();

        // Make sure side-effects of evaluating `left` only happen once
        let reference = ctx.scoping.symbols_mut().get_reference_mut(ident.reference_id().unwrap());
        let pow_left = if let Some(symbol_id) = reference.symbol_id() {
            // This variable is declared in scope so evaluating it multiple times can't trigger a getter.
            // No need for a temp var.
            // `left **= right` is being transformed to `left = Math.pow(left, right)`,
            // so if `left` is no longer being read from, update its `ReferenceFlags`.
            if matches!(ctx.ancestry.parent(), Ancestor::ExpressionStatementExpression(_)) {
                *reference.flags_mut() = ReferenceFlags::Write;
            }

            ctx.ast.expression_from_identifier_reference(ctx.create_bound_reference_id(
                SPAN,
                ident.name.clone(),
                symbol_id,
                ReferenceFlags::Read,
            ))
        } else {
            // Unbound reference. Could possibly trigger a getter so we need to only evaluate it once.
            // Assign to a temp var.
            let reference = ctx.ast.expression_from_identifier_reference(
                ctx.create_unbound_reference_id(SPAN, ident.name.clone(), ReferenceFlags::Read),
            );
            let binding = self.create_temp_var(reference, &mut temp_var_inits, ctx);
            binding.create_read_expression(ctx)
        };

        (pow_left, temp_var_inits)
    }

    /// Convert `AssignmentExpression` where assignee is a static member expression.
    ///
    /// `obj.prop **= right` transformed to:
    /// * If `obj` is a bound symbol:
    ///   -> `obj["prop"] = Math.pow(obj["prop"], right)`
    /// * If `obj` is unbound:
    ///   -> `var _obj; _obj = obj, _obj["prop"] = Math.pow(_obj["prop"], right)`
    ///
    /// `obj.foo.bar.qux **= right` transformed to:
    /// ```js
    /// var _obj$foo$bar;
    /// _obj$foo$bar = obj.foo.bar, _obj$foo$bar["qux"] = Math.pow(_obj$foo$bar["qux"], right)
    /// ```
    ///
    /// Temporary variables are to avoid side-effects of getting `obj` / `obj.foo.bar` being run twice.
    ///
    /// TODO(improve-on-babel): `obj.prop` does not need to be transformed to `obj["prop"]`.
    //
    // `#[inline]` so compiler knows `expr` is an `AssignmentExpression` with `StaticMemberExpression` on left
    #[inline]
    fn convert_static_member_expression_assignment(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let AssignmentTarget::StaticMemberExpression(member_expr) = &mut assign_expr.left else {
            unreachable!()
        };

        let (replacement_left, pow_left, temp_var_inits) =
            self.get_pow_left_static_member(member_expr, ctx);
        assign_expr.left = replacement_left;
        Self::convert_assignment(assign_expr, pow_left, ctx);
        Self::revise_expression(expr, temp_var_inits, ctx);
    }

    /// Get left side of `Math.pow(pow_left, ...)` for static member expression
    /// and replacement for left side of assignment.
    fn get_pow_left_static_member(
        &mut self,
        member_expr: &mut StaticMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (
        // Replacement left of assignment
        AssignmentTarget<'a>,
        // Left side of `Math.pow(pow_left, ...)`
        Expression<'a>,
        // Temporary var initializations
        Vec<'a, Expression<'a>>,
    ) {
        // Object part of 2nd member expression
        // ```
        // obj["prop"] = Math.pow(obj["prop"], right)
        //                        ^^^
        // ```
        let mut temp_var_inits = ctx.ast.vec();
        let obj = self.get_second_member_expression_object(
            &mut member_expr.object,
            &mut temp_var_inits,
            ctx,
        );

        // Property part of 2nd member expression
        // ```
        // obj["prop"] = Math.pow(obj["prop"], right)
        //                            ^^^^^^
        // ```
        let prop_span = member_expr.property.span;
        let prop_name = member_expr.property.name.clone();
        let prop = ctx.ast.expression_string_literal(prop_span, prop_name.clone());

        // Complete 2nd member expression
        // ```
        // obj["prop"] = Math.pow(obj["prop"], right)
        //                        ^^^^^^^^^^^
        // ```
        let pow_left = Expression::from(ctx.ast.member_expression_computed(SPAN, obj, prop, false));

        // Replacement for original member expression
        // ```
        // obj["prop"] = Math.pow(obj["prop"], right)
        // ^^^^^^^^^^^
        // ```
        let replacement_left =
            AssignmentTarget::ComputedMemberExpression(ctx.ast.alloc_computed_member_expression(
                member_expr.span,
                ctx.ast.move_expression(&mut member_expr.object),
                ctx.ast.expression_string_literal(prop_span, prop_name),
                false,
            ));

        (replacement_left, pow_left, temp_var_inits)
    }

    /// Convert `AssignmentExpression` where assignee is a computed member expression.
    ///
    /// `obj[prop] **= right` transformed to:
    /// * If `obj` is a bound symbol:
    ///   -> `var _prop; _prop = prop, obj[_prop] = Math.pow(obj[_prop], 2)`
    /// * If `obj` is unbound:
    ///   -> `var _obj, _prop; _obj = obj, _prop = prop, _obj[_prop] = Math.pow(_obj[_prop], 2)`
    ///
    /// `obj.foo.bar[qux] **= right` transformed to:
    /// ```js
    /// var _obj$foo$bar, _qux;
    /// _obj$foo$bar = obj.foo.bar, _qux = qux, _obj$foo$bar[_qux] = Math.pow(_obj$foo$bar[_qux], right)
    /// ```
    ///
    /// Temporary variables are to avoid side-effects of getting `obj` / `obj.foo.bar` or `prop` being run twice.
    ///
    /// TODO(improve-on-babel):
    /// 1. If `prop` is bound, it doesn't need a temp variable `_prop`.
    /// 2. Temp var initializations could be inlined:
    ///    * Current: `(_obj = obj, _prop = prop, _obj[_prop] = Math.pow(_obj[_prop], 2))`
    ///    * Could be: `(_obj = obj)[_prop = prop] = Math.pow(_obj[_prop], 2)`
    //
    // `#[inline]` so compiler knows `expr` is an `AssignmentExpression` with `ComputedMemberExpression` on left
    #[inline]
    fn convert_computed_member_expression_assignment(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let AssignmentTarget::ComputedMemberExpression(member_expr) = &mut assign_expr.left else {
            unreachable!()
        };

        let (pow_left, temp_var_inits) = self.get_pow_left_computed_member(member_expr, ctx);
        Self::convert_assignment(assign_expr, pow_left, ctx);
        Self::revise_expression(expr, temp_var_inits, ctx);
    }

    /// Get left side of `Math.pow(pow_left, ...)` for computed member expression
    fn get_pow_left_computed_member(
        &mut self,
        member_expr: &mut ComputedMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (
        // Left side of `Math.pow(pow_left, ...)`
        Expression<'a>,
        // Temporary var initializations
        Vec<'a, Expression<'a>>,
    ) {
        // Object part of 2nd member expression
        // ```
        // obj[_prop] = Math.pow(obj[_prop], right)
        //                       ^^^
        // ```
        let mut temp_var_inits = ctx.ast.vec();
        let obj = self.get_second_member_expression_object(
            &mut member_expr.object,
            &mut temp_var_inits,
            ctx,
        );

        // Property part of 2nd member expression
        // ```
        // obj[_prop] = Math.pow(obj[_prop], right)
        //     ^^^^^ replaced        ^^^^^ prop
        // ```
        let prop = &mut member_expr.expression;
        let prop = if prop.is_literal() {
            prop.clone_in(ctx.ast.allocator)
        } else {
            let owned_prop = ctx.ast.move_expression(prop);
            let binding = self.create_temp_var(owned_prop, &mut temp_var_inits, ctx);
            *prop = binding.create_read_expression(ctx);
            binding.create_read_expression(ctx)
        };

        // Complete 2nd member expression
        // ```
        // obj[_prop] = Math.pow(obj[_prop], right)
        //                       ^^^^^^^^^^
        // ```
        let pow_left = Expression::from(ctx.ast.member_expression_computed(SPAN, obj, prop, false));

        (pow_left, temp_var_inits)
    }

    /// Convert `AssignmentExpression` where assignee is a private field member expression.
    ///
    /// `obj.#prop **= right` transformed to:
    /// * If `obj` is a bound symbol:
    ///   -> `obj.#prop = Math.pow(obj.#prop, right)`
    /// * If `obj` is unbound:
    ///   -> `var _obj; _obj = obj, _obj.#prop = Math.pow(_obj.#prop, right)`
    ///
    /// `obj.foo.bar.#qux **= right` transformed to:
    /// ```js
    /// var _obj$foo$bar;
    /// _obj$foo$bar = obj.foo.bar, _obj$foo$bar.#qux = Math.pow(_obj$foo$bar.#qux, right)
    /// ```
    ///
    /// Temporary variable is to avoid side-effects of getting `obj` / `obj.foo.bar` being run twice.
    //
    // `#[inline]` so compiler knows `expr` is an `AssignmentExpression` with `PrivateFieldExpression` on left
    #[inline]
    fn convert_private_field_assignment(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::AssignmentExpression(assign_expr) = expr else { unreachable!() };
        let AssignmentTarget::PrivateFieldExpression(member_expr) = &mut assign_expr.left else {
            unreachable!()
        };

        let (pow_left, temp_var_inits) = self.get_pow_left_private_field(member_expr, ctx);
        Self::convert_assignment(assign_expr, pow_left, ctx);
        Self::revise_expression(expr, temp_var_inits, ctx);
    }

    /// Get left side of `Math.pow(pow_left, ...)` for static member expression
    /// and replacement for left side of assignment.
    fn get_pow_left_private_field(
        &mut self,
        field_expr: &mut PrivateFieldExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (
        // Left side of `Math.pow(pow_left, ...)`
        Expression<'a>,
        // Temporary var initializations
        Vec<'a, Expression<'a>>,
    ) {
        // Object part of 2nd member expression
        // ```
        // obj.#prop = Math.pow(obj.#prop, right)
        //                      ^^^
        // ```
        let mut temp_var_inits = ctx.ast.vec();
        let obj = self.get_second_member_expression_object(
            &mut field_expr.object,
            &mut temp_var_inits,
            ctx,
        );

        // Property part of 2nd member expression
        // ```
        // obj.#prop = Math.pow(obj.#prop, right)
        //                          ^^^^^
        // ```
        let field = field_expr.field.clone_in(ctx.ast.allocator);

        // Complete 2nd member expression
        // ```
        // obj.#prop = Math.pow(obj.#prop, right)
        //                      ^^^^^^^^^
        // ```
        let pow_left = Expression::from(
            ctx.ast.member_expression_private_field_expression(SPAN, obj, field, false),
        );

        (pow_left, temp_var_inits)
    }

    /// Get object part of 2nd member expression to be used as `left` in `Math.pow(left, right)`.
    ///
    /// Also update the original `obj` passed in to function, and add a temp var initializer, if necessary.
    ///
    /// Original:
    /// ```js
    /// obj.prop **= 2`
    /// ^^^ original `obj` passed in to this function
    /// ```
    ///
    /// is transformed to:
    ///
    /// If `obj` is a bound symbol:
    /// ```js
    /// obj["prop"] = Math.pow(obj["prop"], 2)
    /// ^^^ not updated        ^^^ returned
    /// ```
    ///
    /// If `obj` is unbound:
    /// ```js
    /// var _obj;
    /// _obj = obj, _obj["prop"] = Math.pow(_obj["prop"], 2)
    ///             ^^^^ updated            ^^^^ returned
    /// ^^^^^^^^^^ added to `temp_var_inits`
    /// ```
    ///
    /// Original:
    /// ```js
    /// obj.foo.bar.qux **= 2
    /// ^^^^^^^^^^^ original `obj` passed in to this function
    /// ```
    /// is transformed to:
    /// ```js
    /// var _obj$foo$bar;
    /// _obj$foo$bar = obj.foo.bar, _obj$foo$bar["qux"] = Math.pow(_obj$foo$bar["qux"], 2)
    ///                             ^^^^^^^^^^^^ updated           ^^^^^^^^^^^^ returned
    /// ^^^^^^^^^^^^^^^^^^^^^^^^^^ added to `temp_var_inits`
    /// ```
    fn get_second_member_expression_object(
        &mut self,
        obj: &mut Expression<'a>,
        temp_var_inits: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // If the object reference that we need to save is locally declared, evaluating it multiple times
        // will not trigger getters or setters. `super` cannot be directly assigned, so use it directly too.
        // TODO(improve-on-babel): We could also skip creating a temp var for `this.x **= 2`.
        match obj {
            Expression::Super(super_) => return ctx.ast.expression_super(super_.span),
            Expression::Identifier(ident) => {
                let symbol_id =
                    ctx.symbols().get_reference(ident.reference_id().unwrap()).symbol_id();
                if let Some(symbol_id) = symbol_id {
                    // This variable is declared in scope so evaluating it multiple times can't trigger a getter.
                    // No need for a temp var.
                    return ctx.ast.expression_from_identifier_reference(
                        ctx.create_bound_reference_id(
                            SPAN,
                            ident.name.clone(),
                            symbol_id,
                            ReferenceFlags::Read,
                        ),
                    );
                }
                // Unbound reference. Could possibly trigger a getter so we need to only evaluate it once.
                // Assign to a temp var.
            }
            _ => {
                // Other expression. Assign to a temp var.
            }
        }

        let binding = self.create_temp_var(ctx.ast.move_expression(obj), temp_var_inits, ctx);
        *obj = binding.create_read_expression(ctx);
        binding.create_read_expression(ctx)
    }

    /// `x **= right` -> `x = Math.pow(pow_left, right)` (with provided `pow_left`)
    fn convert_assignment(
        assign_expr: &mut AssignmentExpression<'a>,
        pow_left: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let pow_right = ctx.ast.move_expression(&mut assign_expr.right);
        assign_expr.right = Self::math_pow(pow_left, pow_right, ctx);
        assign_expr.operator = AssignmentOperator::Assign;
    }

    /// If needs temp var initializers, replace expression `expr` with `(temp1, temp2, expr)`.
    fn revise_expression(
        expr: &mut Expression<'a>,
        mut temp_var_inits: Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !temp_var_inits.is_empty() {
            temp_var_inits.reserve_exact(1);
            temp_var_inits.push(ctx.ast.move_expression(expr));
            *expr = ctx.ast.expression_sequence(SPAN, temp_var_inits);
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
        let arguments = ctx.ast.vec_from_iter([Argument::from(left), Argument::from(right)]);
        ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
    }

    /// Create a temporary variable.
    /// Add a `var _name;` statement to enclosing scope.
    /// Add initialization expression `_name = expr` to `temp_var_inits`.
    /// Return `BoundIdentifier` for the temp var.
    fn create_temp_var(
        &mut self,
        expr: Expression<'a>,
        temp_var_inits: &mut Vec<'a, Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let binding = ctx.generate_uid_in_current_scope_based_on_node(
            &expr,
            SymbolFlags::FunctionScopedVariable,
        );

        // var _name;
        self.ctx.var_declarations.insert(&binding, None, ctx);

        // Add new reference `_name = name` to `temp_var_inits`
        temp_var_inits.push(ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            binding.create_read_write_target(ctx),
            expr,
        ));

        binding
    }
}
