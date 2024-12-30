use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
use oxc_ecmascript::{ToInt32, ToJsString};
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::{
    es_target::ESTarget,
    identifier::is_identifier_name,
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};
use oxc_traverse::{traverse_mut_with_ctx, Ancestor, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{node_util::Ctx, CompressOptions, CompressorPass};

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntax.java>
pub struct PeepholeSubstituteAlternateSyntax {
    options: CompressOptions,
    /// Do not compress syntaxes that are hard to analyze inside the fixed loop.
    /// e.g. Do not compress `undefined -> void 0`, `true` -> `!0`.
    /// Opposite of `late` in Closure Compiler.
    in_fixed_loop: bool,

    // states
    in_define_export: bool,

    pub(crate) changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeSubstituteAlternateSyntax {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeSubstituteAlternateSyntax {
    fn exit_return_statement(
        &mut self,
        stmt: &mut ReturnStatement<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // We may fold `void 1` to `void 0`, so compress it after visiting
        self.compress_return_statement(stmt);
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.compress_catch_clause(catch);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for declarator in decl.declarations.iter_mut() {
            self.compress_variable_declarator(declarator, Ctx(ctx));
        }
    }

    /// Set `in_define_export` flag if this is a top-level statement of form:
    /// ```js
    /// Object.defineProperty(exports, 'Foo', {
    ///   enumerable: true,
    ///   get: function() { return Foo_1.Foo; }
    /// });
    /// ```
    fn enter_call_expression(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.parent().is_expression_statement()
            && Self::is_object_define_property_exports(call_expr)
        {
            self.in_define_export = true;
        }
    }

    fn exit_call_expression(&mut self, _expr: &mut CallExpression<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.in_define_export = false;
    }

    fn exit_property_key(&mut self, key: &mut PropertyKey<'a>, ctx: &mut TraverseCtx<'a>) {
        self.try_compress_property_key(key, ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.try_compress_computed_member_expression(expr, Ctx(ctx));
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);

        // Change syntax
        match expr {
            Expression::ArrowFunctionExpression(e) => self.try_compress_arrow_expression(e, ctx),
            Expression::ChainExpression(e) => self.try_compress_chain_call_expression(e, ctx),
            Expression::BinaryExpression(e) => self.try_compress_type_of_equal_string(e, ctx),
            _ => {}
        }

        // Fold
        if let Some(folded_expr) = match expr {
            Expression::Identifier(ident) => self.try_compress_undefined(ident, ctx),
            Expression::BooleanLiteral(_) => self.try_compress_boolean(expr, ctx),
            Expression::AssignmentExpression(e) => Self::try_compress_assignment_expression(e, ctx),
            Expression::LogicalExpression(e) => Self::try_compress_is_null_or_undefined(e, ctx),
            Expression::NewExpression(e) => Self::try_fold_new_expression(e, ctx),
            Expression::TemplateLiteral(t) => Self::try_fold_template_literal(t, ctx),
            Expression::BinaryExpression(e) => Self::try_compress_typeof_undefined(e, ctx),
            Expression::CallExpression(e) => {
                Self::try_fold_literal_constructor_call_expression(e, ctx)
                    .or_else(|| Self::try_fold_simple_function_call(e, ctx))
            }
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        }
    }
}

impl<'a, 'b> PeepholeSubstituteAlternateSyntax {
    pub fn new(options: CompressOptions, in_fixed_loop: bool) -> Self {
        Self { options, in_fixed_loop, in_define_export: false, changed: false }
    }

    /// Test `Object.defineProperty(exports, ...)`
    fn is_object_define_property_exports(call_expr: &CallExpression<'a>) -> bool {
        let Some(Argument::Identifier(ident)) = call_expr.arguments.first() else { return false };
        if ident.name != "exports" {
            return false;
        }

        // Use tighter check than `call_expr.callee.is_specific_member_access("Object", "defineProperty")`
        // because we're looking for `Object.defineProperty` specifically, not e.g. `Object['defineProperty']`
        if let Expression::StaticMemberExpression(callee) = &call_expr.callee {
            if let Expression::Identifier(id) = &callee.object {
                if id.name == "Object" && callee.property.name == "defineProperty" {
                    return true;
                }
            }
        }
        false
    }

    /// Transforms `undefined` => `void 0`
    fn try_compress_undefined(
        &self,
        ident: &IdentifierReference<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        if self.in_fixed_loop {
            return None;
        }
        if !ctx.is_identifier_undefined(ident) {
            return None;
        }
        Some(ctx.ast.void_0(ident.span))
    }

    /// Transforms boolean expression `true` => `!0` `false` => `!1`.
    /// Do not compress `true` in `Object.defineProperty(exports, 'Foo', {enumerable: true, ...})`.
    fn try_compress_boolean(
        &self,
        expr: &mut Expression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        if self.in_fixed_loop {
            return None;
        }
        let Expression::BooleanLiteral(lit) = expr else { return None };
        if self.in_define_export {
            return None;
        }
        let parent = ctx.ancestry.parent();
        let no_unary = {
            if let Ancestor::BinaryExpressionRight(u) = parent {
                !matches!(
                    u.operator(),
                    BinaryOperator::Addition // Other effect, like string concatenation.
                            | BinaryOperator::Instanceof // Relational operator.
                            | BinaryOperator::In
                            | BinaryOperator::StrictEquality // It checks type, so we should not fold.
                            | BinaryOperator::StrictInequality
                )
            } else {
                false
            }
        };
        // XOR: We should use `!neg` when it is not in binary expression.
        let num = ctx.ast.expression_numeric_literal(
            SPAN,
            if lit.value ^ no_unary { 0.0 } else { 1.0 },
            None,
            NumberBase::Decimal,
        );
        Some(if no_unary {
            num
        } else {
            ctx.ast.expression_unary(SPAN, UnaryOperator::LogicalNot, num)
        })
    }

    /// `() => { return foo })` -> `() => foo`
    fn try_compress_arrow_expression(
        &mut self,
        arrow_expr: &mut ArrowFunctionExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) {
        if !arrow_expr.expression
            && arrow_expr.body.directives.is_empty()
            && arrow_expr.body.statements.len() == 1
        {
            if let Some(body) = arrow_expr.body.statements.first_mut() {
                if let Statement::ReturnStatement(ret_stmt) = body {
                    let return_stmt_arg =
                        ret_stmt.argument.as_mut().map(|arg| ctx.ast.move_expression(arg));

                    if let Some(return_stmt_arg) = return_stmt_arg {
                        *body = ctx.ast.statement_expression(SPAN, return_stmt_arg);
                        arrow_expr.expression = true;
                        self.changed = true;
                    }
                }
            }
        }
    }

    /// Compress `typeof foo == "undefined"`
    ///
    /// - `typeof foo == "undefined"` (if foo is resolved) -> `foo === undefined`
    /// - `typeof foo != "undefined"` (if foo is resolved) -> `foo !== undefined`
    /// - `typeof foo == "undefined"` -> `typeof foo > "u"`
    /// - `typeof foo != "undefined"` -> `typeof foo < "u"`
    ///
    /// Enabled by `compress.typeofs`
    fn try_compress_typeof_undefined(
        expr: &mut BinaryExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let (new_eq_op, new_comp_op) = match expr.operator {
            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                (BinaryOperator::StrictEquality, BinaryOperator::GreaterThan)
            }
            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                (BinaryOperator::StrictInequality, BinaryOperator::LessThan)
            }
            _ => return None,
        };
        let pair = Self::commutative_pair(
            (&expr.left, &expr.right),
            |a| a.is_specific_string_literal("undefined").then_some(()),
            |b| {
                if let Expression::UnaryExpression(op) = b {
                    if op.operator == UnaryOperator::Typeof {
                        if let Expression::Identifier(id) = &op.argument {
                            return Some((*id).clone());
                        }
                    }
                }
                None
            },
        );
        let (_void_exp, id_ref) = pair?;
        let is_resolved = ctx.scopes().find_binding(ctx.current_scope_id(), &id_ref.name).is_some();
        if is_resolved {
            let left = Expression::Identifier(ctx.alloc(id_ref));
            let right = ctx.ast.void_0(SPAN);
            Some(ctx.ast.expression_binary(expr.span, left, new_eq_op, right))
        } else {
            let argument = Expression::Identifier(ctx.alloc(id_ref));
            let left = ctx.ast.expression_unary(SPAN, UnaryOperator::Typeof, argument);
            let right = ctx.ast.expression_string_literal(SPAN, "u", None);
            Some(ctx.ast.expression_binary(expr.span, left, new_comp_op, right))
        }
    }

    /// Compress `foo === null || foo === undefined` into `foo == null`.
    ///
    /// `foo === null || foo === undefined` => `foo == null`
    /// `foo !== null && foo !== undefined` => `foo != null`
    ///
    /// This compression assumes that `document.all` is a normal object.
    /// If that assumption does not hold, this compression is not allowed.
    /// - `document.all === null || document.all === undefined` is `false`
    /// - `document.all == null` is `true`
    fn try_compress_is_null_or_undefined(
        expr: &mut LogicalExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let op = expr.operator;
        let target_ops = match op {
            LogicalOperator::Or => (BinaryOperator::StrictEquality, BinaryOperator::Equality),
            LogicalOperator::And => (BinaryOperator::StrictInequality, BinaryOperator::Inequality),
            LogicalOperator::Coalesce => return None,
        };
        if let Some(new_expr) = Self::try_compress_is_null_or_undefined_for_left_and_right(
            &expr.left,
            &expr.right,
            expr.span,
            target_ops,
            ctx,
        ) {
            return Some(new_expr);
        }
        let Expression::LogicalExpression(left) = &mut expr.left else {
            return None;
        };
        if left.operator != op {
            return None;
        }
        Self::try_compress_is_null_or_undefined_for_left_and_right(
            &left.right,
            &expr.right,
            Span::new(left.right.span().start, expr.span.end),
            target_ops,
            ctx,
        )
        .map(|new_expr| {
            ctx.ast.expression_logical(
                expr.span,
                ctx.ast.move_expression(&mut left.left),
                expr.operator,
                new_expr,
            )
        })
    }

    fn try_compress_is_null_or_undefined_for_left_and_right(
        left: &Expression<'a>,
        right: &Expression<'a>,
        span: Span,
        (find_op, replace_op): (BinaryOperator, BinaryOperator),
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let pair = Self::commutative_pair(
            (&left, &right),
            |a| {
                if let Expression::BinaryExpression(op) = a {
                    if op.operator == find_op {
                        return Self::commutative_pair(
                            (&op.left, &op.right),
                            |a_a| a_a.is_null().then_some(a_a.span()),
                            |a_b| {
                                if let Expression::Identifier(id) = a_b {
                                    Some((a_b.span(), (*id).clone()))
                                } else {
                                    None
                                }
                            },
                        );
                    }
                }
                None
            },
            |b| {
                if let Expression::BinaryExpression(op) = b {
                    if op.operator == find_op {
                        return Self::commutative_pair(
                            (&op.left, &op.right),
                            |b_a| b_a.evaluate_to_undefined().then_some(()),
                            |b_b| {
                                if let Expression::Identifier(id) = b_b {
                                    Some((*id).clone())
                                } else {
                                    None
                                }
                            },
                        )
                        .map(|v| v.1);
                    }
                }
                None
            },
        );
        let ((null_expr_span, (left_id_expr_span, left_id_ref)), right_id_ref) = pair?;
        if left_id_ref.name != right_id_ref.name {
            return None;
        }
        let left_id_expr =
            ctx.ast.expression_identifier_reference(left_id_expr_span, left_id_ref.name);
        let null_expr = ctx.ast.expression_null_literal(null_expr_span);
        Some(ctx.ast.expression_binary(span, left_id_expr, replace_op, null_expr))
    }

    fn commutative_pair<A, F, G, RetF: 'a, RetG: 'a>(
        pair: (&A, &A),
        check_a: F,
        check_b: G,
    ) -> Option<(RetF, RetG)>
    where
        F: Fn(&A) -> Option<RetF>,
        G: Fn(&A) -> Option<RetG>,
    {
        if let Some(a) = check_a(pair.0) {
            if let Some(b) = check_b(pair.1) {
                return Some((a, b));
            }
        } else if let Some(a) = check_a(pair.1) {
            if let Some(b) = check_b(pair.0) {
                return Some((a, b));
            }
        }
        None
    }

    /// Removes redundant argument of `ReturnStatement`
    ///
    /// `return undefined` -> `return`
    /// `return void 0` -> `return`
    fn compress_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        if stmt.argument.as_ref().is_some_and(|expr| expr.is_undefined() || expr.is_void_0()) {
            stmt.argument = None;
            self.changed = true;
        }
    }

    fn compress_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: Ctx<'a, 'b>,
    ) {
        // Destructuring Pattern has error throwing side effect.
        if decl.kind.is_const() || decl.id.kind.is_destructuring_pattern() {
            return;
        }
        if decl.init.as_ref().is_some_and(|init| ctx.is_expression_undefined(init)) {
            decl.init = None;
            self.changed = true;
        }
    }

    fn try_compress_assignment_expression(
        expr: &mut AssignmentExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        let target = expr.left.as_simple_assignment_target_mut()?;
        if !matches!(expr.operator, AssignmentOperator::Subtraction) {
            return None;
        }
        match &expr.right {
            Expression::NumericLiteral(num) if num.value.to_int_32() == 1 => {
                // The `_` will not be placed to the target code.
                let target = std::mem::replace(
                    target,
                    ctx.ast.simple_assignment_target_identifier_reference(SPAN, "_"),
                );
                Some(ctx.ast.expression_update(SPAN, UpdateOperator::Decrement, true, target))
            }
            Expression::UnaryExpression(un)
                if matches!(un.operator, UnaryOperator::UnaryNegation) =>
            {
                let Expression::NumericLiteral(num) = &un.argument else { return None };
                (num.value.to_int_32() == 1).then(|| {
                    // The `_` will not be placed to the target code.
                    let target = std::mem::replace(
                        target,
                        ctx.ast.simple_assignment_target_identifier_reference(SPAN, "_"),
                    );
                    ctx.ast.expression_update(SPAN, UpdateOperator::Increment, true, target)
                })
            }
            _ => None,
        }
    }

    fn is_window_object(expr: &Expression) -> bool {
        expr.as_member_expression()
            .is_some_and(|mem_expr| mem_expr.is_specific_member_access("window", "Object"))
    }

    fn try_fold_new_expression(
        new_expr: &mut NewExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        // `new Object` -> `{}`
        if new_expr.arguments.is_empty()
            && (new_expr.callee.is_global_reference_name("Object", ctx.symbols())
                || Self::is_window_object(&new_expr.callee))
        {
            Some(ctx.ast.expression_object(new_expr.span, ctx.ast.vec(), None))
        } else if new_expr.callee.is_global_reference_name("Array", ctx.symbols()) {
            // `new Array` -> `[]`
            if new_expr.arguments.is_empty() {
                Some(Self::empty_array_literal(ctx))
            } else if new_expr.arguments.len() == 1 {
                let arg = new_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?;
                // `new Array(0)` -> `[]`
                if arg.is_number_0() {
                    Some(Self::empty_array_literal(ctx))
                }
                // `new Array(8)` -> `Array(8)`
                else if arg.is_number_literal() {
                    Some(Self::array_constructor_call(
                        ctx.ast.move_vec(&mut new_expr.arguments),
                        ctx,
                    ))
                }
                // `new Array(literal)` -> `[literal]`
                else if arg.is_literal() || matches!(arg, Expression::ArrayExpression(_)) {
                    let mut elements = ctx.ast.vec();
                    let element = ArrayExpressionElement::from(ctx.ast.move_expression(arg));
                    elements.push(element);
                    Some(Self::array_literal(elements, ctx))
                }
                // `new Array()` -> `Array()`
                else {
                    Some(Self::array_constructor_call(
                        ctx.ast.move_vec(&mut new_expr.arguments),
                        ctx,
                    ))
                }
            } else {
                // `new Array(1, 2, 3)` -> `[1, 2, 3]`
                let elements = ctx.ast.vec_from_iter(
                    new_expr
                        .arguments
                        .iter_mut()
                        .filter_map(|arg| arg.as_expression_mut())
                        .map(|arg| ArrayExpressionElement::from(ctx.ast.move_expression(arg))),
                );
                Some(Self::array_literal(elements, ctx))
            }
        } else {
            None
        }
    }

    fn try_fold_literal_constructor_call_expression(
        call_expr: &mut CallExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        // `Object()` -> `{}`
        if call_expr.arguments.is_empty()
            && (call_expr.callee.is_global_reference_name("Object", ctx.symbols())
                || Self::is_window_object(&call_expr.callee))
        {
            Some(ctx.ast.expression_object(call_expr.span, ctx.ast.vec(), None))
        } else if call_expr.callee.is_global_reference_name("Array", ctx.symbols()) {
            // `Array()` -> `[]`
            if call_expr.arguments.is_empty() {
                Some(Self::empty_array_literal(ctx))
            } else if call_expr.arguments.len() == 1 {
                let arg = call_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?;
                // `Array(0)` -> `[]`
                if arg.is_number_0() {
                    Some(Self::empty_array_literal(ctx))
                }
                // `Array(8)` -> `Array(8)`
                else if arg.is_number_literal() {
                    Some(Self::array_constructor_call(
                        ctx.ast.move_vec(&mut call_expr.arguments),
                        ctx,
                    ))
                }
                // `Array(literal)` -> `[literal]`
                else if arg.is_literal() || matches!(arg, Expression::ArrayExpression(_)) {
                    let mut elements = ctx.ast.vec();
                    let element = ArrayExpressionElement::from(ctx.ast.move_expression(arg));
                    elements.push(element);
                    Some(Self::array_literal(elements, ctx))
                } else {
                    None
                }
            } else {
                // `Array(1, 2, 3)` -> `[1, 2, 3]`
                let elements = ctx.ast.vec_from_iter(
                    call_expr
                        .arguments
                        .iter_mut()
                        .filter_map(|arg| arg.as_expression_mut())
                        .map(|arg| ArrayExpressionElement::from(ctx.ast.move_expression(arg))),
                );
                Some(Self::array_literal(elements, ctx))
            }
        } else {
            None
        }
    }

    fn try_fold_simple_function_call(
        call_expr: &mut CallExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) -> Option<Expression<'a>> {
        if call_expr.optional || call_expr.arguments.len() != 1 {
            return None;
        }
        if call_expr.callee.is_global_reference_name("Boolean", ctx.symbols()) {
            // `Boolean(a)` -> `!!(a)`
            // http://www.ecma-international.org/ecma-262/6.0/index.html#sec-boolean-constructor-boolean-value
            // and
            // http://www.ecma-international.org/ecma-262/6.0/index.html#sec-logical-not-operator-runtime-semantics-evaluation

            let arg = call_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?;

            if let Expression::UnaryExpression(unary_expr) = arg {
                if unary_expr.operator == UnaryOperator::LogicalNot {
                    return Some(ctx.ast.move_expression(arg));
                }
            }

            Some(ctx.ast.expression_unary(
                call_expr.span,
                UnaryOperator::LogicalNot,
                ctx.ast.expression_unary(
                    call_expr.span,
                    UnaryOperator::LogicalNot,
                    ctx.ast.move_expression(
                        call_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?,
                    ),
                ),
            ))
        } else if call_expr.callee.is_global_reference_name("String", ctx.symbols()) {
            // `String(a)` -> `'' + (a)`
            let arg = call_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?;

            if !matches!(arg, Expression::Identifier(_) | Expression::CallExpression(_))
                && !arg.is_literal()
            {
                return None;
            }

            Some(ctx.ast.expression_binary(
                call_expr.span,
                ctx.ast.expression_string_literal(SPAN, "", None),
                BinaryOperator::Addition,
                ctx.ast.move_expression(arg),
            ))
        } else {
            None
        }
    }

    /// `typeof foo === 'number'` -> `typeof foo == 'number'`
    fn try_compress_type_of_equal_string(
        &mut self,
        e: &mut BinaryExpression<'a>,
        _ctx: Ctx<'a, 'b>,
    ) {
        let op = match e.operator {
            BinaryOperator::StrictEquality => BinaryOperator::Equality,
            BinaryOperator::StrictInequality => BinaryOperator::Inequality,
            _ => return,
        };
        if Self::commutative_pair(
            (&e.left, &e.right),
            |a| a.is_string_literal().then_some(()),
            |b| matches!(b, Expression::UnaryExpression(e) if e.operator.is_typeof()).then_some(()),
        )
        .is_none()
        {
            return;
        }
        e.operator = op;
        self.changed = true;
    }

    fn try_compress_chain_call_expression(
        &mut self,
        chain_expr: &mut ChainExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) {
        if let ChainElement::CallExpression(call_expr) = &mut chain_expr.expression {
            // `window.Object?.()` -> `Object?.()`
            if call_expr.arguments.is_empty() && Self::is_window_object(&call_expr.callee) {
                call_expr.callee =
                    ctx.ast.expression_identifier_reference(call_expr.callee.span(), "Object");
                self.changed = true;
            }
        }
    }

    fn try_fold_template_literal(t: &TemplateLiteral, ctx: Ctx<'a, 'b>) -> Option<Expression<'a>> {
        t.to_js_string().map(|val| ctx.ast.expression_string_literal(t.span(), val, None))
    }

    /// returns an `Array()` constructor call with zero, one, or more arguments, copying from the input
    fn array_constructor_call(
        arguments: Vec<'a, Argument<'a>>,
        ctx: Ctx<'a, 'b>,
    ) -> Expression<'a> {
        let callee = ctx.ast.expression_identifier_reference(SPAN, "Array");
        ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
    }

    /// returns an array literal `[]` of zero, one, or more elements, copying from the input
    fn array_literal(
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        ctx: Ctx<'a, 'b>,
    ) -> Expression<'a> {
        ctx.ast.expression_array(SPAN, elements, None)
    }

    /// returns a new empty array literal expression: `[]`
    fn empty_array_literal(ctx: Ctx<'a, 'b>) -> Expression<'a> {
        Self::array_literal(ctx.ast.vec(), ctx)
    }

    // https://github.com/swc-project/swc/blob/4e2dae558f60a9f5c6d2eac860743e6c0b2ec562/crates/swc_ecma_minifier/src/compress/pure/properties.rs
    #[allow(clippy::cast_lossless)]
    fn try_compress_property_key(&mut self, key: &mut PropertyKey<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.in_fixed_loop {
            return;
        }
        let PropertyKey::StringLiteral(s) = key else { return };
        if match ctx.parent() {
            Ancestor::ObjectPropertyKey(key) => *key.computed(),
            Ancestor::BindingPropertyKey(key) => *key.computed(),
            Ancestor::MethodDefinitionKey(key) => *key.computed(),
            Ancestor::PropertyDefinitionKey(key) => *key.computed(),
            Ancestor::AccessorPropertyKey(key) => *key.computed(),
            _ => true,
        } {
            return;
        }
        if is_identifier_name(&s.value) {
            self.changed = true;
            *key = PropertyKey::StaticIdentifier(
                ctx.ast.alloc_identifier_name(s.span, s.value.clone()),
            );
        } else if (!s.value.starts_with('0') && !s.value.starts_with('+')) || s.value.len() <= 1 {
            if let Ok(value) = s.value.parse::<u32>() {
                self.changed = true;
                *key = PropertyKey::NumericLiteral(ctx.ast.alloc_numeric_literal(
                    s.span,
                    value as f64,
                    None,
                    NumberBase::Decimal,
                ));
            }
        }
    }

    /// `foo['bar']` -> `foo.bar`
    /// `foo?.['bar']` -> `foo?.bar`
    fn try_compress_computed_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) {
        if self.in_fixed_loop {
            return;
        }

        if let MemberExpression::ComputedMemberExpression(e) = expr {
            let Expression::StringLiteral(s) = &e.expression else { return };
            if !is_identifier_name(&s.value) {
                return;
            }
            let property = ctx.ast.identifier_name(s.span, s.value.clone());
            let object = ctx.ast.move_expression(&mut e.object);
            *expr = MemberExpression::StaticMemberExpression(
                ctx.ast.alloc_static_member_expression(e.span, object, property, e.optional),
            );
            self.changed = true;
        }
    }

    fn compress_catch_clause(&mut self, catch: &mut CatchClause<'a>) {
        if catch.body.body.is_empty()
            && !self.in_fixed_loop
            && self.options.target >= ESTarget::ES2019
        {
            if let Some(param) = &catch.param {
                if param.pattern.kind.is_binding_identifier() {
                    catch.param = None;
                    self.changed = true;
                }
            };
        }
    }
}

/// Port from <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{tester, CompressOptions};

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let options = CompressOptions::default();
        let mut pass = super::PeepholeSubstituteAlternateSyntax::new(options, false);
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn test_fold_return_result() {
        test("function f(){return !1;}", "function f(){return !1}");
        test("function f(){return null;}", "function f(){return null}");
        test("function f(){return void 0;}", "function f(){return}");
        test("function f(){return void foo();}", "function f(){return void foo()}");
        test("function f(){return undefined;}", "function f(){return}");
        // Here we handle the block in dce.
        test("function f(){if(a()){return undefined;}}", "function f(){if(a()){return}}");
    }

    #[test]
    fn test_undefined() {
        test("var x = undefined", "var x");
        test_same("var undefined = 1;function f() {var undefined=2;var x;}");
        test("function f(undefined) {}", "function f(undefined){}");
        test("try {} catch(undefined) {foo}", "try{}catch(undefined){foo}");
        test("for (undefined in {}) {}", "for(undefined in {}){}");
        test("undefined++;", "undefined++");
        test("undefined += undefined;", "undefined+=void 0");

        // shadowd
        test_same("(function(undefined) { let x = typeof undefined; })()");

        // destructuring throw error side effect
        test_same("var {} = void 0");
        test_same("var [] = void 0");
    }

    #[test]
    fn test_fold_true_false_comparison() {
        test("x == true", "x == 1");
        test("x == false", "x == 0");
        test("x != true", "x != 1");
        test("x < true", "x < 1");
        test("x <= true", "x <= 1");
        test("x > true", "x > 1");
        test("x >= true", "x >= 1");

        test("x instanceof true", "x instanceof !0");
        test("x + false", "x + !1");

        // Order: should perform the nearest.
        test("x == x instanceof false", "x == x instanceof !1");
        test("x in x >> true", "x in x >> 1");
        test("x == fake(false)", "x == fake(!1)");

        // The following should not be folded.
        test("x === true", "x === !0");
        test("x !== false", "x !== !1");
    }

    #[test]
    fn test_fold_subtraction_assignment() {
        test("x -= 1", "--x");
        test("x -= -1", "++x");
        test_same("x -= 2");
        test_same("x += 1"); // The string concatenation may be triggered, so we don't fold this.
        test_same("x += -1");
    }

    #[test]
    fn test_fold_literal_object_constructors() {
        test("x = new Object", "x = ({})");
        test("x = new Object()", "x = ({})");
        test("x = Object()", "x = ({})");

        test_same("x = (function f(){function Object(){this.x=4}return new Object();})();");
    }

    #[test]
    fn test_fold_literal_object_constructors_on_window() {
        test("x = new window.Object", "x = ({})");
        test("x = new window.Object()", "x = ({})");

        // Mustn't fold optional chains
        test("x = window.Object()", "x = ({})");
        test("x = window.Object?.()", "x = Object?.()");

        test(
            "x = (function f(){function Object(){this.x=4};return new window.Object;})();",
            "x = (function f(){function Object(){this.x=4}return {};})();",
        );
    }

    #[test]
    fn test_fold_literal_array_constructors() {
        test("x = new Array", "x = []");
        test("x = new Array()", "x = []");
        test("x = Array()", "x = []");
        // do not fold optional chains
        test_same("x = Array?.()");

        // One argument
        test("x = new Array(0)", "x = []");
        test("x = new Array(\"a\")", "x = [\"a\"]");
        test("x = new Array(7)", "x = Array(7)");
        test("x = new Array(y)", "x = Array(y)");
        test("x = new Array(foo())", "x = Array(foo())");
        test("x = Array(0)", "x = []");
        test("x = Array(\"a\")", "x = [\"a\"]");
        test_same("x = Array(7)");
        test_same("x = Array(y)");
        test_same("x = Array(foo())");

        // 1+ arguments
        test("x = new Array(1, 2, 3, 4)", "x = [1, 2, 3, 4]");
        test("x = Array(1, 2, 3, 4)", "x = [1, 2, 3, 4]");
        test("x = new Array('a', 1, 2, 'bc', 3, {}, 'abc')", "x = ['a', 1, 2, 'bc', 3, {}, 'abc']");
        test("x = Array('a', 1, 2, 'bc', 3, {}, 'abc')", "x = ['a', 1, 2, 'bc', 3, {}, 'abc']");
        test("x = new Array(Array(1, '2', 3, '4'))", "x = [[1, '2', 3, '4']]");
        test("x = Array(Array(1, '2', 3, '4'))", "x = [[1, '2', 3, '4']]");
        test(
            "x = new Array(Object(), Array(\"abc\", Object(), Array(Array())))",
            "x = [{}, [\"abc\", {}, [[]]]]",
        );
        test(
            "x = new Array(Object(), Array(\"abc\", Object(), Array(Array())))",
            "x = [{}, [\"abc\", {}, [[]]]]",
        );
    }

    #[test]
    #[ignore]
    fn test_split_comma_expressions() {
        // late = false;
        // Don't try to split in expressions.
        test_same("while (foo(), !0) boo()");
        test_same("var a = (foo(), !0);");
        test_same("a = (foo(), !0);");

        // Don't try to split COMMA under LABELs.
        test_same("a:a(),b()");
        test("1, 2, 3, 4", "1; 2; 3; 4");
        test("x = 1, 2, 3", "x = 1; 2; 3");
        test_same("x = (1, 2, 3)");
        test("1, (2, 3), 4", "1; 2; 3; 4");
        test("(x=2), foo()", "x=2; foo()");
        test("foo(), boo();", "foo(); boo()");
        test("(a(), b()), (c(), d());", "a(); b(); c(); d()");
        test("a(); b(); (c(), d());", "a(); b(); c(); d();");
        test("foo(), true", "foo();true");
        test_same("foo();true");
        test("function x(){foo(), !0}", "function x(){foo(); !0}");
        test_same("function x(){foo(); !0}");
    }

    #[test]
    #[ignore]
    fn test_comma1() {
        // late = false;
        test("1, 2", "1; 2");
        // late = true;
        // test_same("1, 2");
    }

    #[test]
    #[ignore]
    fn test_comma2() {
        // late = false;
        test("1, a()", "1; a()");
        test("1, a?.()", "1; a?.()");

        // late = true;
        // test_same("1, a()");
        // test_same("1, a?.()");
    }

    #[test]
    #[ignore]
    fn test_comma3() {
        // late = false;
        test("1, a(), b()", "1; a(); b()");
        test("1, a?.(), b?.()", "1; a?.(); b?.()");

        // late = true;
        // test_same("1, a(), b()");
        // test_same("1, a?.(), b?.()");
    }

    #[test]
    #[ignore]
    fn test_comma4() {
        // late = false;
        test("a(), b()", "a();b()");
        test("a?.(), b?.()", "a?.();b?.()");

        // late = true;
        // test_same("a(), b()");
        // test_same("a?.(), b?.()");
    }

    #[test]
    #[ignore]
    fn test_comma5() {
        // late = false;
        test("a(), b(), 1", "a(); b(); 1");
        test("a?.(), b?.(), 1", "a?.(); b?.(); 1");

        // late = true;
        // test_same("a(), b(), 1");
        // test_same("a?.(), b?.(), 1");
    }

    #[test]
    #[ignore]
    fn test_string_array_splitting() {
        test_same("var x=['1','2','3','4']");
        test_same("var x=['1','2','3','4','5']");
        test("var x=['1','2','3','4','5','6']", "var x='123456'.split('')");
        test("var x=['1','2','3','4','5','00']", "var x='1 2 3 4 5 00'.split(' ')");
        test("var x=['1','2','3','4','5','6','7']", "var x='1234567'.split('')");
        test("var x=['1','2','3','4','5','6','00']", "var x='1 2 3 4 5 6 00'.split(' ')");
        test("var x=[' ,',',',',',',',',',',']", "var x=' ,;,;,;,;,;,'.split(';')");
        test("var x=[',,',' ',',',',',',',',']", "var x=',,; ;,;,;,;,'.split(';')");
        test("var x=['a,',' ',',',',',',',',']", "var x='a,; ;,;,;,;,'.split(';')");

        // all possible delimiters used, leave it alone
        test_same("var x=[',', ' ', ';', '{', '}']");
    }

    #[test]
    fn test_template_string_to_string() {
        test("`abcde`", "'abcde'");
        test("`ab cd ef`", "'ab cd ef'");
        test_same("`hello ${name}`");
        test_same("tag `hello ${name}`");
        test_same("tag `hello`");
        test("`hello ${'foo'}`", "'hello foo'");
        test("`${2} bananas`", "'2 bananas'");
        test("`This is ${true}`", "'This is true'");
    }

    #[test]
    #[ignore]
    fn test_bind_to_call1() {
        test("(goog.bind(f))()", "f()");
        test("(goog.bind(f,a))()", "f.call(a)");
        test("(goog.bind(f,a,b))()", "f.call(a,b)");

        test("(goog.bind(f))(a)", "f(a)");
        test("(goog.bind(f,a))(b)", "f.call(a,b)");
        test("(goog.bind(f,a,b))(c)", "f.call(a,b,c)");

        test("(goog.partial(f))()", "f()");
        test("(goog.partial(f,a))()", "f(a)");
        test("(goog.partial(f,a,b))()", "f(a,b)");

        test("(goog.partial(f))(a)", "f(a)");
        test("(goog.partial(f,a))(b)", "f(a,b)");
        test("(goog.partial(f,a,b))(c)", "f(a,b,c)");

        test("((function(){}).bind())()", "((function(){}))()");
        test("((function(){}).bind(a))()", "((function(){})).call(a)");
        test("((function(){}).bind(a,b))()", "((function(){})).call(a,b)");

        test("((function(){}).bind())(a)", "((function(){}))(a)");
        test("((function(){}).bind(a))(b)", "((function(){})).call(a,b)");
        test("((function(){}).bind(a,b))(c)", "((function(){})).call(a,b,c)");

        // Without using type information we don't know "f" is a function.
        test_same("(f.bind())()");
        test_same("(f.bind(a))()");
        test_same("(f.bind())(a)");
        test_same("(f.bind(a))(b)");

        // Don't rewrite if the bind isn't the immediate call target
        test_same("(goog.bind(f)).call(g)");
    }

    #[test]
    #[ignore]
    fn test_bind_to_call2() {
        test("(goog$bind(f))()", "f()");
        test("(goog$bind(f,a))()", "f.call(a)");
        test("(goog$bind(f,a,b))()", "f.call(a,b)");

        test("(goog$bind(f))(a)", "f(a)");
        test("(goog$bind(f,a))(b)", "f.call(a,b)");
        test("(goog$bind(f,a,b))(c)", "f.call(a,b,c)");

        test("(goog$partial(f))()", "f()");
        test("(goog$partial(f,a))()", "f(a)");
        test("(goog$partial(f,a,b))()", "f(a,b)");

        test("(goog$partial(f))(a)", "f(a)");
        test("(goog$partial(f,a))(b)", "f(a,b)");
        test("(goog$partial(f,a,b))(c)", "f(a,b,c)");
        // Don't rewrite if the bind isn't the immediate call target
        test_same("(goog$bind(f)).call(g)");
    }

    #[test]
    #[ignore]
    fn test_bind_to_call3() {
        // TODO(johnlenz): The code generator wraps free calls with (0,...) to
        // prevent leaking "this", but the parser doesn't unfold it, making a
        // AST comparison fail.  For now do a string comparison to validate the
        // correct code is in fact generated.
        // The FREE call wrapping should be moved out of the code generator
        // and into a denormalizing pass.
        // disableCompareAsTree();
        // retraverseOnChange = true;
        // late = false;

        test("(goog.bind(f.m))()", "(0,f.m)()");
        test("(goog.bind(f.m,a))()", "f.m.call(a)");

        test("(goog.bind(f.m))(a)", "(0,f.m)(a)");
        test("(goog.bind(f.m,a))(b)", "f.m.call(a,b)");

        test("(goog.partial(f.m))()", "(0,f.m)()");
        test("(goog.partial(f.m,a))()", "(0,f.m)(a)");

        test("(goog.partial(f.m))(a)", "(0,f.m)(a)");
        test("(goog.partial(f.m,a))(b)", "(0,f.m)(a,b)");

        // Without using type information we don't know "f" is a function.
        test_same("f.m.bind()()");
        test_same("f.m.bind(a)()");
        test_same("f.m.bind()(a)");
        test_same("f.m.bind(a)(b)");

        // Don't rewrite if the bind isn't the immediate call target
        test_same("goog.bind(f.m).call(g)");
    }

    #[test]
    fn test_simple_function_call1() {
        test("var a = String(23)", "var a = '' + 23");
        // Don't fold the existence check to preserve behavior
        test_same("var a = String?.(23)");

        test("var a = String('hello')", "var a = '' + 'hello'");
        // Don't fold the existence check to preserve behavior
        test_same("var a = String?.('hello')");

        test_same("var a = String('hello', bar());");
        test_same("var a = String({valueOf: function() { return 1; }});");
    }

    #[test]
    fn test_simple_function_call2() {
        test("var a = Boolean(true)", "var a = !0");
        // Don't fold the existence check to preserve behavior
        test("var a = Boolean?.(true)", "var a = Boolean?.(!0)");

        test("var a = Boolean(false)", "var a = !1");
        // Don't fold the existence check to preserve behavior
        test("var a = Boolean?.(false)", "var a = Boolean?.(!1)");

        test("var a = Boolean(1)", "var a = !!1");
        // Don't fold the existence check to preserve behavior
        test_same("var a = Boolean?.(1)");

        test("var a = Boolean(x)", "var a = !!x");
        // Don't fold the existence check to preserve behavior
        test_same("var a = Boolean?.(x)");

        test("var a = Boolean({})", "var a = !!{}");
        // Don't fold the existence check to preserve behavior
        test_same("var a = Boolean?.({})");

        test_same("var a = Boolean()");
        test_same("var a = Boolean(!0, !1);");
    }

    #[test]
    #[ignore]
    fn test_rotate_associative_operators() {
        test("a || (b || c); a * (b * c); a | (b | c)", "(a || b) || c; (a * b) * c; (a | b) | c");
        test_same("a % (b % c); a / (b / c); a - (b - c);");
        test("a * (b % c);", "b % c * a");
        test("a * b * (c / d)", "c / d * b * a");
        test("(a + b) * (c % d)", "c % d * (a + b)");
        test_same("(a / b) * (c % d)");
        test_same("(c = 5) * (c % d)");
        test("(a + b) * c * (d % e)", "d % e * c * (a + b)");
        test("!a * c * (d % e)", "d % e * c * !a");
    }

    #[test]
    #[ignore]
    fn nullish_coalesce() {
        test("a ?? (b ?? c);", "(a ?? b) ?? c");
    }

    #[test]
    #[ignore]
    fn test_no_rotate_infinite_loop() {
        test("1/x * (y/1 * (1/z))", "1/x * (y/1) * (1/z)");
        test_same("1/x * (y/1) * (1/z)");
    }

    #[test]
    fn test_fold_arrow_function_return() {
        test("const foo = () => { return 'baz' }", "const foo = () => 'baz'");
        test_same("const foo = () => { foo; return 'baz' }");
    }

    #[test]
    fn test_fold_is_typeof_equals_undefined_resolved() {
        test("var x; typeof x !== 'undefined'", "var x; x !== void 0");
        test("var x; typeof x != 'undefined'", "var x; x !== void 0");
        test("var x; 'undefined' !== typeof x", "var x; x !== void 0");
        test("var x; 'undefined' != typeof x", "var x; x !== void 0");

        test("var x; typeof x === 'undefined'", "var x; x === void 0");
        test("var x; typeof x == 'undefined'", "var x; x === void 0");
        test("var x; 'undefined' === typeof x", "var x; x === void 0");
        test("var x; 'undefined' == typeof x", "var x; x === void 0");

        test(
            "var x; function foo() { typeof x !== 'undefined' }",
            "var x; function foo() { x !== void 0 }",
        );
        test(
            "typeof x !== 'undefined'; function foo() { var x }",
            "typeof x < 'u'; function foo() { var x }",
        );
        test("typeof x !== 'undefined'; { var x }", "x !== void 0; { var x }");
        test("typeof x !== 'undefined'; { let x }", "typeof x < 'u'; { let x }");
        test("typeof x !== 'undefined'; var x", "x !== void 0; var x");
        // input and output both errors with same TDZ error
        test("typeof x !== 'undefined'; let x", "x !== void 0; let x");
    }

    /// Port from <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser_test.go#L4658>
    #[test]
    fn test_fold_is_typeof_equals_undefined() {
        test("typeof x !== 'undefined'", "typeof x < 'u'");
        test("typeof x != 'undefined'", "typeof x < 'u'");
        test("'undefined' !== typeof x", "typeof x < 'u'");
        test("'undefined' != typeof x", "typeof x < 'u'");

        test("typeof x === 'undefined'", "typeof x > 'u'");
        test("typeof x == 'undefined'", "typeof x > 'u'");
        test("'undefined' === typeof x", "typeof x > 'u'");
        test("'undefined' == typeof x", "typeof x > 'u'");
    }

    #[test]
    fn test_fold_is_null_or_undefined() {
        test("foo === null || foo === undefined", "foo == null");
        test("foo === undefined || foo === null", "foo == null");
        test("foo === null || foo === void 0", "foo == null");
        test("foo === null || foo === void 0 || foo === 1", "foo == null || foo === 1");
        test("foo === 1 || foo === null || foo === void 0", "foo === 1 || foo == null");
        test_same("foo === void 0 || bar === null");
        test_same("foo !== 1 && foo === void 0 || foo === null");
        test_same("foo.a === void 0 || foo.a === null"); // cannot be folded because accessing foo.a might have a side effect

        test("foo !== null && foo !== undefined", "foo != null");
        test("foo !== undefined && foo !== null", "foo != null");
        test("foo !== null && foo !== void 0", "foo != null");
        test("foo !== null && foo !== void 0 && foo !== 1", "foo != null && foo !== 1");
        test("foo !== 1 && foo !== null && foo !== void 0", "foo !== 1 && foo != null");
        test("foo !== 1 || foo !== void 0 && foo !== null", "foo !== 1 || foo != null");
        test_same("foo !== void 0 && bar !== null");
    }

    #[test]
    fn test_try_compress_type_of_equal_string() {
        test("typeof foo === 'number'", "typeof foo == 'number'");
        test("'number' === typeof foo", "'number' == typeof foo");
        test("typeof foo === `number`", "typeof foo == 'number'");
        test("`number` === typeof foo", "'number' == typeof foo");
        test("typeof foo !== 'number'", "typeof foo != 'number'");
        test("'number' !== typeof foo", "'number' != typeof foo");
        test("typeof foo !== `number`", "typeof foo != 'number'");
        test("`number` !== typeof foo", "'number' != typeof foo");
    }

    #[test]
    fn test_object_key() {
        test("({ '0': _, 'a': _ })", "({ 0: _, a: _ })");
        test_same("({ '1.1': _, '😊': _, 'a.a': _ })");
    }

    #[test]
    fn test_computed_to_member_expression() {
        test("x['true']", "x.true");
        test_same("x['😊']");
    }

    #[test]
    fn optional_catch_binding() {
        test("try {} catch(e) {}", "try {} catch {}");
        test_same("try {} catch([e]) {}");
        test_same("try {} catch({e}) {}");

        let allocator = Allocator::default();
        let options = CompressOptions {
            target: oxc_syntax::es_target::ESTarget::ES2018,
            ..CompressOptions::default()
        };
        let mut pass = super::PeepholeSubstituteAlternateSyntax::new(options, false);
        let code = "try {} catch(e) {}";
        tester::test(&allocator, code, code, &mut pass);
    }

    /// Port from <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/ConvertToDottedPropertiesTest.java>
    mod convert_to_dotted_properties {
        use super::{test, test_same};

        #[test]
        fn test_convert_to_dotted_properties_convert() {
            test("a['p']", "a.p");
            test("a['_p_']", "a._p_");
            test("a['_']", "a._");
            test("a['$']", "a.$");
            test("a.b.c['p']", "a.b.c.p");
            test("a.b['c'].p", "a.b.c.p");
            test("a['p']();", "a.p();");
            test("a()['p']", "a().p");
            // ASCII in Unicode is always safe.
            test("a['\\u0041A']", "a.AA");
            // This is safe for ES5+. (keywords cannot be used for ES3)
            test("a['default']", "a.default");
            // This is safe for ES2015+. (\u1d17 was introduced in Unicode 3.1, ES2015+ uses Unicode 5.1+)
            test("a['\\u1d17A']", "a.\u{1d17}A");
            // Latin capital N with tilde - this is safe for ES3+.
            test("a['\\u00d1StuffAfter']", "a.\u{00d1}StuffAfter");
        }

        #[test]
        fn test_convert_to_dotted_properties_do_not_convert() {
            test_same("a[0]");
            test_same("a['']");
            test_same("a[' ']");
            test_same("a[',']");
            test_same("a[';']");
            test_same("a[':']");
            test_same("a['.']");
            test_same("a['0']");
            test_same("a['p ']");
            test_same("a['p' + '']");
            test_same("a[p]");
            test_same("a[P]");
            test_same("a[$]");
            test_same("a[p()]");
            // Ignorable control characters are ok in Java identifiers, but not in JS.
            test_same("a['A\\u0004']");
        }

        #[test]
        fn test_convert_to_dotted_properties_already_dotted() {
            test_same("a.b");
            test_same("var a = {b: 0};");
        }

        #[test]
        fn test_convert_to_dotted_properties_quoted_props() {
            test_same("({'':0})");
            test_same("({'1.0':0})");
            test("({'\\u1d17A':0})", "({ \u{1d17}A: 0 })");
            test_same("({'a\\u0004b':0})");
        }

        #[test]
        fn test5746867() {
            test_same("var a = { '$\\\\' : 5 };");
            test_same("var a = { 'x\\\\u0041$\\\\' : 5 };");
        }

        #[test]
        fn test_convert_to_dotted_properties_optional_chaining() {
            test("data?.['name']", "data?.name");
            test("data?.['name']?.['first']", "data?.name?.first");
            test("data['name']?.['first']", "data.name?.first");
            test_same("a?.[0]");
            test_same("a?.['']");
            test_same("a?.[' ']");
            test_same("a?.[',']");
            test_same("a?.[';']");
            test_same("a?.[':']");
            test_same("a?.['.']");
            test_same("a?.['0']");
            test_same("a?.['p ']");
            test_same("a?.['p' + '']");
            test_same("a?.[p]");
            test_same("a?.[P]");
            test_same("a?.[$]");
            test_same("a?.[p()]");
            // This is safe for ES5+. (keywords cannot be used for ES3)
            test("a?.['default']", "a?.default");
        }

        #[test]
        #[ignore]
        fn test_convert_to_dotted_properties_computed_property_or_field() {
            test("const test1 = {['prop1']:87};", "const test1 = {prop1:87};");
            test(
                "const test1 = {['prop1']:87,['prop2']:bg,['prop3']:'hfd'};",
                "const test1 = {prop1:87,prop2:bg,prop3:'hfd'};",
            );
            test(
                "o = {['x']: async function(x) { return await x + 1; }};",
                "o = {x:async function (x) { return await x + 1; }};",
            );
            test("o = {['x']: function*(x) {}};", "o = {x: function*(x) {}};");
            test(
                "o = {['x']: async function*(x) { return await x + 1; }};",
                "o = {x:async function*(x) { return await x + 1; }};",
            );
            test("class C {'x' = 0;  ['y'] = 1;}", "class C { x= 0;y= 1;}");
            test("class C {'m'() {} }", "class C {m() {}}");

            test(
                "const o = {'b'() {}, ['c']() {}};",
                "const o = {b: function() {}, c:function(){}};",
            );
            test("o = {['x']: () => this};", "o = {x: () => this};");

            test("const o = {get ['d']() {}};", "const o = {get d() {}};");
            test("const o = { set ['e'](x) {}};", "const o = { set e(x) {}};");
            test(
                "class C {'m'() {}  ['n']() {} 'x' = 0;  ['y'] = 1;}",
                "class C {m() {}  n() {} x= 0;y= 1;}",
            );
            test(
                "const o = { get ['d']() {},  set ['e'](x) {}};",
                "const o = {get d() {},  set e(x){}};",
            );
            test(
                "const o = {['a']: 1,'b'() {}, ['c']() {},  get ['d']() {},  set ['e'](x) {}};",
                "const o = {a: 1,b: function() {}, c: function() {},  get d() {},  set e(x) {}};",
            );

            // test static keyword
            test(
                r"
                class C {
                'm'(){}
                ['n'](){}
                static 'x' = 0;
                static ['y'] = 1;}
            ",
                r"
                class C {
                m(){}
                n(){}
                static x = 0;
                static y= 1;}
            ",
            );
            test(
                r"
                window['MyClass'] = class {
                static ['Register'](){}
                };
            ",
                r"
                window.MyClass = class {
                static Register(){}
                };
            ",
            );
            test(
                r"
                class C {
                'method'(){}
                async ['method1'](){}
                *['method2'](){}
                static ['smethod'](){}
                static async ['smethod1'](){}
                static *['smethod2'](){}}
            ",
                r"
                class C {
                method(){}
                async method1(){}
                *method2(){}
                static smethod(){}
                static async smethod1(){}
                static *smethod2(){}}
            ",
            );

            test_same("const o = {[fn()]: 0}");
            test_same("const test1 = {[0]:87};");
            test_same("const test1 = {['default']:87};");
            test_same("class C { ['constructor']() {} }");
            test_same("class C { ['constructor'] = 0 }");
        }

        #[test]
        #[ignore]
        fn test_convert_to_dotted_properties_computed_property_with_default_value() {
            test("const {['o']: o = 0} = {};", "const {o:o = 0} = {};");
        }

        #[test]
        fn test_convert_to_dotted_properties_continue_optional_chaining() {
            test("const opt1 = window?.a?.['b'];", "const opt1 = window?.a?.b;");

            test("const opt2 = window?.a['b'];", "const opt2 = window?.a.b;");
            test(
                r"
                const chain =
                window['a'].x.y.b.x.y['c'].x.y?.d.x.y['e'].x.y
                ['f-f'].x.y?.['g-g'].x.y?.['h'].x.y['i'].x.y;
            ",
                r"
                const chain = window.a.x.y.b.x.y.c.x.y?.d.x.y.e.x.y
                ['f-f'].x.y?.['g-g'].x.y?.h.x.y.i.x.y;
            ",
            );
        }
    }
}
