use crate::{test, test_same, test_target};

#[test]
fn test_minimize_expr_condition() {
    test("(x ? true : false) && y()", "x && y()");
    test("(x ? false : true) && y()", "!x && y()");
    test("(x ? true : y) && y()", "(x || y) && y();");
    test("(x ? y : false) && y()", "(x && y) && y()");
    test("v = function(x) { (x && true) && y() }", "v = function(x) { x && y() }");
    test("var x; (x && false) && y()", "var x");
    test("(x && true) && y()", "x && y()");
    test("(x && false) && y()", "x");
    test("var x; (x || true) && y()", "var x; y()");
    test("v = function(x) { (x || false) && y() }", "v = function(x) { x && y() }");

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
    test(
        "v = function(a, b) { return a ? b(c, d) : b(e, d) }",
        "v = function(a, b) { return b(a ? c : e, d) }",
    );
    test(
        "v = function(a, b) { return a ? b(...c) : b(...e) }",
        "v = function(a, b) { return b(...a ? c : e) }",
    );
    test(
        "v = function(a, b) { return a ? b(c) : b(e) }",
        "v = function(a, b) { return b(a ? c : e) }",
    );
    test("v = function(a, b) { return a ? b() : b() }", "v = function(a, b) { return b() }");
    test(
        "v = function(a, b) { return a === 0 ? b(c) : b(e) }",
        "v = function(a, b) { return b(a === 0 ? c : e) }",
    );
    test_same("v = function(a) { return a === 0 ? b(c) : b(e) }"); // accessing global `b` may assign a different value to `a`
    test_same("v = function(b) { return a === 0 ? b(c) : b(e) }"); // accessing global `a` may assign a different value to `b`
    test_same("a === 0 ? b(c) : b(e)"); // accessing global `a`, `b` may have a side effect
    test("a() != null ? a() : b", "a() == null ? b : a()");
    test("v = function(a) { return a != null ? a : b }", "v = function(a) { return a ?? b }");
    test("var a; (a = _a) != null ? a : b", "var a; (a = _a) ?? b");
    test("v = a != null ? a : b", "v = a == null ? b : a"); // accessing global `a` may have a getter with side effects
    test_target(
        "v = function(a) { return a != null ? a : b }",
        "v = function(a) { return a == null ? b : a }",
        "chrome79",
    );
    test(
        "v = function(a) { return a != null ? a.b.c[d](e) : undefined }",
        "v = function(a) { return a?.b.c[d](e) }",
    );
    test("var a; v = (a = _a) != null ? a.b.c[d](e) : undefined", "var a; v = (a = _a)?.b.c[d](e)");
    test("v = a != null ? a.b.c[d](e) : undefined", "v = a == null ? void 0 : a.b.c[d](e)"); // accessing global `a` may have a getter with side effects
    test(
        "v = function(a) { var undefined = 1; return a != null ? a.b.c[d](e) : undefined }",
        "v = function(a) { return a == null ? 1 : a.b.c[d](e) }",
    );
    test_target(
        "v = function(a) { return a != null ? a.b.c[d](e) : undefined }",
        "v = function(a) { return a == null ? void 0 : a.b.c[d](e) }",
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

#[test]
fn test_minimize_conditional_numeric() {
    // "a ? 1 : 0" => "+a" when a is known boolean
    test("let x = !y ? 1 : 0", "let x = +!y");

    // "a ? 1 : 0" => "+!!a" when a is not known boolean (no parens needed)
    test("let x = a ? 1 : 0", "let x = +!!a");

    // "a ? 1 : 0" stays when parens + !! would make it longer
    test("let x = a + b ? 1 : 0", "let x = a + b ? 1 : 0");

    // "a ? 0 : 1" => "+!a"
    test("let x = a ? 0 : 1", "let x = +!a");
    test("let x = !y ? 0 : 1", "let x = +!!y");

    // "a ? 0 : 1" stays when parens would make it same or longer
    test("let x = a + b ? 0 : 1", "let x = a + b ? 0 : 1");

    // `-0` must not be folded to `+a`/`+!a`: that would turn the `-0` branch into `+0`.
    test_same("let x = a ? 1 : -0");
    test_same("let x = a ? -0 : 1");
    // The test may still be negated + branches swapped, but `-0` is preserved (not `+0`).
    test("let x = !y ? 1 : -0", "let x = y ? -0 : 1");
}

#[test]
fn test_minimize_conditional_boolean_value_context() {
    // Form 1: "c ? false : x" => "!c && x" (exact for any `c`)
    test("let x = foo() ? false : bar()", "let x = !foo() && bar()");
    test("let x = num ? false : y", "let x = !num && y");
    test("foo(a ? false : b)", "foo(!a && b)");
    // equality tests invert their operator in place via `minimize_not`
    test("function f() { return a === b ? false : x }", "function f() { return a !== b && x }");

    // Form 2: "c ? x : true" => "!c || x" (exact for any `c`)
    test("let x = foo() ? bar() : true", "let x = !foo() || bar()");
    test("let x = a === b ? c : true", "let x = a !== b || c");
    // "!a ? true : x" first flips to "a ? x : true", then folds via form 2
    test("let x = !a ? true : b", "let x = !a || b");

    // Form 3: "c ? true : x" => "c || x" only when `c` is boolean-typed
    test("let x = a === b ? true : c", "let x = a === b || c");
    // Form 4: "c ? x : false" => "c && x" only when `c` is boolean-typed
    test("let x = a === b ? c : false", "let x = a === b && c");

    // A sequence branch already needs parentheses in a conditional, so using it
    // as a logical operand does not add any bytes.
    test("use(flag ? false : (touch(), true))", "use(!flag && (touch(), !0))");
    test("use(flag ? (touch(), value) : true)", "use(!flag || (touch(), value))");
    test("use(a === b ? true : (touch(), value))", "use(a === b || (touch(), value))");
    test("use(a === b ? (touch(), value) : false)", "use(a === b && (touch(), value))");
    test("use((prepare(), a === b) ? true : value)", "use((prepare(), a === b || value))");

    // Low-precedence conditional tests already need parentheses, so reusing
    // them as logical operands does not add any bytes.
    test("use((flag = a === b) ? false : value)", "use(!(flag = a === b) && value)");
    test("use((flag = a === b) ? value : true)", "use(!(flag = a === b) || value)");
    test("use((flag = a === b) ? true : value)", "use((flag = a === b) || value)");
    test("use((flag = a === b) ? value : false)", "use((flag = a === b) && value)");
    test("use((flag ? left : right) ? false : value)", "use(!(flag ? left : right) && value)");
    test(
        "use((flag ? a === b : c === d) ? true : value)",
        "use((flag ? a === b : c === d) || value)",
    );

    // Negative: forms 3/4 require a boolean-typed test, else the value changes.
    test("let x = num ? true : y", "let x = num ? !0 : y");
    test("let x = num ? y : false", "let x = num ? y : !1");

    // Negative: size guard. `!(a || b) && c` is longer than `a || b ? !1 : c`.
    test("let x = a || b ? false : c", "let x = a || b ? !1 : c");
    // Negative: `x` needing parens as a logical operand would not save bytes.
    test("let x = a ? false : b ? c : d", "let x = a ? !1 : b ? c : d");
    test("use(flag ? false : (value = touch()))", "use(flag ? !1 : value = touch())");
    test("use(a === b || c === d ? value : false)", "use(a === b || c === d ? value : !1)");
}

/// `a ? 1 : 0` becomes `+!!a`, whose value is `ToBoolean(a)` coerced to a
/// number. Deriving it from `ToNumber(a)` instead flips the result for every
/// value where `ToNumber(a) === 0` and `ToBoolean(a)` disagree.
#[test]
fn test_conditional_to_unary_plus_uses_to_boolean() {
    // `ToNumber(NaN)` is not `0`, but `NaN` is falsy.
    test("x = (NaN ? 1 : 0) || 2", "x = 2");
    test("x = (NaN ? 1 : 0) && 2", "x = 0");
    // `ToNumber("0")` is `0`, but a non-empty string is truthy.
    test("x = ('0' ? 1 : 0) || 2", "x = 1");
    test("x = (' ' ? 1 : 0) || 2", "x = 1");
    test("x = ('0.0' ? 1 : 0) || 2", "x = 1");
    // `ToNumber([])` is `0`, but every object is truthy.
    test("x = ([] ? 1 : 0) || 2", "x = 1");
}
