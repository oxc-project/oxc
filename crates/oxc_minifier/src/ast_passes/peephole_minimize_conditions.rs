use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::CompressorPass;

/// Minimize Conditions
///
/// A peephole optimization that minimizes conditional expressions according to De Morgan's laws.
/// Also rewrites conditional statements as expressions by replacing them
/// with `? :` and short-circuit binary operators.
///
/// <https://github.com/google/closure-compiler/blob/master/src/com/google/javascript/jscomp/PeepholeMinimizeConditions.java>
pub struct PeepholeMinimizeConditions {
    changed: bool,
}

impl<'a> CompressorPass<'a> for PeepholeMinimizeConditions {
    fn changed(&self) -> bool {
        self.changed
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.changed = false;
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for PeepholeMinimizeConditions {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(folded_expr) = match expr {
            Expression::UnaryExpression(e) if e.operator.is_not() => Self::try_minimize_not(e, ctx),
            _ => None,
        } {
            *expr = folded_expr;
            self.changed = true;
        };
    }
}

impl<'a> PeepholeMinimizeConditions {
    pub fn new() -> Self {
        Self { changed: false }
    }

    /// Try to minimize NOT nodes such as `!(x==y)`.
    fn try_minimize_not(
        expr: &mut UnaryExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        debug_assert!(expr.operator.is_not());
        if let Expression::BinaryExpression(binary_expr) = &mut expr.argument {
            if let Some(new_op) = binary_expr.operator.equality_inverse_operator() {
                binary_expr.operator = new_op;
                return Some(ctx.ast.move_expression(&mut expr.argument));
            }
        }
        None
    }
}

/// <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeMinimizeConditionsTest.java>
#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn test(source_text: &str, positive: &str) {
        let allocator = Allocator::default();
        let mut pass = super::PeepholeMinimizeConditions::new();
        tester::test(&allocator, source_text, positive, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    fn fold_same(js: &str) {
        test_same(js);
    }

    fn fold(js: &str, expected: &str) {
        test(js, expected);
    }

    /** Check that removing blocks with 1 child works */
    #[test]
    #[ignore]
    fn test_fold_one_child_blocks() {
        // late = false;
        fold("function f(){if(x)a();x=3}", "function f(){x&&a();x=3}");
        fold("function f(){if(x)a?.();x=3}", "function f(){x&&a?.();x=3}");

        fold("function f(){if(x){a()}x=3}", "function f(){x&&a();x=3}");
        fold("function f(){if(x){a?.()}x=3}", "function f(){x&&a?.();x=3}");

        fold("function f(){if(x){return 3}}", "function f(){if(x)return 3}");
        fold("function f(){if(x){a()}}", "function f(){x&&a()}");
        fold("function f(){if(x){throw 1}}", "function f(){if(x)throw 1;}");

        // Try it out with functions
        fold("function f(){if(x){foo()}}", "function f(){x&&foo()}");
        fold("function f(){if(x){foo()}else{bar()}}", "function f(){x?foo():bar()}");

        // Try it out with properties and methods
        fold("function f(){if(x){a.b=1}}", "function f(){if(x)a.b=1}");
        fold("function f(){if(x){a.b*=1}}", "function f(){x&&(a.b*=1)}");
        fold("function f(){if(x){a.b+=1}}", "function f(){x&&(a.b+=1)}");
        fold("function f(){if(x){++a.b}}", "function f(){x&&++a.b}");
        fold("function f(){if(x){a.foo()}}", "function f(){x&&a.foo()}");
        fold("function f(){if(x){a?.foo()}}", "function f(){x&&a?.foo()}");

        // Try it out with throw/catch/finally [which should not change]
        fold_same("function f(){try{foo()}catch(e){bar(e)}finally{baz()}}");

        // Try it out with switch statements
        fold_same("function f(){switch(x){case 1:break}}");

        // Do while loops stay in a block if that's where they started
        fold_same("function f(){if(e1){do foo();while(e2)}else foo2()}");
        // Test an obscure case with do and while
        fold("if(x){do{foo()}while(y)}else bar()", "if(x){do foo();while(y)}else bar()");

        // Play with nested IFs
        fold("function f(){if(x){if(y)foo()}}", "function f(){x && (y && foo())}");
        fold("function f(){if(x){if(y)foo();else bar()}}", "function f(){x&&(y?foo():bar())}");
        fold("function f(){if(x){if(y)foo()}else bar()}", "function f(){x?y&&foo():bar()}");
        fold(
            "function f(){if(x){if(y)foo();else bar()}else{baz()}}",
            "function f(){x?y?foo():bar():baz()}",
        );

        fold("if(e1){while(e2){if(e3){foo()}}}else{bar()}", "if(e1)while(e2)e3&&foo();else bar()");

        fold("if(e1){with(e2){if(e3){foo()}}}else{bar()}", "if(e1)with(e2)e3&&foo();else bar()");

        fold("if(a||b){if(c||d){var x;}}", "if(a||b)if(c||d)var x");
        fold("if(x){ if(y){var x;}else{var z;} }", "if(x)if(y)var x;else var z");

        // NOTE - technically we can remove the blocks since both the parent
        // and child have elses. But we don't since it causes ambiguities in
        // some cases where not all descendent ifs having elses
        fold(
            "if(x){ if(y){var x;}else{var z;} }else{var w}",
            "if(x)if(y)var x;else var z;else var w",
        );
        fold("if (x) {var x;}else { if (y) { var y;} }", "if(x)var x;else if(y)var y");

        // Here's some of the ambiguous cases
        fold(
            "if(a){if(b){f1();f2();}else if(c){f3();}}else {if(d){f4();}}",
            "if(a)if(b){f1();f2()}else c&&f3();else d&&f4()",
        );

        fold("function f(){foo()}", "function f(){foo()}");
        fold("switch(x){case y: foo()}", "switch(x){case y:foo()}");
        fold(
            "try{foo()}catch(ex){bar()}finally{baz()}",
            "try{foo()}catch(ex){bar()}finally{baz()}",
        );
    }

    /** Try to minimize returns */
    #[test]
    #[ignore]
    fn test_fold_returns() {
        fold("function f(){if(x)return 1;else return 2}", "function f(){return x?1:2}");
        fold("function f(){if(x)return 1;return 2}", "function f(){return x?1:2}");
        fold("function f(){if(x)return;return 2}", "function f(){return x?void 0:2}");
        fold("function f(){if(x)return 1+x;else return 2-x}", "function f(){return x?1+x:2-x}");
        fold("function f(){if(x)return 1+x;return 2-x}", "function f(){return x?1+x:2-x}");
        fold(
            "function f(){if(x)return y += 1;else return y += 2}",
            "function f(){return x?(y+=1):(y+=2)}",
        );

        fold("function f(){if(x)return;else return 2-x}", "function f(){if(x);else return 2-x}");
        fold("function f(){if(x)return;return 2-x}", "function f(){return x?void 0:2-x}");
        fold("function f(){if(x)return x;else return}", "function f(){if(x)return x;{}}");
        fold("function f(){if(x)return x;return}", "function f(){if(x)return x}");

        fold_same("function f(){for(var x in y) { return x.y; } return k}");
    }

    #[test]
    #[ignore]
    fn test_combine_ifs1() {
        fold(
            "function f() {if (x) return 1; if (y) return 1}",
            "function f() {if (x||y) return 1;}",
        );
        fold(
            "function f() {if (x) return 1; if (y) foo(); else return 1}",
            "function f() {if ((!x)&&y) foo(); else return 1;}",
        );
    }

    #[test]
    #[ignore]
    fn test_combine_ifs2() {
        // combinable but not yet done
        fold_same("function f() {if (x) throw 1; if (y) throw 1}");
        // Can't combine, side-effect
        fold("function f(){ if (x) g(); if (y) g() }", "function f(){ x&&g(); y&&g() }");
        fold("function f(){ if (x) g?.(); if (y) g?.() }", "function f(){ x&&g?.(); y&&g?.() }");
        // Can't combine, side-effect
        fold(
            "function f(){ if (x) y = 0; if (y) y = 0; }",
            "function f(){ x&&(y = 0); y&&(y = 0); }",
        );
    }

    #[test]
    #[ignore]
    fn test_combine_ifs3() {
        fold_same("function f() {if (x) return 1; if (y) {g();f()}}");
    }

    /** Try to minimize assignments */
    #[test]
    #[ignore]
    fn test_fold_assignments() {
        fold("function f(){if(x)y=3;else y=4;}", "function f(){y=x?3:4}");
        fold("function f(){if(x)y=1+a;else y=2+a;}", "function f(){y=x?1+a:2+a}");

        // and operation assignments
        fold("function f(){if(x)y+=1;else y+=2;}", "function f(){y+=x?1:2}");
        fold("function f(){if(x)y-=1;else y-=2;}", "function f(){y-=x?1:2}");
        fold("function f(){if(x)y%=1;else y%=2;}", "function f(){y%=x?1:2}");
        fold("function f(){if(x)y|=1;else y|=2;}", "function f(){y|=x?1:2}");

        // Don't fold if the 2 ops don't match.
        fold_same("function f(){x ? y-=1 : y+=2}");

        // Don't fold if the 2 LHS don't match.
        fold_same("function f(){x ? y-=1 : z-=1}");

        // Don't fold if there are potential effects.
        fold_same("function f(){x ? y().a=3 : y().a=4}");
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_statements() {
        // enableNormalize();
        // TODO(bradfordcsmith): Stop normalizing the expected output or document why it is necessary.
        // enableNormalizeExpectedOutput();
        fold("if (a) { x = 1; x++ } else { x = 2; x++ }", "x=(a) ? 1 : 2; x++");
        // fold(
        // "if (a) { x = 1; x++; y += 1; z = pi; }" + " else  { x = 2; x++; y += 1; z = pi; }",
        // "x=(a) ? 1 : 2; x++; y += 1; z = pi;",
        // );
        // fold(
        // "function z() {" + "if (a) { foo(); return !0 } else { goo(); return !0 }" + "}",
        // "function z() {(a) ? foo() : goo(); return !0}",
        // );
        // fold(
        // "function z() {if (a) { foo(); x = true; return true "
        // + "} else { goo(); x = true; return true }}",
        // "function z() {(a) ? foo() : goo(); x = true; return true}",
        // );

        // fold(
        // "function z() {"
        // + "  if (a) { bar(); foo(); return true }"
        // + "    else { bar(); goo(); return true }"
        // + "}",
        // "function z() {"
        // + "  if (a) { bar(); foo(); }"
        // + "    else { bar(); goo(); }"
        // + "  return true;"
        // + "}",
        // );
    }

    #[test]
    #[ignore]
    fn test_fold_returns_integration2() {
        // late = true;
        // disableNormalize();

        // if-then-else duplicate statement removal handles this case:
        test_same(
            "function test(a) {if (a) {const a = Math.random();if(a) {return a;}} return a; }",
        );
    }

    #[test]
    #[ignore]
    fn test_dont_remove_duplicate_statements_without_normalization() {
        // In the following test case, we can't remove the duplicate "alert(x);" lines since each "x"
        // refers to a different variable.
        // We only try removing duplicate statements if the AST is normalized and names are unique.
        test_same(
            "if (Math.random() < 0.5) { const x = 3; alert(x); } else { const x = 5; alert(x); }",
        );
    }

    #[test]
    #[ignore]
    fn test_not_cond() {
        fold("function f(){if(!x)foo()}", "function f(){x||foo()}");
        fold("function f(){if(!x)b=1}", "function f(){x||(b=1)}");
        fold("if(!x)z=1;else if(y)z=2", "if(x){y&&(z=2);}else{z=1;}");
        fold("if(x)y&&(z=2);else z=1;", "x ? y&&(z=2) : z=1");
        fold("function f(){if(!(x=1))a.b=1}", "function f(){(x=1)||(a.b=1)}");
    }

    #[test]
    #[ignore]
    fn test_and_parentheses_count() {
        fold("function f(){if(x||y)a.foo()}", "function f(){(x||y)&&a.foo()}");
        fold("function f(){if(x.a)x.a=0}", "function f(){x.a&&(x.a=0)}");
        fold("function f(){if(x?.a)x.a=0}", "function f(){x?.a&&(x.a=0)}");
        fold_same("function f(){if(x()||y()){x()||y()}}");
    }

    #[test]
    #[ignore]
    fn test_fold_logical_op_string_compare() {
        // side-effects
        // There is two way to parse two &&'s and both are correct.
        fold("if (foo() && false) z()", "(foo(), 0) && z()");
    }

    #[test]
    #[ignore]
    fn test_fold_not() {
        fold("while(!(x==y)){a=b;}", "while(x!=y){a=b;}");
        fold("while(!(x!=y)){a=b;}", "while(x==y){a=b;}");
        fold("while(!(x===y)){a=b;}", "while(x!==y){a=b;}");
        fold("while(!(x!==y)){a=b;}", "while(x===y){a=b;}");
        // Because !(x<NaN) != x>=NaN don't fold < and > cases.
        fold_same("while(!(x>y)){a=b;}");
        fold_same("while(!(x>=y)){a=b;}");
        fold_same("while(!(x<y)){a=b;}");
        fold_same("while(!(x<=y)){a=b;}");
        fold_same("while(!(x<=NaN)){a=b;}");

        // NOT forces a boolean context
        fold("x = !(y() && true)", "x = !y()");
        // This will be further optimized by PeepholeFoldConstants.
        fold("x = !true", "x = !1");
    }

    #[test]
    #[ignore]
    fn test_minimize_expr_condition() {
        fold("(x ? true : false) && y()", "x&&y()");
        fold("(x ? false : true) && y()", "(!x)&&y()");
        fold("(x ? true : y) && y()", "(x || y)&&y()");
        fold("(x ? y : false) && y()", "(x && y)&&y()");
        fold("(x && true) && y()", "x && y()");
        fold("(x && false) && y()", "0&&y()");
        fold("(x || true) && y()", "1&&y()");
        fold("(x || false) && y()", "x&&y()");
    }

    #[test]
    #[ignore]
    fn test_minimize_while_condition() {
        // This test uses constant folding logic, so is only here for completeness.
        fold("while(!!true) foo()", "while(1) foo()");
        // These test tryMinimizeCondition
        fold("while(!!x) foo()", "while(x) foo()");
        fold("while(!(!x&&!y)) foo()", "while(x||y) foo()");
        fold("while(x||!!y) foo()", "while(x||y) foo()");
        fold("while(!(!!x&&y)) foo()", "while(!x||!y) foo()");
        fold("while(!(!x&&y)) foo()", "while(x||!y) foo()");
        fold("while(!(x||!y)) foo()", "while(!x&&y) foo()");
        fold("while(!(x||y)) foo()", "while(!x&&!y) foo()");
        fold("while(!(!x||y-z)) foo()", "while(x&&!(y-z)) foo()");
        fold("while(!(!(x/y)||z+w)) foo()", "while(x/y&&!(z+w)) foo()");
        fold_same("while(!(x+y||z)) foo()");
        fold_same("while(!(x&&y*z)) foo()");
        fold("while(!(!!x&&y)) foo()", "while(!x||!y) foo()");
        fold("while(x&&!0) foo()", "while(x) foo()");
        fold("while(x||!1) foo()", "while(x) foo()");
        fold("while(!((x,y)&&z)) foo()", "while((x,!y)||!z) foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan_remove_leading_not() {
        fold("if(!(!a||!b)&&c) foo()", "((a&&b)&&c)&&foo()");
        fold("if(!(x&&y)) foo()", "x&&y||foo()");
        fold("if(!(x||y)) foo()", "(x||y)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan1() {
        fold("if(!a&&!b)foo()", "(a||b)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan2() {
        // Make sure trees with cloned functions are marked as changed
        fold("(!(a&&!((function(){})())))||foo()", "!a||(function(){})()||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan2b() {
        // Make sure unchanged trees with functions are not marked as changed
        fold_same("!a||(function(){})()||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan3() {
        fold("if((!a||!b)&&(c||d)) foo()", "(a&&b||!c&&!d)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan5() {
        fold("if((!a||!b)&&c) foo()", "(a&&b||!c)||foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan11() {
        fold(
            "if (x && (y===2 || !f()) && (y===3 || !h())) foo()",
            "(!x || y!==2 && f() || y!==3 && h()) || foo()",
        );
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan20a() {
        fold(
            "if (0===c && (2===a || 1===a)) f(); else g()",
            "if (0!==c || 2!==a && 1!==a) g(); else f()",
        );
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan20b() {
        fold("if (0!==c || 2!==a && 1!==a) g(); else f()", "(0!==c || 2!==a && 1!==a) ? g() : f()");
    }

    #[test]
    #[ignore]
    fn test_preserve_if() {
        fold_same("if(!a&&!b)for(;f(););");
    }

    #[test]
    #[ignore]
    fn test_no_swap_with_dangling_else() {
        fold_same("if(!x) {for(;;)foo(); for(;;)bar()} else if(y) for(;;) f()");
        fold_same("if(!a&&!b) {for(;;)foo(); for(;;)bar()} else if(y) for(;;) f()");
    }

    #[test]
    #[ignore]
    fn test_minimize_hook() {
        fold("x ? x : y", "x || y");
        // We assume GETPROPs don't have side effects.
        fold("x.y ? x.y : x.z", "x.y || x.z");
        fold("x?.y ? x?.y : x.z", "x?.y || x.z");
        fold("x?.y ? x?.y : x?.z", "x?.y || x?.z");

        // This can be folded if x() does not have side effects.
        fold_same("x() ? x() : y()");
        fold_same("x?.() ? x?.() : y()");

        fold("!x ? foo() : bar()", "x ? bar() : foo()");
        fold("while(!(x ? y : z)) foo();", "while(x ? !y : !z) foo();");
        fold("(x ? !y : !z) ? foo() : bar()", "(x ? y : z) ? bar() : foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_comma() {
        fold("while(!(inc(), test())) foo();", "while(inc(), !test()) foo();");
        fold("(inc(), !test()) ? foo() : bar()", "(inc(), test()) ? bar() : foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_expr_result() {
        fold("!x||!y", "x&&y");
        fold("if(!(x&&!y)) foo()", "(!x||y)&&foo()");
        fold("if(!x||y) foo()", "(!x||y)&&foo()");
        fold("(!x||y)&&foo()", "x&&!y||!foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_demorgan21() {
        fold("if (0===c && (2===a || 1===a)) f()", "(0!==c || 2!==a && 1!==a) || f()");
    }

    #[test]
    #[ignore]
    fn test_minimize_and_or1() {
        fold("if ((!a || !b) && (d || e)) f()", "(a&&b || !d&&!e) || f()");
    }

    #[test]
    #[ignore]
    fn test_minimize_for_condition() {
        // This test uses constant folding logic, so is only here for completeness.
        // These could be simplified to "for(;;) ..."
        fold("for(;!!true;) foo()", "for(;1;) foo()");
        // Verify function deletion tracking.
        fold("if(!!true||function(){}) {}", "if(1) {}");
        // Don't bother with FOR inits as there are normalized out.
        fold("for(!!true;;) foo()", "for(!0;;) foo()");

        // These test tryMinimizeCondition
        fold("for(;!!x;) foo()", "for(;x;) foo()");

        fold_same("for(a in b) foo()");
        fold_same("for(a in {}) foo()");
        fold_same("for(a in []) foo()");
        fold("for(a in !!true) foo()", "for(a in !0) foo()");

        fold_same("for(a of b) foo()");
        fold_same("for(a of {}) foo()");
        fold_same("for(a of []) foo()");
        fold("for(a of !!true) foo()", "for(a of !0) foo()");
    }

    #[test]
    #[ignore]
    fn test_minimize_condition_example1() {
        // Based on a real failing code sample.
        fold("if(!!(f() > 20)) {foo();foo()}", "if(f() > 20){foo();foo()}");
    }

    #[test]
    #[ignore]
    fn test_fold_loop_break_late() {
        // late = true;
        fold("for(;;) if (a) break", "for(;!a;);");
        fold_same("for(;;) if (a) { f(); break }");
        fold("for(;;) if (a) break; else f()", "for(;!a;) { { f(); } }");
        fold("for(;a;) if (b) break", "for(;a && !b;);");
        fold("for(;a;) { if (b) break; if (c) break; }", "for(;(a && !b);) if (c) break;");
        fold("for(;(a && !b);) if (c) break;", "for(;(a && !b) && !c;);");
        fold("for(;;) { if (foo) { break; var x; } } x;", "var x; for(;!foo;) {} x;");

        // 'while' is normalized to 'for'
        // enableNormalize();
        fold("while(true) if (a) break", "for(;1&&!a;);");
        // disableNormalize();
    }

    #[test]
    #[ignore]
    fn test_fold_loop_break_early() {
        // late = false;
        fold_same("for(;;) if (a) break");
        fold_same("for(;;) if (a) { f(); break }");
        fold_same("for(;;) if (a) break; else f()");
        fold_same("for(;a;) if (b) break");
        fold_same("for(;a;) { if (b) break; if (c) break; }");

        fold_same("while(1) if (a) break");
        // enableNormalize();
        fold_same("for (; 1; ) if (a) break");
    }

    #[test]
    #[ignore]
    fn test_fold_conditional_var_declaration() {
        fold("if(x) var y=1;else y=2", "var y=x?1:2");
        fold("if(x) y=1;else var y=2", "var y=x?1:2");

        fold_same("if(x) var y = 1; z = 2");
        fold_same("if(x||y) y = 1; var z = 2");

        fold_same("if(x) { var y = 1; print(y)} else y = 2 ");
        fold_same("if(x) var y = 1; else {y = 2; print(y)}");
    }

    #[test]
    #[ignore]
    fn test_fold_if_with_lower_operators_inside() {
        fold("if (x + (y=5)) z && (w,z);", "x + (y=5) && (z && (w,z))");
        fold("if (!(x+(y=5))) z && (w,z);", "x + (y=5) || z && (w,z)");
        fold(
            "if (x + (y=5)) if (z && (w,z)) for(;;) foo();",
            "if (x + (y=5) && (z && (w,z))) for(;;) foo();",
        );
    }

    #[test]
    #[ignore]
    fn test_substitute_return() {
        // late = false;
        // enableNormalize();
        // TODO(bradfordcsmith): Stop normalizing the expected output or document why it is necessary.
        // enableNormalizeExpectedOutput();

        fold("function f() { while(x) { return }}", "function f() { while(x) { break }}");

        fold_same("function f() { while(x) { return 5 } }");

        fold_same("function f() { a: { return 5 } }");

        fold(
            "function f() { while(x) { return 5}  return 5}",
            "function f() { while(x) { break }    return 5}",
        );

        fold(
            "function f() { while(x) { return x}  return x}",
            "function f() { while(x) { break }    return x}",
        );

        fold(
            "function f() { while(x) { if (y) { return }}}",
            "function f() { while(x) { if (y) { break  }}}",
        );

        fold(
            "function f() { while(x) { if (y) { return }} return}",
            "function f() { while(x) { if (y) { break  }}}",
        );

        fold(
            "function f() { while(x) { if (y) { return 5 }} return 5}",
            "function f() { while(x) { if (y) { break    }} return 5}",
        );

        // It doesn't matter if x is changed between them. We are still returning
        // x at whatever x value current holds. The whole x = 1 is skipped.
        fold(
            "function f() { while(x) { if (y) { return x } x = 1} return x}",
            "function f() { while(x) { if (y) { break    } x = 1} return x}",
        );

        fold(
            "function f() { while(x) { if (y) { return x } return x} return x}",
            "function f() { while(x) { if (y) {} break }return x}",
        );

        // A break here only breaks out of the inner loop.
        fold_same("function f() { while(x) { while (y) { return } } }");

        fold_same("function f() { while(1) { return 7}  return 5}");

        // fold_same("function f() {" + "  try { while(x) {return f()}} catch (e) { } return f()}");

        // fold_same("function f() {" + "  try { while(x) {return f()}} finally {alert(1)} return f()}");

        // Both returns has the same handler
        // fold(
        // "function f() {" + "  try { while(x) { return f() } return f() } catch (e) { } }",
        // "function f() {" + "  try { while(x) { break } return f() } catch (e) { } }",
        // );

        // We can't fold this because it'll change the order of when foo is called.
        // fold_same(
        // "function f() {"
        // + "  try { while(x) { return foo() } } finally { alert(1) } "
        // + "  return foo()}",
        // );

        // This is fine, we have no side effect in the return value.
        // fold(
        // "function f() {" + "  try { while(x) { return 1 } } finally { alert(1) } return 1}",
        // "function f() {" + "  try { while(x) { break    } } finally { alert(1) } return 1}",
        // );

        fold_same("function f() { try{ return a } finally { a = 2 } return a; }");

        fold(
            "function f() { switch(a){ case 1: return a; default: g();} return a;}",
            "function f() { switch(a){ case 1: break; default: g();} return a; }",
        );
    }

    #[test]
    #[ignore]
    fn test_substitute_break_for_throw() {
        // late = false;
        // enableNormalize();
        // TODO(bradfordcsmith): Stop normalizing the expected output or document why it is necessary.
        // enableNormalizeExpectedOutput();

        fold_same("function f() { while(x) { throw Error }}");

        fold(
            "function f() { while(x) { throw Error } throw Error }",
            "function f() { while(x) { break } throw Error}",
        );
        fold_same("function f() { while(x) { throw Error(1) } throw Error(2)}");
        fold_same("function f() { while(x) { throw Error(1) } return Error(2)}");

        fold_same("function f() { while(x) { throw 5 } }");

        fold_same("function f() { a: { throw 5 } }");

        fold(
            "function f() { while(x) { throw 5}  throw 5}",
            "function f() { while(x) { break }   throw 5}",
        );

        fold(
            "function f() { while(x) { throw x}  throw x}",
            "function f() { while(x) { break }   throw x}",
        );

        fold_same("function f() { while(x) { if (y) { throw Error }}}");

        fold(
            "function f() { while(x) { if (y) { throw Error }} throw Error}",
            "function f() { while(x) { if (y) { break }} throw Error}",
        );

        fold(
            "function f() { while(x) { if (y) { throw 5 }} throw 5}",
            "function f() { while(x) { if (y) { break    }} throw 5}",
        );

        // It doesn't matter if x is changed between them. We are still throwing
        // x at whatever x value current holds. The whole x = 1 is skipped.
        fold(
            "function f() { while(x) { if (y) { throw x } x = 1} throw x}",
            "function f() { while(x) { if (y) { break    } x = 1} throw x}",
        );

        fold(
            "function f() { while(x) { if (y) { throw x } throw x} throw x}",
            "function f() { while(x) { if (y) {} break }throw x}",
        );

        // A break here only breaks out of the inner loop.
        fold_same("function f() { while(x) { while (y) { throw Error } } }");

        fold_same("function f() { while(1) { throw 7}  throw 5}");

        // fold_same("function f() {" + "  try { while(x) {throw f()}} catch (e) { } throw f()}");

        // fold_same("function f() {" + "  try { while(x) {throw f()}} finally {alert(1)} throw f()}");

        // Both throws has the same handler
        // fold(
        // "function f() {" + "  try { while(x) { throw f() } throw f() } catch (e) { } }",
        // "function f() {" + "  try { while(x) { break } throw f() } catch (e) { } }",
        // );

        // We can't fold this because it'll change the order of when foo is called.
        // fold_same(
        // "function f() {"
        // + "  try { while(x) { throw foo() } } finally { alert(1) } "
        // + "  throw foo()}",
        // );

        // This is fine, we have no side effect in the throw value.
        // fold(
        // "function f() {" + "  try { while(x) { throw 1 } } finally { alert(1) } throw 1}",
        // "function f() {" + "  try { while(x) { break    } } finally { alert(1) } throw 1}",
        // );

        fold_same("function f() { try{ throw a } finally { a = 2 } throw a; }");

        fold(
            "function f() { switch(a){ case 1: throw a; default: g();} throw a;}",
            "function f() { switch(a){ case 1: break; default: g();} throw a; }",
        );
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_return() {
        // late = false;
        // enableNormalize();

        fold("function f() { return; }", "function f(){}");
        fold_same("function f() { return a; }");
        fold(
            "function f() { if (x) { return a } return a; }",
            "function f() { if (x) {} return a; }",
        );
        fold_same("function f() { try { if (x) { return a } } catch(e) {} return a; }");
        fold_same("function f() { try { if (x) {} } catch(e) {} return 1; }");

        // finally clauses may have side effects
        fold_same("function f() { try { if (x) { return a } } finally { a++ } return a; }");
        // but they don't matter if the result doesn't have side effects and can't
        // be affect by side-effects.
        fold(
            "function f() { try { if (x) { return 1 } } finally {} return 1; }",
            "function f() { try { if (x) {} } finally {} return 1; }",
        );

        fold(
            "function f() { switch(a){ case 1: return a; } return a; }",
            "function f() { switch(a){ case 1: } return a; }",
        );

        // fold(
        // "function f() { switch(a){ " + "  case 1: return a; case 2: return a; } return a; }",
        // "function f() { switch(a){ " + "  case 1: break; case 2: } return a; }",
        // );
    }

    #[test]
    #[ignore]
    fn test_remove_duplicate_throw() {
        // late = false;
        // enableNormalize();

        fold_same("function f() { throw a; }");
        fold("function f() { if (x) { throw a } throw a; }", "function f() { if (x) {} throw a; }");
        fold_same("function f() { try { if (x) {throw a} } catch(e) {} throw a; }");
        fold_same("function f() { try { if (x) {throw 1} } catch(e) {f()} throw 1; }");
        fold_same("function f() { try { if (x) {throw 1} } catch(e) {f()} throw 1; }");
        fold_same("function f() { try { if (x) {throw 1} } catch(e) {throw 1}}");
        fold(
            "function f() { try { if (x) {throw 1} } catch(e) {throw 1} throw 1; }",
            "function f() { try { if (x) {throw 1} } catch(e) {} throw 1; }",
        );

        // finally clauses may have side effects
        fold_same("function f() { try { if (x) { throw a } } finally { a++ } throw a; }");
        // but they don't matter if the result doesn't have side effects and can't
        // be affect by side-effects.
        fold(
            "function f() { try { if (x) { throw 1 } } finally {} throw 1; }",
            "function f() { try { if (x) {} } finally {} throw 1; }",
        );

        fold(
            "function f() { switch(a){ case 1: throw a; } throw a; }",
            "function f() { switch(a){ case 1: } throw a; }",
        );

        // fold(
        // "function f() { switch(a){ " + "case 1: throw a; case 2: throw a; } throw a; }",
        // "function f() { switch(a){ case 1: break; case 2: } throw a; }",
        // );
    }

    #[test]
    #[ignore]
    fn test_nested_if_combine() {
        fold("if(x)if(y){while(1){}}", "if(x&&y){while(1){}}");
        fold("if(x||z)if(y){while(1){}}", "if((x||z)&&y){while(1){}}");
        fold("if(x)if(y||z){while(1){}}", "if((x)&&(y||z)){while(1){}}");
        fold_same("if(x||z)if(y||z){while(1){}}");
        fold("if(x)if(y){if(z){while(1){}}}", "if(x&&(y&&z)){while(1){}}");
    }

    // See: http://blickly.github.io/closure-compiler-issues/#291
    #[test]
    #[ignore]
    fn test_issue291() {
        fold("if (true) { f.onchange(); }", "if (1) f.onchange();");
        fold_same("if (f) { f.onchange(); }");
        fold_same("if (f) { f.bar(); } else { f.onchange(); }");
        fold("if (f) { f.bonchange(); }", "f && f.bonchange();");
        fold_same("if (f) { f['x'](); }");

        // optional versions
        fold("if (true) { f?.onchange(); }", "if (1) f?.onchange();");
        fold_same("if (f) { f?.onchange(); }");
        fold_same("if (f) { f?.bar(); } else { f?.onchange(); }");
        fold("if (f) { f?.bonchange(); }", "f && f?.bonchange();");
        fold_same("if (f) { f?.['x'](); }");
    }

    #[test]
    #[ignore]
    fn test_object_literal() {
        test("({})", "1");
        test("({a:1})", "1");
        test_same("({a:foo()})");
        test_same("({'a':foo()})");
    }

    #[test]
    #[ignore]
    fn test_array_literal() {
        test("([])", "1");
        test("([1])", "1");
        test("([a])", "1");
        test_same("([foo()])");
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause() {
        // test(
        // "function f() {" + " if(x) return 1;" + " else if(x) return 2;" + " else if(x) return 3 }",
        // "function f() {" + " if(x) return 1;" + "{ if(x) return 2;" + "{ if(x) return 3 } } }",
        // );
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause1() {
        test(
            "function f() { if (x) throw 1; else f() }",
            "function f() { if (x) throw 1; { f() } }",
        );
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause2() {
        test(
            "function f() { if (x) return 1; else f() }",
            "function f() { if (x) return 1; { f() } }",
        );
        test("function f() { if (x) return; else f() }", "function f() { if (x) {} else { f() } }");
        // This case is handled by minimize exit points.
        test_same("function f() { if (x) return; f() }");
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause3() {
        test_same("function f() { a:{if (x) break a; else f() } }");
        test_same("function f() { if (x) { a:{ break a } } else f() }");
        test_same("function f() { if (x) a:{ break a } else f() }");
    }

    #[test]
    #[ignore]
    fn test_remove_else_cause4() {
        test_same("function f() { if (x) { if (y) { return 1; } } else f() }");
    }

    #[test]
    #[ignore]
    fn test_issue925() {
        // test(
        // "if (x[--y] === 1) {\n" + "    x[y] = 0;\n" + "} else {\n" + "    x[y] = 1;\n" + "}",
        // "(x[--y] === 1) ? x[y] = 0 : x[y] = 1;",
        // );

        // test(
        // "if (x[--y]) {\n" + "    a = 0;\n" + "} else {\n" + "    a = 1;\n" + "}",
        // "a = (x[--y]) ? 0 : 1;",
        // );

        // test(
        // lines(
        // "if (x?.[--y]) {", //
        // "    a = 0;",
        // "} else {",
        // "    a = 1;",
        // "}",
        // ),
        // "a = (x?.[--y]) ? 0 : 1;",
        // );

        test("if (x++) { x += 2 } else { x += 3 }", "x++ ? x += 2 : x += 3");

        test("if (x++) { x = x + 2 } else { x = x + 3 }", "x = x++ ? x + 2 : x + 3");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_disabled() {
        // enableTypeCheck();
        test_same("var x = {}; if (x != null) throw 'a';");
        test_same("var x = {}; var y = x != null;");

        test_same("var x = 1; if (x != 0) throw 'a';");
        test_same("var x = 1; var y = x != 0;");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_boolean_result0() {
        // enableTypeCheck();
        test_same("var x = {}; var y = x != null;");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_boolean_result1() {
        // enableTypeCheck();
        test_same("var x = {}; var y = x == null;");
        test_same("var x = {}; var y = x !== null;");
        test_same("var x = undefined; var y = x !== null;");
        test_same("var x = {}; var y = x === null;");
        test_same("var x = undefined; var y = x === null;");

        test_same("var x = 1; var y = x != 0;");
        test_same("var x = 1; var y = x == 0;");
        test_same("var x = 1; var y = x !== 0;");
        test_same("var x = 1; var y = x === 0;");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_if() {
        // enableTypeCheck();
        test("var x = {};\nif (x != null) throw 'a';\n", "var x={}; if (x!=null) throw 'a'");
        test_same("var x = {};\nif (x == null) throw 'a';\n");
        test_same("var x = {};\nif (x !== null) throw 'a';\n");
        test_same("var x = {};\nif (x === null) throw 'a';\n");
        test_same("var x = {};\nif (null != x) throw 'a';\n");
        test_same("var x = {};\nif (null == x) throw 'a';\n");
        test_same("var x = {};\nif (null !== x) throw 'a';\n");
        test_same("var x = {};\nif (null === x) throw 'a';\n");

        test_same("var x = 1;\nif (x != 0) throw 'a';\n");
        test_same("var x = 1;\nif (x != 0) throw 'a';\n");
        test_same("var x = 1;\nif (x == 0) throw 'a';\n");
        test_same("var x = 1;\nif (x !== 0) throw 'a';\n");
        test_same("var x = 1;\nif (x === 0) throw 'a';\n");
        test_same("var x = 1;\nif (0 != x) throw 'a';\n");
        test_same("var x = 1;\nif (0 == x) throw 'a';\n");
        test_same("var x = 1;\nif (0 !== x) throw 'a';\n");
        test_same("var x = 1;\nif (0 === x) throw 'a';\n");
        test_same("var x = NaN;\nif (0 === x) throw 'a';\n");
        test_same("var x = NaN;\nif (x === 0) throw 'a';\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_expression() {
        // enableTypeCheck();
        test_same("var x = {}; x != null && alert('b');");
        test_same("var x = 1; x != 0 && alert('b');");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_hook() {
        // enableTypeCheck();
        // test_same(lines("var x = {};", "var y = x != null ? 1 : 2;"));
        // test_same(lines("var x = 1;", "var y = x != 0 ? 1 : 2;"));
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_not() {
        // enableTypeCheck();
        test(
            "var x = {};\nvar y = !(x != null) ? 1 : 2;\n",
            "var x = {};\nvar y = (x == null) ? 1 : 2;\n",
        );
        test("var x = 1;\nvar y = !(x != 0) ? 1 : 2;\n", "var x = 1;\nvar y = x == 0 ? 1 : 2;\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_while() {
        // enableTypeCheck();
        test_same("var x = {}; while (x != null) throw 'a';");
        test_same("var x = 1; while (x != 0) throw 'a';");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_unknown_type() {
        // enableTypeCheck();
        test_same("var x = /** @type {?} */ ({});\nif (x != null) throw 'a';\n");
        test_same("var x = /** @type {?} */ (1);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_all_type() {
        // enableTypeCheck();
        test_same("var x = /** @type {*} */ ({});\nif (x != null) throw 'a';\n");
        test_same("var x = /** @type {*} */ (1);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_primitives_vs_null() {
        // enableTypeCheck();
        test_same("var x = 0;\nif (x != null) throw 'a';\n");
        test_same("var x = '';\nif (x != null) throw 'a';\n");
        test_same("var x = false;\nif (x != null) throw 'a';\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_non_number_vs_zero() {
        // enableTypeCheck();
        test_same("var x = {};\nif (x != 0) throw 'a';\n");
        test_same("var x = '';\nif (x != 0) throw 'a';\n");
        test_same("var x = false;\nif (x != 0) throw 'a';\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_boxed_number_vs_zero() {
        // enableTypeCheck();
        test_same("var x = new Number(0);\nif (x != 0) throw 'a';\n");
    }

    #[test]
    #[ignore]
    fn test_coercion_substitution_boxed_primitives() {
        // enableTypeCheck();
        test_same("var x = new Number(); if (x != null) throw 'a';");
        test_same("var x = new String(); if (x != null) throw 'a';");
        test_same("var x = new Boolean();\nif (x != null) throw 'a';");
    }

    #[test]
    #[ignore]
    fn test_minimize_if_with_new_target_condition() {
        // Related to https://github.com/google/closure-compiler/issues/3097
        // test(
        // lines(
        // "function x() {",
        // "  if (new.target) {",
        // "    return 1;",
        // "  } else {",
        // "    return 2;",
        // "  }",
        // "}",
        // ),
        // lines("function x() {", "  return new.target ? 1 : 2;", "}"),
        // );
    }
}
