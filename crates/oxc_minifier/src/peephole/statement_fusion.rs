/// Statement Fusion
///
/// Tries to fuse all the statements in a block into a one statement by using COMMAs or statements.
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/StatementFusion.java>
#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};
    #[test]
    fn fold_block_with_statements() {
        test("a;b;c", "a,b,c");
        test("a();b();c();", "a(),b(),c()");
        test("a(),b();c(),d()", "a(),b(),c(),d()");
        test("a();b(),c(),d()", "a(),b(),c(),d()");
        test("a(),b(),c();d()", "a(),b(),c(),d()");
    }

    #[test]
    fn fold_block_into_if() {
        test("a;b;c;if(x){}", "a,b,c,x");
        test("a;b;c;if(x,y){}else{}", "a, b, c, x, y");
        test("a;b;c;if(x,y){}", "a, b, c, x, y");
        test("a;b;c;if(x,y,z){}", "a, b, c, x, y, z");
        test("a();if(a()){}a()", "a(), a(), a()");
    }

    #[test]
    fn fold_block_return() {
        test("a;b;c;return x", "return a,b,c,x");
        test("a;b;c;return x+y", "return a,b,c,x+y");
        test("a();b();c();return x();a();b();c()", "return a(),b(),c(),x()");
    }

    #[test]
    fn fold_block_throw() {
        test("a;b;c;throw x", "throw a,b,c,x");
        test("a;b;c;throw x+y", "throw a,b,c,x+y");
        test("a();b();c();throw x();a();b();c", "throw a(),b(),c(),x()");
    }

    #[test]
    fn fold_switch() {
        test("a;b;c;switch(x){}", "switch(a,b,c,x){}");
    }

    #[test]
    fn fuse_into_for_in1() {
        test("a;b;c;for(x in y){}", "for(x in a,b,c,y);");
    }

    #[test]
    fn fuse_into_for_in2() {
        // this should not be compressed into `for (var x = a() in b(), [0])`
        // as the side effect order of `a()` and `b()` changes
        test_same("a();for(var x = b() in y);");
        test("a = 1; for(var x = 2 in y);", "for(var x = 2 in a = 1, y);");
        // this can be compressed because b() runs after a()
        test("a(); for (var { x = b() } in y);", "for (var { x = b() } in a(), y);");
    }

    #[test]
    fn fuse_into_vanilla_for1() {
        test("a;b;c;for(;g;){}", "for(a,b,c;g;);");
        test("a;b;c;for(d;g;){}", "for(a,b,c,d;g;);");
        test("a;b;c;for(d,e;g;){}", "for(a,b,c,d,e;g;);");
        test_same("a();for(var x;g;);");
    }

    #[test]
    fn fuse_into_vanilla_for2() {
        test("a;b;c;for(var d;g;){}", "a,b,c;for(var d;g;);");
        test("a;b;c;for(let d;g;){}", "a,b,c;for(let d;g;);");
        test("a;b;c;for(const d = 5;g;){}", "a,b,c;for(let d = 5;g;);");
    }

    #[test]
    #[ignore]
    fn fuse_into_label() {
        test("a;b;c;label:for(x in y){}", "label:for(x in a,b,c,y);");
        test("a;b;c;label:for(;g;){}", "label:for(a,b,c;g;);");
        test("a;b;c;l1:l2:l3:for(;g;){}", "l1:l2:l3:for(a,b,c;g;);");
        test("a;b;c;label:while(true){}", "label:for(a,b,c;;);");
    }

    #[test]
    #[ignore]
    fn fuse_into_block() {
        test("a;b;c;{d;e;f}", "a,b,c,d,e,f");
        test(
            "a;b; label: { if(q) break label; bar(); }",
            "label: { if(a,b,q) break label; bar(); }",
        );
        test("a;b;c;{var x;d;e;}", "a,b,c;{var x;d,e;}");
        test("a;b;c;label:{break label;d;e;}", "a,b,c");
    }

    #[test]
    fn fuse_into_switch_cases() {
        test("switch (_) { case _: a; return b }", "switch (_) { case _: return a, b }");
    }

    #[test]
    fn no_fuse_into_while() {
        test("a;b;c;while(x){}", "for(a,b,c;x;);");
    }

    #[test]
    fn no_fuse_into_do() {
        test("a;b;c;do;while(x)", "a,b,c;do;while(x)");
    }

    #[test]
    #[ignore]
    fn no_fuse_into_block() {
        // Never fuse a statement into a block that contains let/const/class declarations, or you risk
        // colliding variable names. (unless the AST is normalized).
        test("a; {b;}", "a,b");
        test("a; {b; var a = 1;}", "{a, b; var a = 1;}");
        test_same("a; { b; let a = 1; }");
        test("a; { b; const a = 1; }", "a; { b; let a = 1; }");
        test_same("a; { b; class a {} }");
        test_same("a; { b; function a() {} }");
        test("a; { b; const otherVariable = 1; }", "a; { b; let otherVariable = 1; }");

        // test(
        // "function f(a) { if (COND) { a; { b; let a = 1; } } }",
        // "function f(a) { if (COND) { { a,b; let a$jscomp$1 = 1; } } }",
        // );
        // test(
        // "function f(a) { if (COND) { a; { b; let otherVariable = 1; } } }",
        // "function f(a) { if (COND) {  { a,b; let otherVariable = 1; } } }",
        // );
    }

    #[test]
    fn no_global_scope_changes() {
        test_same("a,b,c");
    }

    #[test]
    fn no_function_block_changes() {
        test_same("function foo() { a,b,c }");
    }
}
