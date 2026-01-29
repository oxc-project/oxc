use crate::{test, test_same, test_target};

#[test]
fn test_minimize_expr_condition() {
    test("(x ? true : false) && y()", "x && y()");
    test("(x ? false : true) && y()", "!x && y()");
    test("(x ? true : y) && y()", "(x || y) && y();");
    test("(x ? y : false) && y()", "(x && y) && y()");
    test("var x; (x && true) && y()", "var x; x && y()");
    test("var x; (x && false) && y()", "var x");
    test("(x && true) && y()", "x && y()");
    test("(x && false) && y()", "x");
    test("var x; (x || true) && y()", "var x; y()");
    test("var x; (x || false) && y()", "var x; x && y()");

    test("(x || true) && y()", "x, y()");
    test("(x || false) && y()", "x && y()");

    test("let x = foo ? true : false", "let x = !!foo");
    test("let x = foo ? true : bar", "let x = foo ? !0 : bar");
    test("let x = foo ? bar : false", "let x = foo ? bar : !1");
    test("function x () { return a ? true : false }", "function x() { return !!a }");
    test("function x () { return a ? false : true }", "function x() { return !a }");
    test("function x () { return a ? true : b }", "function x() { return a ? !0 : b }");
    // can't be minified e.g. `a = ''` would return `''`
    test("function x() { return a && true }", "function x() { return a && !0 }");

    test("foo ? bar : bar", "foo, bar");
    test_same("foo ? bar : baz");
    test("foo() ? bar : bar", "foo(), bar");

    test_same("var k = () => !!x;");
}

#[test]
fn minimize_conditional_exprs() {
    test("(a, b) ? c : d", "a, b ? c : d");
    test("!a ? b : c", "a ? c : b");
    test("/* @__PURE__ */ a() ? b : b", "b");
    test("a ? b : b", "a, b");
    test("a ? true : false", "a");
    test("a ? false : true", "a");
    test("a ? a : b", "a || b");
    test("a ? b : a", "a && b");
    test("a ? b ? c : d : d", "a && b ? c : d");
    test("a ? b : c ? b : d", "a || c ? b : d");
    test("a ? c : (b, c)", "(a || b), c");
    test("a ? (b, c) : c", "(a && b), c");
    test("a ? b || c : c", "(a && b) || c");
    test("a ? c : b && c", "(a || b) && c");
    test("var a, b; a ? b(c, d) : b(e, d)", "var a, b; b(a ? c : e, d)");
    test("var a, b; a ? b(...c) : b(...e)", "var a, b; b(...a ? c : e)");
    test("var a, b; a ? b(c) : b(e)", "var a, b; b(a ? c : e)");
    test("var a, b; a ? b() : b()", "var a, b; b()");
    test("var a, b; a === 0 ? b(c) : b(e)", "var a, b; b(a === 0 ? c : e)");
    test_same("var a; a === 0 ? b(c) : b(e)"); // accessing global `b` may assign a different value to `a`
    test_same("var b; a === 0 ? b(c) : b(e)"); // accessing global `a` may assign a different value to `b`
    test_same("a === 0 ? b(c) : b(e)"); // accessing global `a`, `b` may have a side effect
    test("a() != null ? a() : b", "a() == null ? b : a()");
    test("var a; a != null ? a : b", "var a; a ?? b");
    test("var a; (a = _a) != null ? a : b", "var a; (a = _a) ?? b");
    test("v = a != null ? a : b", "v = a == null ? b : a"); // accessing global `a` may have a getter with side effects
    test_target("var a; v = a != null ? a : b", "var a; v = a == null ? b : a", "chrome79");
    test("var a; v = a != null ? a.b.c[d](e) : undefined", "var a; v = a?.b.c[d](e)");
    test("var a; v = (a = _a) != null ? a.b.c[d](e) : undefined", "var a; v = (a = _a)?.b.c[d](e)");
    test("v = a != null ? a.b.c[d](e) : undefined", "v = a == null ? void 0 : a.b.c[d](e)"); // accessing global `a` may have a getter with side effects
    test(
        "var a, undefined = 1; v = a != null ? a.b.c[d](e) : undefined",
        "var a; v = a == null ? 1 : a.b.c[d](e)",
    );
    test_target(
        "var a; v = a != null ? a.b.c[d](e) : undefined",
        "var a; v = a == null ? void 0 : a.b.c[d](e)",
        "chrome79",
    );
    test("v = cmp !== 0 ? cmp : (bar, cmp);", "v = (cmp === 0 && bar, cmp);");
    test("v = cmp === 0 ? cmp : (bar, cmp);", "v = (cmp === 0 || bar, cmp);");
    test("v = cmp !== 0 ? (bar, cmp) : cmp;", "v = (cmp === 0 || bar, cmp);");
    test("v = cmp === 0 ? (bar, cmp) : cmp;", "v = (cmp === 0 && bar, cmp);");
}

#[test]
fn compress_conditional() {
    test("foo ? foo : bar", "foo || bar");
    test("foo ? bar : foo", "foo && bar");
    test_same("x.y ? x.y : bar");
    test_same("x.y ? bar : x.y");
}
