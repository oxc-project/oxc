use crate::{test, test_same};

#[test]
fn merge_assignments_to_declarations_var() {
    test("var a; a = 0", "var a = 0");
    test_same("var a = 0; a = 1"); // this can be improved to `var a = 1`
    test_same("var a = 0; a = b()"); // `b()` may access `a`
    test_same("var a = b(); a = c()"); // `c()` may access `a`
    test_same("var a, b = 1; a = 0"); // this can be improved to `var a = 0, b = 1`
    test_same("var a, b = c(); a = 0"); // `c()` may access `a`
    test("var a, b; a = 0", "var a = 0, b");
    test("var a, b; a = 0, b = 1", "var a = 0, b = 1");
    test("var a, b; a = 0; b = 1", "var a = 0, b = 1");
    test("var a, b; a = c()", "var a = c(), b");
    test("var a, b; a = c(), b = d()", "var a = c(), b = d()");
    test("var a, b; a = c(); b = d()", "var a = c(), b = d()");
    test("var a, b; a = b", "var a = b, b");

    test("var a, b, c; a = 0, b = 1, c = 2", "var a = 0, b = 1, c = 2");
    test("var a, b; a = 0, b = 1, foo()", "var a = 0, b = 1; foo()");
    test("var a; a = 0, foo(), bar()", "var a = 0; foo(), bar()");
    test_same("var a, b; foo(), bar()");
}

#[test]
fn merge_assignments_to_declarations_let() {
    test("let a; a = 0", "let a = 0");
    test_same("let a = 0; a = 1"); // this can be improved to `let a = 1`
    test_same("let a = 0; a = b()"); // `b()` may access `a`
    test_same("let a = b(); a = c()"); // `c()` may access `a`
    test_same("let a, b = 1; a = 0"); // this can be improved to `let a = 0, b = 1`
    test_same("let a, b = c(); a = 0"); // `c()` may access `a`
    test_same("let a, b; a = 0"); // this can be improved to `let a = 0, b`
    test("let a, b; a = 0; b = 1", "let a, b; a = 0, b = 1"); // this can be improved to `let a = 0, b = 1`
    test_same("let a, b; a = c()"); // `c()` may access `b`, `let a = c(), b` will cause TDZ error
    test("let a, b; a = c(); b = d()", "let a, b; a = c(), b = d()"); // same as above
    test_same("let a, b; a = b"); // `let a = b, b` will cause TDZ error; `b` reads as the implicit `undefined`, which is not worth inlining (rolldown#10174)
    test_same("let a; a = foo(a)"); // `let a = foo(a)` will cause TDZ error
    test("let a; a = (() => a)()", "let a; a = a;"); // `let a = (() => a)()` will cause TDZ error
    test("let a; a = () => a", "let a = () => a");
}

#[test]
fn merge_assignments_to_declarations_other() {
    test_same("const a = 0; a = 1");
    test_same("using a = 0; a = 1");
    test_same("await using a = 0; a = 1");
}

#[test]
fn take_leading_assignments_from_statements() {
    test("function f() { let a; return a = 0, a }", "function f() { let a = 0; return 0 }");
    test("function f() { var a; return a = b(), a }", "function f() { return b() }");
    test(
        "function f() { var a, b; return a = 1, b = 2, c }",
        "function f() { var a = 1, b = 2; return c }",
    );
    // only a contiguous prefix may move, `a = 1` must not be hoisted above `foo()`
    test_same("function f() { var a; return foo(), a = 1, b }");
    // the last expression is never taken, its value is needed
    test_same("function f() { var a; return a = 1 }");

    test("function f() { var a; throw a = 1, e }", "function f() { var a = 1; throw e }");
    test("function f() { var a; if (a = 1, c) foo() }", "function f() { var a = 1; c && foo() }");
    test(
        "function f() { var a; switch (a = 1, c) { case 1: foo() } }",
        "function f() { var a = 1; c === 1 && foo() }",
    );
    test(
        "function f() { var a; for (a = 1, b(); c; d()) foo() }",
        "function f() { var a = 1; for (b(); c; d()) foo() }",
    );
    test(
        "function f() { var a; for (a = 1; c; d()) foo() }",
        "function f() { for (var a = 1; c; d()) foo() }",
    );
    test_same("function f() { var a; for (x of a = 1) foo() }");
    test(
        "function f() { var a; for (x of (a = 1, b)) foo() }",
        "function f() { var a = 1; for (x of b) foo() }",
    );
    test_same("function f() { var a; for (x in a = 1) foo() }");
    test(
        "function f() { var a; for (x in (a = 1, b)) foo() }",
        "function f() { var a = 1; for (x in b) foo() }",
    );
    test("function f() { var a; var b = (a = 1, 2) }", "function f() { var a = 1, b = 2 }");
}

#[test]
fn take_leading_assignments_edge_cases() {
    // `let` may only take a literal, otherwise a TDZ error can be introduced
    test_same("function f(b) { let a; return a = c(), b }");
    // Annex B initializer in a for-in head is evaluated before the right hand side
    test_same("function f() { var a; for (var x = (a = 1) in (a = 2, obj)) foo() }");
    // loop tests are re-evaluated per iteration, so `a = 1` must stay in place
    test(
        "function f() { var a; while (a = 1, c) foo() }",
        "function f() { for (var a; a = 1, c;) foo() }",
    );
}
