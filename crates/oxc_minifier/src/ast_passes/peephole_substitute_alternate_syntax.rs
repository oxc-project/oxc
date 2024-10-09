use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::number::ToJsInt32;
use oxc_syntax::{
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{node_util::NodeUtil, CompressOptions, CompressorPass};

/// A peephole optimization that minimizes code by simplifying conditional
/// expressions, replacing IFs with HOOKs, replacing object constructors
/// with literals, and simplifying returns.
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntax.java>
pub struct PeepholeSubstituteAlternateSyntax {
    options: CompressOptions,
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
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.compress_block(stmt);
        // self.compress_while(stmt);
    }

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
            self.compress_variable_declarator(declarator, ctx);
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
        if let Expression::AssignmentExpression(assignment_expr) = expr {
            if let Some(new_expr) = self.try_compress_assignment_expression(assignment_expr, ctx) {
                *expr = new_expr;
                self.changed = true;
            }
        }
        if !self.compress_undefined(expr, ctx) {
            self.compress_boolean(expr, ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::NewExpression(new_expr) => {
                if let Some(new_expr) = self.try_fold_new_expression(new_expr, ctx) {
                    *expr = new_expr;
                    self.changed = true;
                }
            }
            Expression::CallExpression(call_expr) => {
                if let Some(call_expr) = self.try_fold_call_expression(call_expr, ctx) {
                    *expr = call_expr;
                    self.changed = true;
                }
            }
            Expression::ChainExpression(chain_expr) => {
                if let ChainElement::CallExpression(call_expr) = &mut chain_expr.expression {
                    self.try_fold_chain_call_expression(call_expr, ctx);
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
        self.compress_typeof_undefined(expr, ctx);
    }
}

impl<'a> PeepholeSubstituteAlternateSyntax {
    pub fn new(options: CompressOptions) -> Self {
        Self { options, in_define_export: false, changed: false }
    }

    /* Utilities */

    /// Transforms `undefined` => `void 0`
    fn compress_undefined(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        if ctx.is_expression_undefined(expr) {
            *expr = ctx.ast.void_0(expr.span());
            return true;
        };
        false
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

    /// Remove block from single line blocks
    /// `{ block } -> block`
    fn compress_block(&mut self, stmt: &mut Statement<'a>) {
        if let Statement::BlockStatement(block) = stmt {
            // Avoid compressing `if (x) { var x = 1 }` to `if (x) var x = 1` due to different
            // semantics according to AnnexB, which lead to different semantics.
            if block.body.len() == 1 && !block.body[0].is_declaration() {
                *stmt = block.body.remove(0);
                self.compress_block(stmt);
                self.changed = true;
            }
        }
    }

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
    /// Enabled by `compress.booleans`.
    /// Do not compress `true` in `Object.defineProperty(exports, 'Foo', {enumerable: true, ...})`.
    fn compress_boolean(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        let Expression::BooleanLiteral(lit) = expr else { return false };
        if self.options.booleans && !self.in_define_export {
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
            true
        } else {
            false
        }
    }

    /// Compress `typeof foo == "undefined"` into `typeof foo > "u"`
    /// Enabled by `compress.typeofs`
    fn compress_typeof_undefined(
        &self,
        expr: &mut BinaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.options.typeofs {
            return;
        }
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
        let argument = ctx.ast.expression_from_identifier_reference(id_ref);
        let left = ctx.ast.unary_expression(SPAN, UnaryOperator::Typeof, argument);
        let right = ctx.ast.string_literal(SPAN, "u");
        let binary_expr = ctx.ast.binary_expression(
            expr.span,
            ctx.ast.expression_from_unary(left),
            BinaryOperator::GreaterThan,
            ctx.ast.expression_from_string_literal(right),
        );
        *expr = binary_expr;
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
        ctx: &mut TraverseCtx<'a>,
    ) {
        if decl.kind.is_const() {
            return;
        }
        if decl.init.as_ref().is_some_and(|init| ctx.is_expression_undefined(init)) {
            decl.init = None;
            self.changed = true;
        }
    }

    fn try_compress_assignment_expression(
        &mut self,
        expr: &mut AssignmentExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let target = expr.left.as_simple_assignment_target_mut()?;
        if matches!(expr.operator, AssignmentOperator::Subtraction) {
            match &expr.right {
                Expression::NumericLiteral(num) if num.value.to_js_int_32() == 1 => {
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
                        (num.value.to_js_int_32() == 1).then(|| {
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
        &mut self,
        new_expr: &mut NewExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
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
                Some(self.empty_array_literal(ctx))
            } else if new_expr.arguments.len() == 1 {
                let arg = new_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?;
                // `new Array(0)` -> `[]`
                if arg.is_number_0() {
                    Some(self.empty_array_literal(ctx))
                }
                // `new Array(8)` -> `Array(8)`
                else if arg.is_number_literal() {
                    Some(
                        self.array_constructor_call(ctx.ast.move_vec(&mut new_expr.arguments), ctx),
                    )
                }
                // `new Array(literal)` -> `[literal]`
                else if arg.is_literal() || matches!(arg, Expression::ArrayExpression(_)) {
                    let mut elements = ctx.ast.vec();
                    let element =
                        ctx.ast.array_expression_element_expression(ctx.ast.move_expression(arg));
                    elements.push(element);
                    Some(self.array_literal(elements, ctx))
                }
                // `new Array()` -> `Array()`
                else {
                    Some(
                        self.array_constructor_call(ctx.ast.move_vec(&mut new_expr.arguments), ctx),
                    )
                }
            } else {
                // `new Array(1, 2, 3)` -> `[1, 2, 3]`
                let elements = ctx.ast.vec_from_iter(
                    new_expr.arguments.iter_mut().filter_map(|arg| arg.as_expression_mut()).map(
                        |arg| {
                            ctx.ast
                                .array_expression_element_expression(ctx.ast.move_expression(arg))
                        },
                    ),
                );
                Some(self.array_literal(elements, ctx))
            }
        } else {
            None
        }
    }

    fn try_fold_call_expression(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
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
                Some(self.empty_array_literal(ctx))
            } else if call_expr.arguments.len() == 1 {
                let arg = call_expr.arguments.get_mut(0).and_then(|arg| arg.as_expression_mut())?;
                // `Array(0)` -> `[]`
                if arg.is_number_0() {
                    Some(self.empty_array_literal(ctx))
                }
                // `Array(8)` -> `Array(8)`
                else if arg.is_number_literal() {
                    Some(
                        self.array_constructor_call(
                            ctx.ast.move_vec(&mut call_expr.arguments),
                            ctx,
                        ),
                    )
                }
                // `Array(literal)` -> `[literal]`
                else if arg.is_literal() || matches!(arg, Expression::ArrayExpression(_)) {
                    let mut elements = ctx.ast.vec();
                    let element =
                        ctx.ast.array_expression_element_expression(ctx.ast.move_expression(arg));
                    elements.push(element);
                    Some(self.array_literal(elements, ctx))
                } else {
                    None
                }
            } else {
                // `Array(1, 2, 3)` -> `[1, 2, 3]`
                let elements = ctx.ast.vec_from_iter(
                    call_expr.arguments.iter_mut().filter_map(|arg| arg.as_expression_mut()).map(
                        |arg| {
                            ctx.ast
                                .array_expression_element_expression(ctx.ast.move_expression(arg))
                        },
                    ),
                );
                Some(self.array_literal(elements, ctx))
            }
        } else {
            None
        }
    }

    fn try_fold_chain_call_expression(
        &mut self,
        call_expr: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
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
        &self,
        arguments: Vec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let callee = ctx.ast.expression_identifier_reference(SPAN, "Array");
        ctx.ast.expression_call(SPAN, callee, NONE, arguments, false)
    }

    /// returns an array literal `[]` of zero, one, or more elements, copying from the input
    fn array_literal(
        &self,
        elements: Vec<'a, ArrayExpressionElement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        ctx.ast.expression_array(SPAN, elements, None)
    }

    /// returns a new empty array literal expression: `[]`
    fn empty_array_literal(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        self.array_literal(ctx.ast.vec(), ctx)
    }
}

/// <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{tester, CompressOptions};

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeSubstituteAlternateSyntax::new(CompressOptions::default());
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
        test("function f(){if(a()){return undefined;}}", "function f(){if(a())return}");
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
}
