/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/CollapseVariableDeclarationsTest.java>
#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    mod join_vars {
        use super::{test, test_same};

        #[test]
        fn test_collapsing() {
            // Basic collapsing
            test("var a;var b;", "var a,b;");

            // With initial values
            test("var a = 1;var b = 1;", "var a=1,b=1;");

            // Already collapsed
            test_same("var a, b;");

            // Already collapsed with values
            test_same("var a = 1, b = 1;");

            // Some already collapsed
            test("var a;var b, c;var d;", "var a,b,c,d;");

            // Some already collapsed with values
            test("var a = 1;var b = 2, c = 3;var d = 4;", "var a=1,b=2,c=3,d=4;");

            test(
                "var x = 2; foo(x); x = 3; x = 1; var y = 2; var z = 4; x = 5",
                "var x = 2; foo(x), x = 3, x = 1; var y = 2, z = 4; x = 5",
            );
        }

        #[test]
        fn test_issue820() {
            // Don't redeclare function parameters, this is incompatible with
            // strict mode.
            test_same("function f(a){ var b=1; a=2; var c; }");
        }

        #[test]
        fn test_if_else_var_declarations() {
            test_same("if (x) var a = 1; else var b = 2;");
        }

        #[test]
        fn test_aggressive_redeclaration_in_for() {
            test_same("for(var x = 1; x = 2; x = 3) x = 4");
            test_same("for(var x = 1; y = 2; z = 3) {var a = 4}");
            test_same("var x; for(x = 1; x = 2; z = 3) x = 4");
        }

        #[test]
        fn test_issue397() {
            test_same("var x; x = 5; var z = 7;");
            test("var x; var y = 3; x = 5;", "var x, y = 3; x = 5;");
            test("var a = 1; var x; var y = 3; x = 5;", "var a = 1, x, y = 3; x = 5;");
            test("var x; var y = 3; x = 5; var z = 7;", "var x, y = 3; x = 5; var z = 7;");
        }

        #[test]
        fn test_arguments_assignment() {
            test_same("function f() {arguments = 1;}");
        }

        // ES6 Tests
        #[test]
        fn test_collapsing_let_const() {
            // Basic collapsing
            test("let a;let b;", "let a,b;");

            // With initial values
            test("const a = 1;const b = 1;", "const a=1,b=1;");

            // Already collapsed
            test_same("let a, b;");

            // Already collapsed with values
            test_same("let a = 1, b = 1;");

            // Some already collapsed
            test("let a;let b, c;let d;", "let a,b,c,d;");

            // Some already collapsed with values
            test("let a = 1;let b = 2, c = 3;let d = 4;", "let a=1,b=2,c=3,d=4;");

            // Different variable types
            test_same("let a = 1; const b = 2;");
        }

        #[test]
        fn test_if_else_var_declarations_let() {
            test_same("if (x) { let a = 1; } else { let b = 2; }");
        }

        #[test]
        fn test_aggressive_redeclaration_of_let_in_for() {
            test_same("for(let x = 1; x = 2; x = 3) x = 4");
            test_same("for(let x = 1; y = 2; z = 3) {let a = 4}");
            test_same("let x; for(x = 1; x = 2; z = 3) x = 4");
        }

        #[test]
        fn test_redeclaration_let_in_function() {
            test(
                "function f() { let x = 1; let y = 2; let z = 3; x + y + z; }",
                "function f() { let x = 1, y = 2, z = 3; x + y + z; } ",
            );

            // recognize local scope version of x
            test(
                "var x = 1; function f() { let x = 1; let y = 2; x + y; }",
                "var x = 1; function f() { let x = 1, y = 2; x + y } ",
            );

            // do not redeclare function parameters
            // incompatible with strict mode
            test_same("function f(x) { let y = 3; x = 4, x + y; }");
        }

        #[test]
        fn test_arrow_function() {
            test(
                "(() => { let x = 1; let y = 2; x + y; })()",
                "(() => { let x = 1, y = 2; x + y; })()",
            );

            // do not redeclare function parameters
            // incompatible with strict mode
            test_same("((x) => { x = 4; let y = 2; x + y; })()");
        }

        #[test]
        fn test_uncollapsable_declarations() {
            test_same("let x = 1; var y = 2; const z = 3");
            test_same("let x = 1; var y = 2; let z = 3;");
        }

        #[test]
        fn test_mixed_declaration_types() {
            // lets, vars, const declarations consecutive
            test("let x = 1; let z = 3; var y = 2;", "let x = 1, z = 3; var y = 2;");
            test(
                "let x = 1; let y = 2; var z = 3; var a = 4;",
                "let x = 1, y = 2; var z = 3, a = 4",
            );
        }
    }

    /// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/DenormalizeTest.java>
    #[cfg(test)]
    mod collapse_for {
        use super::{test, test_same};

        #[test]
        fn test_for() {
            // Verify assignments are moved into the FOR init node.
            test("a = 0; for(; a < 2 ; a++) foo()", "for(a = 0; a < 2 ; a++) foo();");
            // Verify vars are are moved into the FOR init node.
            test("var a = 0; for(; c < b ; c++) foo()", "for(var a = 0; c < b ; c++) foo()");
            test(
                "var a = 0; var b = 0; for(; c < b ; c++) foo()",
                "for(var a = 0, b = 0; c < b ; c++) foo()",
            );

            // We don't handle labels yet.
            test_same("var a = 0; a:for(; c < b ; c++) foo()");
            test_same("var a = 0; a:b:for(; c < b ; c++) foo()");

            // Do not inline let or const
            test_same("let a = 0; for(; c < b ; c++) foo()");
            test_same("const a = 0; for(; c < b ; c++) foo()");

            // Verify FOR inside IFs.
            test(
                "if(x){var a = 0; for(; c < b; c++) foo()}",
                "if(x)for(var a = 0; c < b; c++) foo()",
            );

            // Any other expression.
            test("init(); for(; a < 2 ; a++) foo()", "for(init(); a < 2 ; a++) foo();");

            // Other statements are left as is.
            test(
                "function f(){ var a; for(; a < 2 ; a++) foo() }",
                "function f(){ for(var a; a < 2 ; a++) foo() }",
            );
            test_same("function f(){ for(; a < 2 ; a++) foo() }");

            // TODO
            // Verify destructuring assignments are moved.
            // test(
            // "[a, b] = [1, 2]; for (; a < 2; a = b++) foo();",
            // "for ([a, b] = [1, 2]; a < 2; a = b++) foo();",
            // );

            // test(
            // "var [a, b] = [1, 2]; for (; a < 2; a = b++) foo();",
            // "var a; var b; for ([a, b] = [1, 2]; a < 2; a = b++) foo();",
            // );
        }

        #[test]
        fn test_for_in() {
            test("var a; for(a in b) foo()", "for (var a in b) foo()");
            test("a = 0; for(a in b) foo()", "for (a in a = 0, b) foo();");
            test_same("var a = 0; for(a in b) foo()");

            // We don't handle labels yet.
            test_same("var a; a:for(a in b) foo()");
            test_same("var a; a:b:for(a in b) foo()");

            // Verify FOR inside IFs.
            test("if(x){var a; for(a in b) foo()}", "if(x) for(var a in b) foo()");

            // Any other expression.
            test("init(); for(a in b) foo()", "for (a in init(), b) foo();");

            // Other statements are left as is.
            test_same("function f(){ for(a in b) foo() }");

            // We don't handle destructuring patterns yet.
            test("var a; var b; for ([a, b] in c) foo();", "var a, b; for ([a, b] in c) foo();");
        }

        #[test]
        fn test_for_of() {
            test("var a; for (a of b) foo()", "for (var a of b) foo()");
            test_same("a = 0; for (a of b) foo()");
            test_same("var a = 0; for (a of b) foo()");

            // We don't handle labels yet.
            test_same("var a; a: for (a of b) foo()");
            test_same("var a; a: b: for (a of b) foo()");

            // Verify FOR inside IFs.
            test("if (x) { var a; for (a of b) foo() }", "if (x) for (var a of b) foo()");

            // Any other expression.
            test_same("init(); for (a of b) foo()");

            // Other statements are left as is.
            test_same("function f() { for (a of b) foo() }");

            // We don't handle destructuring patterns yet.
            test("var a; var b; for ([a, b] of c) foo();", "var a, b; for ([a, b] of c) foo();");
        }
    }
}
