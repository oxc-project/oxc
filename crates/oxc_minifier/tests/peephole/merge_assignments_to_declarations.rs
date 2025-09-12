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
    test("let a, b; a = b", "let a, b; a = void 0"); // `let a = b, b` will cause TDZ error
    test_same("let a; a = foo(a)"); // `let a = foo(a)` will cause TDZ error
    test_same("let a; a = (() => a)()"); // `let a = (() => a)()` will cause TDZ error
    test("let a; a = () => a", "let a = () => a");
}

#[test]
fn merge_assignments_to_declarations_other() {
    test_same("const a = 0; a = 1");
    test_same("using a = 0; a = 1");
    test_same("await using a = 0; a = 1");
}
