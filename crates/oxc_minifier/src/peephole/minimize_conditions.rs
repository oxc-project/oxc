use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue, DetermineValueType},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_syntax::es_target::ESTarget;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

/// Minimize Conditions
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeMinimizeConditions.java>
impl<'a> PeepholeOptimizations {
    // The goal of this function is to "rotate" the AST if it's possible to use the
    // left-associative property of the operator to avoid unnecessary parentheses.
    //
    // When using this, make absolutely sure that the operator is actually
    // associative. For example, the "+" operator is not associative for
    // floating-point numbers.
    pub fn join_with_left_associative_op(
        &self,
        span: Span,
        op: LogicalOperator,
        a: Expression<'a>,
        b: Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Expression<'a> {
        // "(a, b) op c" => "a, b op c"
        if let Expression::SequenceExpression(mut sequence_expr) = a {
            if let Some(right) = sequence_expr.expressions.pop() {
                sequence_expr
                    .expressions
                    .push(self.join_with_left_associative_op(span, op, right, b, ctx));
            }
            return Expression::SequenceExpression(sequence_expr);
        }
        let mut a = a;
        let mut b = b;
        // "a op (b op c)" => "(a op b) op c"
        // "a op (b op (c op d))" => "((a op b) op c) op d"
        loop {
            if let Expression::LogicalExpression(logical_expr) = &mut b {
                if logical_expr.operator == op {
                    let right = logical_expr.left.take_in(ctx.ast);
                    a = self.join_with_left_associative_op(span, op, a, right, ctx);
                    b = logical_expr.right.take_in(ctx.ast);
                    continue;
                }
            }
            break;
        }
        // "a op b" => "a op b"
        // "(a op b) op c" => "(a op b) op c"
        let mut logic_expr = ctx.ast.expression_logical(span, a, op, b);
        self.minimize_logical_expression(&mut logic_expr, ctx);
        logic_expr
    }

    // `typeof foo === 'number'` -> `typeof foo == 'number'`
    //  ^^^^^^^^^^ `ctx.expression_value_type(&e.left).is_string()` is `true`.
    // `a instanceof b === true` -> `a instanceof b`
    // `a instanceof b === false` -> `!(a instanceof b)`
    //  ^^^^^^^^^^^^^^ `ctx.expression_value_type(&e.left).is_boolean()` is `true`.
    // `x >> +y !== 0` -> `x >> +y`
    //  ^^^^^^^ ctx.expression_value_type(&e.left).is_number()` is `true`.
    pub fn try_minimize_binary(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::BinaryExpression(e) = expr else { return };
        if !e.operator.is_equality() {
            return;
        }
        let left = e.left.value_type(ctx);
        let right = e.right.value_type(ctx);
        if left.is_undetermined() || right.is_undetermined() {
            return;
        }
        if left == right {
            match e.operator {
                BinaryOperator::StrictInequality => {
                    e.operator = BinaryOperator::Inequality;
                }
                BinaryOperator::StrictEquality => {
                    e.operator = BinaryOperator::Equality;
                }
                _ => {}
            }
        }
        if !left.is_boolean() {
            return;
        }
        if e.right.may_have_side_effects(ctx) {
            return;
        }
        let Some(mut b) = e.right.evaluate_value(ctx).and_then(ConstantValue::into_boolean) else {
            return;
        };
        match e.operator {
            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                e.operator = BinaryOperator::Equality;
                b = !b;
            }
            BinaryOperator::StrictEquality => {
                e.operator = BinaryOperator::Equality;
            }
            BinaryOperator::Equality => {}
            _ => return,
        }
        *expr = if b {
            e.left.take_in(ctx.ast)
        } else {
            let argument = e.left.take_in(ctx.ast);
            ctx.ast.expression_unary(e.span, UnaryOperator::LogicalNot, argument)
        };
        ctx.state.changed = true;
    }

    /// Compress `foo == true` into `foo == 1`.
    ///
    /// - `foo == true` => `foo == 1`
    /// - `foo != false` => `foo != 0`
    ///
    /// In `IsLooselyEqual`, `true` and `false` are converted to `1` and `0` first.
    /// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islooselyequal>
    pub fn try_compress_is_loose_boolean(e: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::BinaryExpression(e) = e else { return };
        if !matches!(e.operator, BinaryOperator::Equality | BinaryOperator::Inequality) {
            return;
        }
        if let Some(ConstantValue::Boolean(left_bool)) = e.left.evaluate_value(ctx) {
            e.left = ctx.ast.expression_numeric_literal(
                e.left.span(),
                if left_bool { 1.0 } else { 0.0 },
                None,
                NumberBase::Decimal,
            );
            ctx.state.changed = true;
            return;
        }
        if let Some(ConstantValue::Boolean(right_bool)) = e.right.evaluate_value(ctx) {
            e.right = ctx.ast.expression_numeric_literal(
                e.right.span(),
                if right_bool { 1.0 } else { 0.0 },
                None,
                NumberBase::Decimal,
            );
            ctx.state.changed = true;
        }
    }

    /// Returns the identifier or the assignment target's identifier of the given expression.
    pub fn extract_id_or_assign_to_id<'b>(
        expr: &'b Expression<'a>,
    ) -> Option<&'b IdentifierReference<'a>> {
        match expr {
            Expression::Identifier(id) => Some(id),
            Expression::AssignmentExpression(assign_expr) => {
                if assign_expr.operator == AssignmentOperator::Assign {
                    if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign_expr.left {
                        return Some(id);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Compress `a = a || b` to `a ||= b`
    ///
    /// This can only be done for resolved identifiers as this would avoid setting `a` when `a` is truthy.
    pub fn try_compress_normal_assignment_to_combined_logical_assignment(
        &self,
        expr: &mut AssignmentExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        if ctx.options().target < ESTarget::ES2020 {
            return;
        }
        if !matches!(expr.operator, AssignmentOperator::Assign) {
            return;
        }

        let Expression::LogicalExpression(logical_expr) = &mut expr.right else { return };
        // NOTE: if the right hand side is an anonymous function, applying this compression will
        // set the `name` property of that function.
        // Since codes relying on the fact that function's name is undefined should be rare,
        // we do this compression even if `keep_names` is enabled.

        let (
            AssignmentTarget::AssignmentTargetIdentifier(write_id_ref),
            Expression::Identifier(read_id_ref),
        ) = (&expr.left, &logical_expr.left)
        else {
            return;
        };
        // It should also early return when the reference might refer to a reference value created by a with statement
        // when the minifier supports with statements
        if write_id_ref.name != read_id_ref.name || ctx.is_global_reference(write_id_ref) {
            return;
        }

        let new_op = logical_expr.operator.to_assignment_operator();
        expr.operator = new_op;
        expr.right = logical_expr.right.take_in(ctx.ast);
        ctx.state.changed = true;
    }

    /// Compress `a = a + b` to `a += b`
    pub fn try_compress_normal_assignment_to_combined_assignment(
        expr: &mut AssignmentExpression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        if !matches!(expr.operator, AssignmentOperator::Assign) {
            return;
        }
        let Expression::BinaryExpression(binary_expr) = &mut expr.right else { return };
        let Some(new_op) = binary_expr.operator.to_assignment_operator() else { return };
        if !Self::has_no_side_effect_for_evaluation_same_target(&expr.left, &binary_expr.left, ctx)
        {
            return;
        }
        expr.operator = new_op;
        expr.right = binary_expr.right.take_in(ctx.ast);
        ctx.state.changed = true;
    }

    /// Compress `a -= 1` to `--a` and `a -= -1` to `++a`
    #[expect(clippy::float_cmp)]
    pub fn try_compress_assignment_to_update_expression(
        expr: &mut Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) {
        let Expression::AssignmentExpression(e) = expr else { return };
        if !matches!(e.operator, AssignmentOperator::Subtraction) {
            return;
        }
        let operator = if let Expression::NumericLiteral(num) = &e.right {
            if num.value == 1.0 {
                UpdateOperator::Decrement
            } else if num.value == -1.0 {
                UpdateOperator::Increment
            } else {
                return;
            }
        } else {
            return;
        };
        let Some(target) = e.left.as_simple_assignment_target_mut() else { return };
        let target = target.take_in(ctx.ast);
        *expr = ctx.ast.expression_update(e.span, operator, true, target);
        ctx.state.changed = true;
    }
}

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/PeepholeMinimizeConditionsTest.java>
#[cfg(test)]
mod test {
    use crate::{
        CompressOptions,
        tester::{test, test_same, test_same_options},
    };
    use oxc_syntax::es_target::ESTarget;

    /** Check that removing blocks with 1 child works */
    #[test]
    fn test_fold_one_child_blocks() {
        test("function f(){if(x)a();x=3}", "function f(){x&&a(),x=3}");
        test("function f(){if(x)a?.();x=3}", "function f(){x&&a?.(),x=3}");

        test("function f(){if(x){a()}x=3}", "function f(){x&&a(),x=3}");
        test("function f(){if(x){a?.()}x=3}", "function f(){x&&a?.(),x=3}");

        test("function f(){if(x){return 3}}", "function f(){if(x)return 3}");
        test("function f(){if(x){a()}}", "function f(){x&&a()}");
        test("function f(){if(x){throw 1}}", "function f(){if(x)throw 1}");

        // Try it out with functions
        test("function f(){if(x){foo()}}", "function f(){x&&foo()}");
        test("function f(){if(x){foo()}else{bar()}}", "function f(){x?foo():bar()}");

        // Try it out with properties and methods
        test("function f(){if(x){a.b=1}}", "function f(){x&&(a.b=1)}");
        test("function f(){if(x){a.b*=1}}", "function f(){x&&(a.b*=1)}");
        test("function f(){if(x){a.b+=1}}", "function f(){x&&(a.b+=1)}");
        test("function f(){if(x){++a.b}}", "function f(){x&&++a.b}");
        test("function f(){if(x){a.foo()}}", "function f(){x&&a.foo()}");
        test("function f(){if(x){a?.foo()}}", "function f(){x&&a?.foo()}");

        // Try it out with throw/catch/finally [which should not change]
        test_same("function f(){try{foo()}catch(e){bar(e)}finally{baz()}}");

        // Try it out with switch statements
        test_same("function f(){switch(x){case 1:break}}");

        // Do while loops stay in a block if that's where they started
        test(
            "function f(){if(e1){do foo();while(e2)}else foo2()}",
            "function f() { if (e1) do foo(); while (e2); else foo2(); }",
        );
        // Test an obscure case with do and while
        test("if(x){do{foo()}while(y)}else bar()", "if(x)do foo();while(y);else bar()");

        // Play with nested IFs
        test("function f(){if(x){if(y)foo()}}", "function f(){x && (y && foo())}");
        test("function f(){if(x){if(y)foo();else bar()}}", "function f(){x&&(y?foo():bar())}");
        test("function f(){if(x){if(y)foo()}else bar()}", "function f(){x?y&&foo():bar()}");
        test(
            "function f(){if(x){if(y)foo();else bar()}else{baz()}}",
            "function f(){x?y?foo():bar():baz()}",
        );

        test("if(e1){while(e2){if(e3){foo()}}}else{bar()}", "if(e1)for(;e2;)e3&&foo();else bar()");

        test("if(e1){with(e2){if(e3){foo()}}}else{bar()}", "if(e1)with(e2)e3&&foo();else bar()");

        // test("if(a||b){if(c||d){var x;}}", "if(a||b)if(c||d)var x");
        test("if(x){ if(y){var x;}else{var z;} }", "if(x)if(y)var x;else var z");

        // NOTE - technically we can remove the blocks since both the parent
        // and child have elses. But we don't since it causes ambiguities in
        // some cases where not all descendent ifs having elses
        test(
            "if(x){ if(y){var x;}else{var z;} }else{var w}",
            "if(x)if(y)var x;else var z;else var w",
        );
        test("if (x) {var x;}else { if (y) { var y;} }", "if(x)var x;else if(y)var y");

        // Here's some of the ambiguous cases
        test(
            "if(a){if(b){f1();f2();}else if(c){f3();}}else {if(d){f4();}}",
            "a ? b ? (f1(), f2()) : c && f3() : d && f4();",
        );

        test_same("function f(){foo()}");
        test_same("switch(x){case y: foo()}");
        test(
            "try{foo()}catch(ex){bar()}finally{baz()}",
            "try { foo(); } catch { bar(); } finally { baz(); }",
        );

        // Dot not fold `let` and `const`.
        // Lexical declaration cannot appear in a single-statement context.
        test(
            "if (foo) { const bar = 1 } else { const baz = 1 }",
            "if (foo) { let bar = 1 } else { let baz = 1 }",
        );
        test_same("if (foo) { let bar = 1 } else { let baz = 1 }");
        test(
            "if (foo) { var bar = 1 } else { var baz = 1 }",
            "if (foo) var bar = 1; else var baz = 1;",
        );
    }

    #[test]
    fn test_fold_returns() {
        test("function f(){if(x)return 1;else return 2}", "function f(){return x?1:2}");
        test("function f(){if(x)return 1;return 2}", "function f(){return x?1:2}");
        test("function f(){if(x)return;return 2}", "function f(){if (!x) return 2;}");
        test("function f(){if(x)return 1+x;else return 2-x}", "function f(){return x?1+x:2-x}");
        test("function f(){if(x)return 1+x;return 2-x}", "function f(){return x?1+x:2-x}");
        test(
            "function f(){if(x)return y += 1;else return y += 2}",
            "function f(){return x?(y+=1):(y+=2)}",
        );

        test("function f(){if(x)return;else return 2-x}", "function f(){if (!x) return 2 - x;}");
        test("function f(){if(x)return;return 2-x}", "function f(){if (!x) return 2 - x;}");
        test("function f(){if(x)return x;else return}", "function f(){if(x)return x;}");
        test("function f(){if(x)return x;return}", "function f(){if(x)return x}");

        test(
            "function f(){for(var x in y) { return x.y; } return k}",
            "function f() { for (var x in y) return x.y; return k; }",
        );
    }

    #[test]
    fn test_combine_ifs1() {
        test(
            "function f() {if (x) return 1; if (y) return 1}",
            "function f() {if (x || y) return 1;}",
        );
        // test(
        //     "function f() {if (x) return 1; if (y) foo(); else return 1}",
        //     "function f() {if ((!x)&&y) foo(); else return 1;}",
        // );
    }

    #[test]
    fn test_combine_ifs2() {
        test("function f() {if (x) throw 1; if (y) throw 1}", "function f() {if (x || y) throw 1}");
        // Can't combine, side-effect
        test("function f(){ if (x) g(); if (y) g() }", "function f(){ x&&g(), y&&g() }");
        test("function f(){ if (x) g?.(); if (y) g?.() }", "function f(){ x&&g?.(), y&&g?.() }");
        // Can't combine, side-effect
        test("function f(){ if (x) y = 0; if (y) y = 0; }", "function f(){ x&&(y = 0), y &&= 0 }");
    }

    #[test]
    fn test_combine_ifs3() {
        test(
            "function f() {if (x) return 1; if (y) {g();f()}}",
            "function f() { if (x) return 1; y && (g(), f()) }",
        );
    }

    /** Try to minimize assignments */
    #[test]
    #[ignore]
    fn test_fold_assignments() {
        test("function f(){if(x)y=3;else y=4;}", "function f(){y=x?3:4}");
        test("function f(){if(x)y=1+a;else y=2+a;}", "function f(){y=x?1+a:2+a}");

        // and operation assignments
        test("function f(){if(x)y+=1;else y+=2;}", "function f(){y+=x?1:2}");
        test("function f(){if(x)y-=1;else y-=2;}", "function f(){y-=x?1:2}");
        test("function f(){if(x)y%=1;else y%=2;}", "function f(){y%=x?1:2}");
        test("function f(){if(x)y|=1;else y|=2;}", "function f(){y|=x?1:2}");

        // Don't fold if the 2 ops don't match.
        test_same("function f(){x ? y-=1 : y+=2}");

        // Don't fold if the 2 LHS don't match.
        test_same("function f(){x ? y-=1 : z-=1}");

        // Don't fold if there are potential effects.
        test_same("function f(){x ? y().a=3 : y().a=4}");
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_statements() {
        test("if (a) { x = 1; x++ } else { x = 2; x++ }", "x=(a) ? 1 : 2; x++");
        test(
            concat!(
                "if (a) { x = 1; x++; y += 1; z = pi; }",
                " else  { x = 2; x++; y += 1; z = pi; }"
            ),
            "x=(a) ? 1 : 2; x++; y += 1; z = pi;",
        );
        test(
            concat!("function z() {", "if (a) { foo(); return !0 } else { goo(); return !0 }", "}"),
            "function z() {(a) ? foo() : goo(); return !0}",
        );
        test(
            concat!(
                "function z() {if (a) { foo(); x = true; return true ",
                "} else { goo(); x = true; return true }}"
            ),
            "function z() {(a) ? foo() : goo(); x = true; return true}",
        );

        test(
            concat!(
                "function z() {",
                "  if (a) { bar(); foo(); return true }",
                "    else { bar(); goo(); return true }",
                "}"
            ),
            concat!(
                "function z() {",
                "  if (a) { bar(); foo(); }",
                "    else { bar(); goo(); }",
                "  return true;",
                "}"
            ),
        );
    }

    #[test]
    fn test_fold_returns_integration2() {
        // if-then-else duplicate statement removal handles this case:
        test(
            "function test(a) {if (a) {let a = Math.random();if(a) {return a;}} return a; }",
            "function test(a) { if (a) { let a = Math.random(); if (a) return a; } return a; }",
        );
    }

    #[test]
    fn test_dont_remove_duplicate_statements_without_normalization() {
        // In the following test case, we can't remove the duplicate "alert(x);" lines since each "x"
        // refers to a different variable.
        // We only try removing duplicate statements if the AST is normalized and names are unique.
        test(
            "if (Math.random() < 0.5) { let x = 3; alert(x); } else { let x = 5; alert(x); }",
            "if (Math.random() < 0.5) { let x = 3; alert(3); } else { let x = 5; alert(5); }",
        );
    }

    #[test]
    fn test_not_cond() {
        test("function f(){if(!x)foo()}", "function f(){x||foo()}");
        test("function f(){if(!x)b=1}", "function f(){x||(b=1)}");
        test("if(!x)z=1;else if(y)z=2", "x ? y&&(z=2) : z=1;");
        test("if(x)y&&(z=2);else z=1;", "x ? y&&(z=2) : z=1");
        test("function f(){if(!(x=1))a.b=1}", "function f(){(x=1)||(a.b=1)}");
    }

    #[test]
    #[ignore]
    fn test_and_parentheses_count() {
        test("function f(){if(x||y)a.foo()}", "function f(){(x||y)&&a.foo()}");
        test("function f(){if(x.a)x.a=0}", "function f(){x.a&&(x.a=0)}");
        test("function f(){if(x?.a)x.a=0}", "function f(){x?.a&&(x.a=0)}");
        test_same("function f(){if(x()||y()){x()||y()}}");
    }

    #[test]
    fn test_fold_logical_op_string_compare() {
        test("if (foo() && false) z()", "foo()");
    }

    #[test]
    fn test_fold_not() {
        test("for(; !(x==y) ;) a=b", "for(; x!=y ;) a=b");
        test("for(; !(x!=y) ;) a=b", "for(; x==y ;) a=b");
        test("for(; !(x===y) ;) a=b", "for(; x!==y ;) a=b");
        test("for(; !(x!==y) ;) a=b", "for(; x===y ;) a=b");
        // Because !(x<NaN) != x>=NaN don't fold < and > cases.
        test_same("for(; !(x>y) ;) a=b");
        test_same("for(; !(x>=y) ;) a=b");
        test_same("for(; !(x<y) ;) a=b");
        test_same("for(; !(x<=y) ;) a=b");
        test_same("for(; !(x<=NaN) ;) a=b");

        // NOT forces a boolean context
        test("x = !(y() && true)", "x = !y()");

        // This will be further optimized by PeepholeFoldConstants.
        test("x = !true", "x = !1");
    }

    #[test]
    fn test_fold_triple_not() {
        test("!!!foo ? bar : baz", "foo ? baz : bar");
    }

    #[test]
    fn test_minimize_while_condition() {
        // This test uses constant folding logic, so is only here for completeness.
        test("while(!!true) foo()", "for(;;) foo()");
        // These test tryMinimizeCondition
        test("while(!!x) foo()", "for(;x;) foo()");
        // test("while(!(!x&&!y)) foo()", "for(;x||y;) foo()");
        test("while(x||!!y) foo()", "for(;x||y;) foo()");
        // TODO
        // test("while(!(!!x&&y)) foo()", "for(;!x||!y;) foo()");
        // test("while(!(!x&&y)) foo()", "for(;x||!y;) foo()");
        // test("while(!(x||!y)) foo()", "for(;!x&&y;) foo()");
        // test("while(!(x||y)) foo()", "for(;!x&&!y;) foo()");
        // test("while(!(!x||y-z)) foo()", "for(;x&&!(y-z;)) foo()");
        // test("while(!(!(x/y)||z+w)) foo()", "for(;x/y&&!(z+w;)) foo()");
        // test("while(!(x+y||z)) foo()", "for(;!(x+y||z);) foo()");
        // test("while(!(x&&y*z)) foo()", "for(;!(x+y||z);) foo()");
        // test("while(!(!!x&&y)) foo()", "for(;!x||!y;) foo()");
        // test("while(x&&!0) foo()", "for(;x;) foo()");
        // test("while(x||!1) foo()", "for(;x;) foo()");
        // test("while(!((x,y)&&z)) foo()", "for(;(x,!y)||!z;) foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan_remove_leading_not() {
        test("if(!(!a||!b)&&c) foo()", "((a&&b)&&c)&&foo()");
        test("if(!(x&&y)) foo()", "x&&y||foo()");
        test("if(!(x||y)) foo()", "(x||y)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan1() {
        test("if(!a&&!b)foo()", "(a||b)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan2() {
        // Make sure trees with cloned functions are marked as changed
        test("(!(a&&!((function(){})())))||foo()", "!a||(function(){})()||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan2b() {
        // Make sure unchanged trees with functions are not marked as changed
        test_same("!a||(function(){})()||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan3() {
        test("if((!a||!b)&&(c||d)) foo()", "(a&&b||!c&&!d)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan5() {
        test("if((!a||!b)&&c) foo()", "(a&&b||!c)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan11() {
        test(
            "if (x && (y===2 || !f()) && (y===3 || !h())) foo()",
            "(!x || y!==2 && f() || y!==3 && h()) || foo()",
        );
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan20a() {
        test(
            "if (0===c && (2===a || 1===a)) f(); else g()",
            "if (0!==c || 2!==a && 1!==a) g(); else f()",
        );
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan20b() {
        test("if (0!==c || 2!==a && 1!==a) g(); else f()", "(0!==c || 2!==a && 1!==a) ? g() : f()");
    }

    #[test]
    fn test_preserve_if() {
        test_same("if(!a&&!b)for(;f(););");
    }

    #[test]
    fn test_dangling_else() {
        test(
            "
        if (!x) {
          for (;;) foo();
          for (;;) bar();
        } else if (y) for (;;) f();",
            "
        if (x) {
          if (y) for (; ; ) f();
        } else {
          for (; ; ) foo();
          for (; ; ) bar();}",
        );
        test_same("if(!a&&!b) {for(;;)foo(); for(;;)bar()} else if(y) for(;;) f()");
    }

    #[test]
    fn test_minimize_hook() {
        test("x ? x : y", "x || y");
        test_same("x.y ? x.y : x.z");
        test_same("x?.y ? x?.y : x.z");
        test_same("x?.y ? x?.y : x?.z");

        test_same("x() ? x() : y()");
        test_same("x?.() ? x?.() : y()");

        test("!x ? foo() : bar()", "x ? bar() : foo()");
        // TODO
        // test("while(!(x ? y : z)) foo();", "while(x ? !y : !z) foo();");
        // test("(x ? !y : !z) ? foo() : bar()", "(x ? y : z) ? bar() : foo()");
    }

    #[test]
    fn test_minimize_comma() {
        test("while(!(inc(), test())) foo();", "for(;inc(), !test();) foo();");
        test("(inc(), !test()) ? foo() : bar()", "inc(), test() ? bar() : foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_expr_result() {
        test("!x||!y", "x&&y");
        test("if(!(x&&!y)) foo()", "(!x||y)&&foo()");
        test("if(!x||y) foo()", "(!x||y)&&foo()");
        test("(!x||y)&&foo()", "x&&!y||!foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan21() {
        test("if (0===c && (2===a || 1===a)) f()", "(0!==c || 2!==a && 1!==a) || f()");
    }

    #[test]
    #[ignore]
    fn test_minimize_and_or1() {
        test("if ((!a || !b) && (d || e)) f()", "(a&&b || !d&&!e) || f()");
    }

    #[test]
    fn test_minimize_for_condition() {
        // This test uses constant folding logic, so is only here for completeness.
        // These could be simplified to "for(;;) ..."
        test("for(;!!true;) foo()", "for(;;) foo()");
        // Verify function deletion tracking.
        test("if(!!true||function(){}) {}", "");
        // Don't bother with FOR inits as there are normalized out.
        test("for(!!true;;) foo()", "for(;;) foo()");

        // These test tryMinimizeCondition
        test("for(;!!x;) foo()", "for(;x;) foo()");

        test_same("for(a in b) foo()");
        test_same("for(a in {}) foo()");
        test_same("for(a in []) foo()");
        test("for(a in !!true) foo()", "for(a in !0) foo()");

        test_same("for(a of b) foo()");
        test_same("for(a of {}) foo()");
        test_same("for(a of []) foo()");
        test("for(a of !!true) foo()", "for(a of !0) foo()");
    }

    #[test]
    fn test_minimize_condition_example1() {
        // Based on a real failing code sample.
        test("if(!!(f() > 20)) {foo();foo()}", "f() > 20 && (foo(), foo())");
    }

    #[test]
    fn test_fold_loop_break() {
        test("for(;;) if (a) break", "for(;!a;);");
        test_same("for(;;) if (a) { f(); break }");
        test("for(;;) if (a) break; else f()", "for(;!a;) f()");
        test("for(;a;) if (b) break", "for(;a && !b;);");
        test("for(;a;) { if (b) break; if (c) break; }", "for (;a && !(b || c););");
        test("for(;(a && !b);) if (c) break;", "for(;(a && !b) && !c;);");
        // test("for(;;) { if (foo) { break; var x; } } x;", "var x; for(;!foo;) {} x;");

        // 'while' is normalized to 'for'
        test("while(true) if (a) break", "for(;!a;);");
    }

    #[test]
    #[ignore]
    fn test_fold_conditional_var_declaration() {
        test("if(x) var y=1;else y=2", "var y=x?1:2");
        test("if(x) y=1;else var y=2", "var y=x?1:2");

        test_same("if(x) var y = 1; z = 2");
        test_same("if(x||y) y = 1; var z = 2");

        test_same("if(x) { var y = 1; print(y)} else y = 2 ");
        test_same("if(x) var y = 1; else {y = 2; print(y)}");
    }

    #[test]
    fn test_fold_if_with_lower_operators_inside() {
        test("if (x + (y=5)) z && (w,z);", "x + (y=5) && (z && (w,z))");
        test("if (!(x+(y=5))) z && (w,z);", "x + (y=5) || z && (w,z)");
        test(
            "if (x + (y=5)) if (z && (w,z)) for(;;) foo();",
            "if (x + (y=5) && (z && (w,z))) for(;;) foo();",
        );
    }

    #[test]
    #[ignore]
    fn test_substitute_return() {
        test("function f() { while(x) { return }}", "function f() { while(x) { break }}");

        test_same("function f() { while(x) { return 5 } }");

        test_same("function f() { a: { return 5 } }");

        test(
            "function f() { while(x) { return 5}  return 5}",
            "function f() { while(x) { break }    return 5}",
        );

        test(
            "function f() { while(x) { return x}  return x}",
            "function f() { while(x) { break }    return x}",
        );

        test(
            "function f() { while(x) { if (y) { return }}}",
            "function f() { while(x) { if (y) { break  }}}",
        );

        test(
            "function f() { while(x) { if (y) { return }} return}",
            "function f() { while(x) { if (y) { break  }}}",
        );

        test(
            "function f() { while(x) { if (y) { return 5 }} return 5}",
            "function f() { while(x) { if (y) { break    }} return 5}",
        );

        // It doesn't matter if x is changed between them. We are still returning
        // x at whatever x value current holds. The whole x = 1 is skipped.
        test(
            "function f() { while(x) { if (y) { return x } x = 1} return x}",
            "function f() { while(x) { if (y) { break    } x = 1} return x}",
        );

        test(
            "function f() { while(x) { if (y) { return x } return x} return x}",
            "function f() { while(x) { if (y) {} break }return x}",
        );

        // A break here only breaks out of the inner loop.
        test_same("function f() { while(x) { while (y) { return } } }");

        test_same("function f() { while(1) { return 7}  return 5}");

        test_same(concat!(
            "function f() {",
            "  try { while(x) {return f()}} catch (e) { } return f()}"
        ));

        test_same(concat!(
            "function f() {",
            "  try { while(x) {return f()}} finally {alert(1)} return f()}"
        ));

        // Both returns has the same handler
        test(
            concat!(
                "function f() {",
                "  try { while(x) { return f() } return f() } catch (e) { } }"
            ),
            concat!("function f() {", "  try { while(x) { break } return f() } catch (e) { } }"),
        );

        // We can't fold this because it'll change the order of when foo is called.
        test_same(concat!(
            "function f() {",
            "  try { while(x) { return foo() } } finally { alert(1) } ",
            "  return foo()}"
        ));

        // This is fine, we have no side effect in the return value.
        test(
            concat!(
                "function f() {",
                "  try { while(x) { return 1 } } finally { alert(1) } return 1}"
            ),
            concat!(
                "function f() {",
                "  try { while(x) { break    } } finally { alert(1) } return 1}"
            ),
        );

        test_same("function f() { try{ return a } finally { a = 2 } return a; }");

        test(
            "function f() { switch(a){ case 1: return a; default: g();} return a;}",
            "function f() { switch(a){ case 1: break; default: g();} return a; }",
        );
    }

    #[test]
    #[ignore]
    fn test_substitute_break_for_throw() {
        test_same("function f() { while(x) { throw Error }}");

        test(
            "function f() { while(x) { throw Error } throw Error }",
            "function f() { while(x) { break } throw Error}",
        );
        test_same("function f() { while(x) { throw Error(1) } throw Error(2)}");
        test_same("function f() { while(x) { throw Error(1) } return Error(2)}");

        test_same("function f() { while(x) { throw 5 } }");

        test_same("function f() { a: { throw 5 } }");

        test(
            "function f() { while(x) { throw 5}  throw 5}",
            "function f() { while(x) { break }   throw 5}",
        );

        test(
            "function f() { while(x) { throw x}  throw x}",
            "function f() { while(x) { break }   throw x}",
        );

        test_same("function f() { while(x) { if (y) { throw Error }}}");

        test(
            "function f() { while(x) { if (y) { throw Error }} throw Error}",
            "function f() { while(x) { if (y) { break }} throw Error}",
        );

        test(
            "function f() { while(x) { if (y) { throw 5 }} throw 5}",
            "function f() { while(x) { if (y) { break    }} throw 5}",
        );

        // It doesn't matter if x is changed between them. We are still throwing
        // x at whatever x value current holds. The whole x = 1 is skipped.
        test(
            "function f() { while(x) { if (y) { throw x } x = 1} throw x}",
            "function f() { while(x) { if (y) { break    } x = 1} throw x}",
        );

        test(
            "function f() { while(x) { if (y) { throw x } throw x} throw x}",
            "function f() { while(x) { if (y) {} break }throw x}",
        );

        // A break here only breaks out of the inner loop.
        test_same("function f() { while(x) { while (y) { throw Error } } }");

        test_same("function f() { while(1) { throw 7}  throw 5}");

        test_same(concat!(
            "function f() {",
            "  try { while(x) {throw f()}} catch (e) { } throw f()}"
        ));

        test_same(concat!(
            "function f() {",
            "  try { while(x) {throw f()}} finally {alert(1)} throw f()}"
        ));

        // Both throws has the same handler
        test(
            concat!("function f() {", "  try { while(x) { throw f() } throw f() } catch (e) { } }"),
            concat!("function f() {", "  try { while(x) { break } throw f() } catch (e) { } }"),
        );

        // We can't fold this because it'll change the order of when foo is called.
        test_same(concat!(
            "function f() {",
            "  try { while(x) { throw foo() } } finally { alert(1) } ",
            "  throw foo()}"
        ));

        // This is fine, we have no side effect in the throw value.
        test(
            concat!(
                "function f() {",
                "  try { while(x) { throw 1 } } finally { alert(1) } throw 1}"
            ),
            concat!(
                "function f() {",
                "  try { while(x) { break    } } finally { alert(1) } throw 1}"
            ),
        );

        test_same("function f() { try{ throw a } finally { a = 2 } throw a; }");

        test(
            "function f() { switch(a){ case 1: throw a; default: g();} throw a;}",
            "function f() { switch(a){ case 1: break; default: g();} throw a; }",
        );
    }

    #[test]
    fn test_remove_duplicate_return() {
        test("function f() { return; }", "function f(){}");
        test_same("function f() { return a; }");
        test("function f() { if (x) { return a } return a; }", "function f() { return x, a; }");
        test_same("function f() { try { if (x) return a; } catch {} return a; }");
        test(
            "function f() { try { if (x) {} } catch {} return 1; }",
            "function f() { try { x } catch {} return 1; }",
        );

        // finally clauses may have side effects
        test_same("function f() { try { if (x) return a } finally { a++ } return a; }");
        // but they don't matter if the result doesn't have side effects and can't
        // be affect by side-effects.
        // test(
        //     "function f() { try { if (x) return 1 } finally {} return 1; }",
        //     "function f() { try { if (x) {} } finally {} return 1; }",
        // );

        // test(
        //     "function f() { switch(a){ case 1: return a; } return a; }",
        //     "function f() { switch(a){ case 1: } return a; }",
        // );

        // test(
        //     concat!(
        //         "function f() { switch(a){ ",
        //         "  case 1: return a; case 2: return a; } return a; }"
        //     ),
        //     concat!("function f() { switch(a){ ", "  case 1: break; case 2: } return a; }"),
        // );
    }

    #[test]
    fn test_remove_duplicate_throw() {
        test_same("function f() { throw a; }");
        test("function f() { if (x) { throw a } throw a; }", "function f() { throw x, a; }");
        test_same("function f() { try { if (x) throw a } catch {} throw a; }");
        test_same("function f() { try { if (x) throw 1 } catch {f()} throw 1; }");
        test_same("function f() { try { if (x) throw 1 } catch {f()} throw 1; }");
        test_same("function f() { try { if (x) throw 1 } catch {throw 1}}");
        // test(
        //     "function f() { try { if (x) throw 1 } catch {throw 1} throw 1; }",
        //     "function f() { try { if (x) throw 1 } catch {} throw 1; }",
        // );

        // finally clauses may have side effects
        test_same("function f() { try { if (x) throw a } finally { a++ } throw a; }");
        // but they don't matter if the result doesn't have side effects and can't
        // be affect by side-effects.
        // test(
        //     "function f() { try { if (x) throw 1 } finally {} throw 1; }",
        //     "function f() { try { if (x) {} } finally {} throw 1; }",
        // );

        // test(
        //     "function f() { switch(a){ case 1: throw a; } throw a; }",
        //     "function f() { switch(a){ case 1: } throw a; }",
        // );

        // test(
        //     concat!("function f() { switch(a){ ", "case 1: throw a; case 2: throw a; } throw a; }"),
        //     concat!("function f() { switch(a){ case 1: break; case 2: } throw a; }"),
        // );
    }

    #[test]
    fn test_nested_if_combine() {
        test("if(x)if(y){for(;;);}", "if(x&&y) for(;;);");
        test("if(x||z)if(y){for(;;);}", "if((x||z)&&y) for(;;);");
        test("if(x)if(y||z){for(;;);}", "if((x)&&(y||z)) for(;;);");
        test("if(x||z)if(y||z){for(;;);}", "if((x||z)&&(y||z)) for(;;);"); // TODO: `if(x||z)if(y||z)for(;;);` is shorter
        test("if(x)if(y){if(z){for(;;);}}", "if(x&&(y&&z)) for(;;);");
    }

    #[test]
    fn test_remove_else_cause() {
        test(
            concat!(
                "function f() {",
                " if(x) return 1;",
                " else if(x) return 2;",
                " else if(x) return 3 }"
            ),
            concat!(
                "function f() {", //
                " if(x) return 1;",
                " if(x) return 2;",
                " if(x) return 3 }"
            ),
        );
    }

    #[test]
    fn test_remove_else_cause1() {
        test("function f() { if (x) throw 1; else f() }", "function f() { if (x) throw 1; f() }");
    }

    #[test]
    fn test_remove_else_cause2() {
        test("function f() { if (x) return 1; else f() }", "function f() { if (x) return 1; f() }");
        test("function f() { if (x) return; else f() }", "function f() { x || f() }");
        // This case is handled by minimize exit points.
        test("function f() { if (x) return; f() }", "function f() { x || f() }");
    }

    #[test]
    fn test_remove_else_cause3() {
        test(
            "function f() { a: { if (x) break a; else f() } }",
            "function f() { a: { if (x) break a; f() } }",
        );
        test("function f() { if (x) { a:{ break a } } else f() }", "function f() { x || f() }");
        test("function f() { if (x) a:{ break a } else f() }", "function f() { x || f() }");
    }

    #[test]
    fn test_remove_else_cause4() {
        test(
            "function f() { if (x) { if (y) { return 1; } } else f() }",
            "function f() { if (x) { if (y) return 1; } else f() }",
        );
    }

    /// https://blickly.github.io/closure-compiler-issues/#925
    #[test]
    fn test_issue925() {
        test(
            concat!(
                "if (x[--y] === 1) {\n",
                "    x[y] = 0;\n",
                "} else {\n",
                "    x[y] = 1;\n",
                "}"
            ),
            "(x[--y] === 1) ? x[y] = 0 : x[y] = 1;",
        );

        test(
            concat!("if (x[--y]) {\n", "    a = 0;\n", "} else {\n", "    a = 1;\n", "}"),
            "a = (x[--y]) ? 0 : 1;",
        );

        test(
            concat!(
                "if (x?.[--y]) {", //
                "    a = 0;",
                "} else {",
                "    a = 1;",
                "}",
            ),
            "a = (x?.[--y]) ? 0 : 1;",
        );

        test("if (x++) { x += 2 } else { x += 3 }", "x++ ? x += 2 : x += 3");
        test("if (x++) { x = x + 2 } else { x = x + 3 }", "x++ ? x += 2 : x += 3");
    }

    #[test]
    fn test_coercion_substitution_disabled() {
        test_same("var x = {}; if (x != null) throw 'a';");
        test("var x = {}; var y = x != null;", "var x = {}, y = x != null;");

        test_same("var x = 1; if (x != 0) throw 'a';");
        test("var x = 1; var y = x != 0;", "var x = 1, y = x != 0;");
    }

    #[test]
    fn test_coercion_substitution_boolean_result0() {
        test("var x = {}; var y = x != null;", "var x = {}, y = x != null");
    }

    #[test]
    fn test_coercion_substitution_boolean_result1() {
        test("var x = {}; var y = x == null;", "var x = {}, y = x == null;");
        test("var x = {}; var y = x !== null;", "var x = {}, y = x !== null;");
        test("var x = undefined; var y = x !== null;", "var x = void 0, y = x !== null;");
        test("var x = {}; var y = x === null;", "var x = {}, y = x === null;");
        test("var x = undefined; var y = x === null;", "var x = void 0, y = x === null;");

        test("var x = 1; var y = x != 0;", "var x = 1, y = x != 0;");
        test("var x = 1; var y = x == 0;", "var x = 1, y = x == 0;");
        test("var x = 1; var y = x !== 0;", "var x = 1, y = x !== 0;");
        test("var x = 1; var y = x === 0;", "var x = 1, y = x === 0;");
    }

    #[test]
    fn test_coercion_substitution_if() {
        test("var x = {};\nif (x != null) throw 'a';\n", "var x={}; if (x!=null) throw 'a'");
        test_same("var x = {};\nif (x == null) throw 'a';\n");
        test_same("var x = {};\nif (x != null) throw 'a';\n");
        test_same("var x = {};\nif (x !== null) throw 'a';\n");
        test_same("var x = {};\nif (x === null) throw 'a';\n");

        test_same("var x = 1;\nif (x != 0) throw 'a';\n");
        test_same("var x = 1;\nif (x != 0) throw 'a';\n");
        test_same("var x = 1;\nif (x == 0) throw 'a';\n");
        test_same("var x = 1;\nif (x !== 0) throw 'a';\n");
        test_same("var x = 1;\nif (x === 0) throw 'a';\n");
        test_same("var x = NaN;\nif (x === 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_expression() {
        test_same("var x = {}; x != null && alert('b');");
        test_same("var x = 1; x != 0 && alert('b');");
    }

    #[test]
    fn test_coercion_substitution_hook() {
        test("var x = {}; var y = x != null ? 1 : 2;", "var x = {}, y = x == null ? 2 : 1;");
        test("var x = 1; var y = x != 0 ? 1 : 2;", "var x = 1, y = x == 0 ? 2 : 1;");
    }

    #[test]
    fn test_coercion_substitution_not() {
        test("var x = {}; var y = !(x != null) ? 1 : 2;", "var x = {}, y = x == null ? 1 : 2;");
        test("var x = 1; var y = !(x != 0) ? 1 : 2; ", "var x = 1, y = x == 0 ? 1 : 2; ");
    }

    #[test]
    fn test_coercion_substitution_while() {
        test("var x = {}; while (x != null) throw 'a';", "for (var x = {} ;x != null;) throw 'a';");
        test("var x = 1; while (x != 0) throw 'a';", "for (var x = 1; x != 0;) throw 'a';");
    }

    #[test]
    fn test_coercion_substitution_unknown_type() {
        test_same("var x = /** @type {?} */ ({});\nif (x != null) throw 'a';\n");
        test_same("var x = /** @type {?} */ (1);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_all_type() {
        test_same("var x = /** @type {*} */ ({});\nif (x != null) throw 'a';\n");
        test_same("var x = /** @type {*} */ (1);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_primitives_vs_null() {
        test_same("var x = 0;\nif (x != null) throw 'a';\n");
        test_same("var x = '';\nif (x != null) throw 'a';\n");
        test_same("var x = !1;\nif (x != null) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_non_number_vs_zero() {
        test_same("var x = {};\nif (x != 0) throw 'a';\n");
        test_same("var x = '';\nif (x != 0) throw 'a';\n");
        test_same("var x = !1;\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_boxed_number_vs_zero() {
        test_same("var x = /* @__PURE__ */ new Number(0);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    fn test_coercion_substitution_boxed_primitives() {
        test_same("var x = /* @__PURE__ */ new Number(); if (x != null) throw 'a';");
        test_same("var x = /* @__PURE__ */ new String(); if (x != null) throw 'a';");
        test_same("var x = /* @__PURE__ */ new Boolean();\nif (x != null) throw 'a';");
    }

    #[test]
    fn test_minimize_if_with_new_target_condition() {
        // Related to https://github.com/google/closure-compiler/issues/3097
        test(
            concat!(
                "function x() {",
                "  if (new.target) {",
                "    return 1;",
                "  } else {",
                "    return 2;",
                "  }",
                "}",
            ),
            concat!("function x() {", "  return new.target ? 1 : 2;", "}"),
        );
    }

    #[test]
    fn compress_binary_boolean() {
        test("a instanceof b === true", "a instanceof b");
        test("a instanceof b == true", "a instanceof b");
        test("a instanceof b === false", "a instanceof b");
        test("a instanceof b == false", "a instanceof b");

        test("a instanceof b !== true", "a instanceof b");
        test("a instanceof b != true", "a instanceof b");
        test("a instanceof b !== false", "a instanceof b");
        test("a instanceof b != false", "a instanceof b");

        test("delete x === true", "delete x");
        test("delete x == true", "delete x");
        test("delete x === false", "delete x");
        test("delete x == false", "delete x");

        test("delete x !== true", "delete x");
        test("delete x != true", "delete x");
        test("delete x !== false", "delete x");
        test("delete x != false", "delete x");
    }

    #[test]
    fn compress_binary_number() {
        test("if(x >> +y == 0){}", "x >> +y");
        test("if(x >> +y === 0){}", "x >> +y");
        test("if(x >> +y != 0){}", "x >> +y");
        test("if(x >> +y !== 0){}", "x >> +y");
        test("if((-0 != +0) !== false){}", "");
        test_same("foo(x >> y == 0)");

        test("v = (x = 1) === 1", "v = (x = 1) == 1");
        test("v = (x = 1) !== 1", "v = (x = 1) != 1");
        test("v = !0 + null !== 1", "v = !1");
    }

    #[test]
    fn test_try_compress_type_of_equal_string() {
        test("v = typeof foo === 'number'", "v = typeof foo == 'number'");
        test("v = 'number' === typeof foo", "v = typeof foo == 'number'");
        test("v = typeof foo === `number`", "v = typeof foo == 'number'");
        test("v = `number` === typeof foo", "v = typeof foo == 'number'");
        test("v = typeof foo !== 'number'", "v = typeof foo != 'number'");
        test("v = 'number' !== typeof foo", "v = typeof foo != 'number'");
        test("v = typeof foo !== `number`", "v = typeof foo != 'number'");
        test("v = `number` !== typeof foo", "v = typeof foo != 'number'");
    }

    #[test]
    fn test_negate_empty_if_stmt_consequent() {
        test("if (x) {} else { foo }", "x || foo");
        test("if (x) ;else { foo }", "x || foo");
        test("if (x) {;} else { foo }", "x || foo");

        test("if (x) { var foo } else { bar }", "if (x) var foo; else bar");
        test("if (x) foo; else { var bar }", "if (x) foo; else var bar");
    }

    #[test]
    fn test_compress_conditional_expression_inside() {
        test("x ? a = 0 : a = 1", "a = x ? 0 : 1");
        test(
            "x ? a = function foo() { return 'a' } : a = function bar() { return 'b' }",
            "a = x ? function () { return 'a' } : function () { return 'b' }",
        );

        // a.b might have a side effect
        test_same("x ? a.b = 0 : a.b = 1");
        // `a = x ? () => 'a' : () => 'b'` does not set the name property of the function, but we ignore that difference
        test("x ? a = () => 'a' : a = () => 'b'", "a = x ? () => 'a' : () => 'b'");

        // for non `=` operators, `GetValue(lref)` is called before `Evaluation of AssignmentExpression`
        // so cannot be fold to `a += x ? 0 : 1`
        // example case: `(()=>{"use strict"; (console.log("log"), 1) ? a += 0 : a += 1; })()`
        test_same("x ? a += 0 : a += 1");
        test_same("x ? a &&= 0 : a &&= 1");
    }

    #[test]
    fn test_fold_is_null_or_undefined() {
        test("v = foo === null || foo === undefined", "v = foo == null");
        test("v = foo === undefined || foo === null", "v = foo == null");
        test("v = foo === null || foo === void 0", "v = foo == null");
        test("v = foo === null || foo === void 0 || foo === 1", "v = foo == null || foo === 1");
        test("v = foo === 1 || foo === null || foo === void 0", "v = foo === 1 || foo == null");
        test_same("v = foo === void 0 || bar === null");
        test_same("var undefined = 1; v = foo === null || foo === undefined");
        test_same("v = foo !== 1 && foo === void 0 || foo === null");
        test_same("v = foo.a === void 0 || foo.a === null"); // cannot be folded because accessing foo.a might have a side effect

        test("v = foo !== null && foo !== undefined", "v = foo != null");
        test("v = foo !== undefined && foo !== null", "v = foo != null");
        test("v = foo !== null && foo !== void 0", "v = foo != null");
        test("v = foo !== null && foo !== void 0 && foo !== 1", "v = foo != null && foo !== 1");
        test("v = foo !== 1 && foo !== null && foo !== void 0", "v = foo !== 1 && foo != null");
        test("v = foo !== 1 || foo !== void 0 && foo !== null", "v = foo !== 1 || foo != null");
        test_same("v = foo !== void 0 && bar !== null");

        test("v = (_foo = foo) === null || _foo === undefined", "v = (_foo = foo) == null");
        test("v = (_foo = foo) === null || _foo === void 0", "v = (_foo = foo) == null");
        test("v = (_foo = foo.bar) === null || _foo === undefined", "v = (_foo = foo.bar) == null");
        test("v = (_foo = foo) !== null && _foo !== undefined", "v = (_foo = foo) != null");
        test("v = (_foo = foo) === undefined || _foo === null", "v = (_foo = foo) == null");
        test("v = (_foo = foo) === void 0 || _foo === null", "v = (_foo = foo) == null");
        test(
            "v = (_foo = foo) === null || _foo === void 0 || _foo === 1",
            "v = (_foo = foo) == null || _foo === 1",
        );
        test(
            "v = _foo === 1 || (_foo = foo) === null || _foo === void 0",
            "v = _foo === 1 || (_foo = foo) == null",
        );
        test_same("v = (_foo = foo) === void 0 || bar === null");
    }

    #[test]
    fn test_fold_logical_expression_to_assignment_expression() {
        test("x || (x = 3)", "x ||= 3");
        test("x && (x = 3)", "x &&= 3");
        test("x ?? (x = 3)", "x ??= 3");
        test("x || (x = g())", "x ||= g()");
        test("x && (x = g())", "x &&= g()");
        test("x ?? (x = g())", "x ??= g()");

        // `||=`, `&&=`, `??=` sets the name property of the function, but we ignore that difference
        // Example case: `let f = false; f || (f = () => {}); console.log(f.name)`
        test("x || (x = () => 'a')", "x ||= () => 'a'");

        test_same("x || (y = 3)");
        test_same("var x; x.y || (x.z = 3)");
        test_same("function _() { this.x || (this.y = 3) }");

        // GetValue(x) has no sideeffect when x is a resolved identifier
        test("var x; x.y || (x.y = 3)", "var x; x.y ||= 3");
        test("var x; x['y'] || (x['y'] = 3)", "var x; x.y ||= 3");
        test("var x; x[0] || (x[0] = 3)", "var x; x[0] ||= 3");
        test("var x; x.#y || (x.#y = 3)", "var x; x.#y ||= 3");
        test("function _() { this.x || (this.x = 3) }", "function _() { this.x ||= 3 }");
        test_same("x.y || (x.y = 3)");
        // this can be compressed if `y` does not have side effect
        test_same("var x; x[y] || (x[y] = 3)");
        // GetValue(x) has a side effect in this case
        // Example case: `var a = { get b() { console.log('b'); return { get c() { console.log('c') } } } }; a.b.c || (a.b.c = 1)`
        test_same("var x; x.y.z || (x.y.z = 3)");
        // This case is not supported, since the minifier does not support with statements
        // test_same("var x; with (z) { x.y || (x.y = 3) }");

        // foo() might have a side effect
        test_same("foo().a || (foo().a = 3)");

        let target = ESTarget::ES2019;
        let options = CompressOptions { target, ..CompressOptions::default() };
        test_same_options("x || (x = 3)", &options);

        test("x || (a, x = 3)", "x ||= (a, 3)");
        test("x && (a, x = 3)", "x &&= (a, 3)");
        test("x ?? (a, x = 3)", "x ??= (a, 3)");
        test("x || (a, x = g())", "x ||= (a, g())");
        test("x && (a, x = g())", "x &&= (a, g())");
        test("x ?? (a, x = g())", "x ??= (a, g())");
        test("var x; x.y || (a, x.y = 3)", "var x; x.y ||= (a, 3)");
        test("var x; x.y && (a, x.y = 3)", "var x; x.y &&= (a, 3)");
        test("var x; x.y ?? (a, x.y = 3)", "var x; x.y ??= (a, 3)");
        // x may have a sideeffect
        // Example case: `var a; Object.defineProperty(globalThis, 'x', { get() { console.log('x'); return {} } }); x.y || (a, x.y = 3);`
        test_same("x.y || (a, x.y = 3)");
        test_same("x.y && (a, x.y = 3)");
        test_same("x.y ?? (a, x.y = 3)");
    }

    #[test]
    fn test_compress_normal_assignment_to_combined_logical_assignment() {
        test("var x; x = x || 1", "var x; x ||= 1");
        test("var x; x = x && 1", "var x; x &&= 1");
        test("var x; x = x ?? 1", "var x; x ??= 1");

        // `x` is a global reference and might have a setter
        // Example case: `Object.defineProperty(globalThis, 'x', { get: () => true, set: () => console.log('x') }); x = x || 1`
        test_same("x = x || 1");
        // setting x.y might have a side effect
        test_same("var x; x.y = x.y || 1");
        // This case is not supported, since the minifier does not support with statements
        // test_same("var x; with (z) { x = x || 1 }");

        // `||=`, `&&=`, `??=` sets the name property of the function, while `= x || y` does not
        // but we ignore that difference
        // Example case: `let f = false; f = f || (() => {}); console.log(f.name)`
        test("var x; x = x || (() => 'a')", "var x; x ||= (() => 'a')");

        let target = ESTarget::ES2019;
        let options = CompressOptions { target, ..CompressOptions::default() };
        test_same_options("var x; x = x || 1", &options);
    }

    #[test]
    fn test_compress_is_loose_boolean() {
        test("v = x == true", "v = x == 1");
        test("v = x != true", "v = x != 1");
        test("v = x == false", "v = x == 0");
        test("v = x != false", "v = x != 0");
        test("v = x == !0", "v = x == 1");
        test("v = x != !0", "v = x != 1");
        test("v = x == !1", "v = x == 0");
        test("v = x != !1", "v = x != 0");
    }

    #[test]
    fn try_minimize_binary() {
        test("f(!a === !0)", "f(!a)");
        test("f(!a === !1)", "f(!!a)");
        test("f(!a === true)", "f(!a)");
        test("f(!a === false)", "f(!!a)");
    }
}
