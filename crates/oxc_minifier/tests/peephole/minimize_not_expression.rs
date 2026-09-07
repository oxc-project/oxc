use crate::{test, test_same};

#[test]
fn minimize_duplicate_nots() {
    test("!x", "x");
    test("!!x", "x");
    test("!!!x", "x");
    test("!!!!x", "x");
    test("!!!(x && y)", "x && y");
    test("var k = () => { !!x; }", "var k = () => { x }");

    test_same("var k = !!x;");
    test_same("function k () { return !!x; }");
    test("var k = () => { return !!x; }", "var k = () => !!x");
    test_same("var k = () => !!x;");
    // Negation of a `1`, `0` is boolean context.
    test("var v = !(!0);", "var v = !1;");
    test("var v = !(!1);", "var v = !0;");
    test("var v = !(a || !0);", "var v = !(a || 1);");
    test("var v = !(a && !1);", "var v = !(a && 0);");
    // fold not into sequence
    test("var v = !(a, b)", "var v = (a, !b)");
}

#[test]
fn minimize_nots_with_de_morgan_comparison_chains() {
    // Jump bodies keep the `if` a statement, so the `!(...)` used to survive to the output.
    test("if (!(a == b || c == d)) throw x;", "if (a != b && c != d) throw x;");
    test("if (!(a === b || c === d)) throw x;", "if (a !== b && c !== d) throw x;");
    // `&&` dual.
    test("if (!(a == b && c == d)) throw x;", "if (a != b || c != d) throw x;");
    // The fold is involutive, so the `if (!x) return` collapse (which negates
    // the test again) still reaches its old output.
    test(
        "function f() { if (!(a === b || c === d)) return; g(); }",
        "function f() { (a === b || c === d) && g(); }",
    );
    // Loop rotation consumes the `!` of a folded chain losslessly as well.
    test("while (e) { if (!(a == b || c == d)) break; }", "for (; e && (a == b || c == d);) ;");
    // Loop tests.
    test("while (!(a == b || c == d)) g();", "for (; a != b && c != d;) g();");
    test("do g(); while (!(a == b && c == d));", "do g(); while (a != b || c != d);");
    test("for (; !(a == b || c == d);) g();", "for (; a != b && c != d;) g();");
    // Longer chains and nesting that loses parentheses after inversion.
    test("if (!(a == b || c == d || e == f)) throw x;", "if (a != b && c != d && e != f) throw x;");
    test(
        "if (!((a == b || c == d) && e == f)) throw x;",
        "if (a != b && c != d || e != f) throw x;",
    );
    // The fold is exact, so value contexts fold too.
    test("var v = !(a == b || c == d);", "var v = a != b && c != d;");
}

#[test]
fn minimize_nots_with_de_morgan_negative_cases() {
    // Relational comparisons don't invert freely (NaN), so the chain must stay.
    test_same("if (!(a < b || c < d)) throw x;");
    // A mixed operand would need a bare `!`; that fold is not involutive and can
    // regress shapes whose test is negated again later (e.g. branch swaps), so
    // it's left alone.
    test_same("if (!(a == b || c)) throw x;");
    // `&&` nested under `||` gains parentheses after inversion; the size guard rejects.
    test_same("if (!(a == b && c == d || e == f)) throw x;");
    // Existing shapes that consume the `!` for free must not regress.
    test("var v = !!(a == b || c == d);", "var v = a == b || c == d;");
    test("if (!(a == b && c == d)) x(); else y();", "a != b || c != d ? x() : y();");
    // mixed unary-not leaves in both boolean and value contexts.
    test("if (!(!a || b)) x();", "a && !b && x();");
    test_same("var v = !(!a || b);");
    test("if (!(a == b || !c && d == e)) x();", "a == b || !c && d == e || x();");
    // parenthesis-size guard.
    test_same("if (!(a == b || fn1())) throw x;");
    test_same("if (!(a == b && fn1())) throw x;");
    test("var v = !!(!a || b);", "var v = !(a && !b);");
    test("var v = !!(a && !b);", "var v = !(!a || b);");
    test_same("if (!(a < b || !c)) throw x;");
    test_same("if (!(a in b || c == d)) throw x;");
    test("if (!(a === b && (a ?? b))) throw x;", "if (!(a === b && (a ?? b))) throw x;");
    test("if (!(a === b && !a)) throw x;", "if (a !== b || a) throw x;");
}

#[test]
fn minimize_nots_with_binary_expressions() {
    test("!(x === undefined)", "x");
    test("!(typeof(x) === 'undefined')", "");
    test("!(typeof(x()) === 'undefined')", "x()");
    test("!(x === void 0)", "x");
    test("!!delete x.y", "delete x.y");
    test("!!!delete x.y", "delete x.y");
    test("!!!!delete x.y", "delete x.y");
    test("var k = !!(foo instanceof bar)", "var k = foo instanceof bar");
    test("!(a === 1 ? void 0 : a.b)", "a !== 1 && a.b;");
    test("!(a, b)", "a, b");
}
