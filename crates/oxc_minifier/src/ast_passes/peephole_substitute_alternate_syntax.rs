use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
use oxc_ecmascript::{ToInt32, ToJsString};
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{node_util::Ctx, CompressorPass};

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntax.java>
pub struct PeepholeSubstituteAlternateSyntax {
    /// Do not compress syntaxes that are hard to analyze inside the fixed loop.
    /// e.g. Do not compress `undefined -> void 0`, `true` -> `!0`.
    /// Opposite of `late` in Closure Compier.
    in_fixed_loop: bool,

    // states
    in_define_export: bool,

    changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeSubstituteAlternateSyntax {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
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

    fn enter_variable_declaration(
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

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);
        if let Expression::AssignmentExpression(assignment_expr) = expr {
            if let Some(new_expr) = Self::try_compress_assignment_expression(assignment_expr, ctx) {
                *expr = new_expr;
                self.changed = true;
            }
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let ctx = Ctx(ctx);
        self.try_compress_boolean(expr, ctx);
        self.try_compress_undefined(expr, ctx);
        match expr {
            Expression::NewExpression(new_expr) => {
                if let Some(new_expr) = Self::try_fold_new_expression(new_expr, ctx) {
                    *expr = new_expr;
                    self.changed = true;
                }
            }
            Expression::CallExpression(call_expr) => {
                if let Some(call_expr) =
                    Self::try_fold_literal_constructor_call_expression(call_expr, ctx)
                {
                    *expr = call_expr;
                    self.changed = true;
                } else if let Some(call_expr) = Self::try_fold_simple_function_call(call_expr, ctx)
                {
                    *expr = call_expr;
                    self.changed = true;
                }
            }
            Expression::ChainExpression(chain_expr) => {
                if let ChainElement::CallExpression(call_expr) = &mut chain_expr.expression {
                    self.try_fold_chain_call_expression(call_expr, ctx);
                }
            }
            Expression::TemplateLiteral(_) => {
                if let Some(val) = expr.to_js_string() {
                    *expr = ctx.ast.expression_string_literal(expr.span(), val);
                    self.changed = true;
                }
            }
            // `() => { return foo })` -> `() => foo`
            Expression::ArrowFunctionExpression(arrow_expr) => {
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
            _ => {}
        }
    }

    fn enter_binary_expression(
        &mut self,
        expr: &mut BinaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.compress_typeof_undefined(expr, Ctx(ctx));
    }
}

impl<'a, 'b> PeepholeSubstituteAlternateSyntax {
    pub fn new(in_fixed_loop: bool) -> Self {
        Self { in_fixed_loop, in_define_export: false, changed: false }
    }

    /* Utilities */

    /// Transforms `undefined` => `void 0`
    fn try_compress_undefined(&mut self, expr: &mut Expression<'a>, ctx: Ctx<'a, 'b>) {
        if self.in_fixed_loop {
            return;
        }
        if ctx.is_expression_undefined(expr) {
            *expr = ctx.ast.void_0(expr.span());
            self.changed = true;
        }
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

    /* Statements */

    // /// Transforms `while(expr)` to `for(;expr;)`
    // fn compress_while(&mut self, stmt: &mut Statement<'a>) {
    // let Statement::WhileStatement(while_stmt) = stmt else { return };
    // if self.options.loops {
    // let dummy_test = ctx.ast.expression_this(SPAN);
    // let test = std::mem::replace(&mut while_stmt.test, dummy_test);
    // let body = ctx.ast.move_statement(&mut while_stmt.body);
    // *stmt = ctx.ast.statement_for(SPAN, None, Some(test), None, body);
    // }
    // }

    /* Expressions */

    /// Transforms boolean expression `true` => `!0` `false` => `!1`.
    /// Do not compress `true` in `Object.defineProperty(exports, 'Foo', {enumerable: true, ...})`.
    fn try_compress_boolean(&mut self, expr: &mut Expression<'a>, ctx: Ctx<'a, 'b>) {
        if self.in_fixed_loop {
            return;
        }
        let Expression::BooleanLiteral(lit) = expr else { return };
        if !self.in_define_export {
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
                if lit.value ^ no_unary { "0" } else { "1" },
                NumberBase::Decimal,
            );
            *expr = if no_unary {
                num
            } else {
                ctx.ast.expression_unary(SPAN, UnaryOperator::LogicalNot, num)
            };
            self.changed = true;
        }
    }

    /// Compress `typeof foo == "undefined"` into `typeof foo > "u"`
    /// Enabled by `compress.typeofs`
    fn compress_typeof_undefined(&mut self, expr: &mut BinaryExpression<'a>, ctx: Ctx<'a, 'b>) {
        if !matches!(expr.operator, BinaryOperator::Equality | BinaryOperator::StrictEquality) {
            return;
        }
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
        let Some((_void_exp, id_ref)) = pair else {
            return;
        };
        let argument = Expression::Identifier(ctx.alloc(id_ref));
        let left = ctx.ast.expression_unary(SPAN, UnaryOperator::Typeof, argument);
        let right = ctx.ast.expression_string_literal(SPAN, "u");
        let binary_expr =
            ctx.ast.binary_expression(expr.span, left, BinaryOperator::GreaterThan, right);
        *expr = binary_expr;
        self.changed = true;
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
        if matches!(expr.operator, AssignmentOperator::Subtraction) {
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
                    if let Expression::NumericLiteral(num) = &un.argument {
                        (num.value.to_int_32() == 1).then(|| {
                            // The `_` will not be placed to the target code.
                            let target = std::mem::replace(
                                target,
                                ctx.ast.simple_assignment_target_identifier_reference(SPAN, "_"),
                            );
                            ctx.ast.expression_update(SPAN, UpdateOperator::Increment, true, target)
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
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
                ctx.ast.expression_string_literal(SPAN, ""),
                BinaryOperator::Addition,
                ctx.ast.move_expression(arg),
            ))
        } else {
            None
        }
    }

    fn try_fold_chain_call_expression(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: Ctx<'a, 'b>,
    ) {
        // `window.Object?.()` -> `Object?.()`
        if call_expr.arguments.is_empty() && Self::is_window_object(&call_expr.callee) {
            call_expr.callee =
                ctx.ast.expression_identifier_reference(call_expr.callee.span(), "Object");
            self.changed = true;
        }
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
}

/// Port from <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeSubstituteAlternateSyntax::new(false);
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
        test("try {} catch(undefined) {}", "try{}catch(undefined){}");
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
}
