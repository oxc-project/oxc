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
