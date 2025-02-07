#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[test]
    #[ignore]
    fn test_break_optimization() {
        test("f:{if(true){a();break f;}else;b();}", "f:{if(true){a()}else{b()}}");
        test("f:{if(false){a();break f;}else;b();break f;}", "f:{if(false){a()}else{b()}}");
        test("f:{if(a()){b();break f;}else;c();}", "f:{if(a()){b();}else{c();}}");
        test("f:{if(a()){b()}else{c();break f;}}", "f:{if(a()){b()}else{c();}}");
        test("f:{if(a()){b();break f;}else;}", "f:{if(a()){b();}else;}");
        test("f:{if(a()){break f;}else;}", "f:{if(a()){}else;}");

        test("f:while(a())break f;", "f:while(a())break f");
        test_same("f:for(x in a())break f");

        test_same("f:{while(a())break;}");
        test_same("f:{for(x in a())break}");

        test("f:try{break f;}catch(e){break f;}", "f:try{}catch(e){}");
        test(
            "f:try{if(a()){break f;}else{break f;} break f;}catch(e){}",
            "f:try{if(a()){}else{}}catch(e){}",
        );

        test("f:g:break f", "");
        test("f:g:{if(a()){break f;}else{break f;} break f;}", "f:g:{if(a()){}else{}}");
        test("function f() { a: break a; }", "function f() {}");
        test("function f() { a: { break a; } }", "function f() { a: {} }");
    }

    #[test]
    fn test_function_return_optimization1() {
        test("function f(){return}", "function f(){}");
    }

    #[test]
    #[ignore]
    fn test_function_return_optimization2() {
        test("function f(){if(a()){b();if(c())return;}}", "function f(){if(a()){b();if(c());}}");
        test("function f(){if(x)return; x=3; return; }", "function f(){if(x); else x=3}");
        test(
            "function f(){if(true){a();return;}else;b();}",
            "function f(){if(true){a();}else{b();}}",
        );
        test(
            "function f(){if(false){a();return;}else;b();return;}",
            "function f(){if(false){a();}else{b();}}",
        );
        test(
            "function f(){if(a()){b();return;}else;c();}",
            "function f(){if(a()){b();}else{c();}}",
        );
        test("function f(){if(a()){b()}else{c();return;}}", "function f(){if(a()){b()}else{c();}}");
        test("function f(){if(a()){b();return;}else;}", "function f(){if(a()){b();}else;}");
        test(
            "function f(){if(a()){return;}else{return;} return;}",
            "function f(){if(a()){}else{}}",
        );
        test(
            "function f(){if(a()){return;}else{return;} b();}",
            "function f(){if(a()){}else{return;b()}}",
        );
        test(
            "function f(){ if (x) return; if (y) return; if (z) return; w(); }",
            "function f() {
              if (x) {} else { if (y) {} else { if (z) {} else w(); }}
            }",
        );

        test("function f(){while(a())return;}", "function f(){while(a())return}");
        test_same("function f(){for(x in a())return}");

        test("function f(){while(a())break;}", "function f(){while(a())break}");
        test_same("function f(){for(x in a())break}");

        test(
            "function f(){try{return;}catch(e){throw 9;}finally{return}}",
            "function f(){try{}catch(e){throw 9;}finally{return}}",
        );
        test_same("function f(){try{throw 9;}finally{return;}}");

        test("function f(){try{return;}catch(e){return;}}", "function f(){try{}catch(e){}}");
        test(
            "function f(){try{if(a()){return;}else{return;} return;}catch(e){}}",
            "function f(){try{if(a()){}else{}}catch(e){}}",
        );

        test("function f(){g:return}", "function f(){}");
        test(
            "function f(){g:if(a()){return;}else{return;} return;}",
            "function f(){g:if(a()){}else{}}",
        );
        test(
            "function f(){try{g:if(a()){throw 9;} return;}finally{return}}",
            "function f(){try{g:if(a()){throw 9;}}finally{return}}",
        );
    }

    #[test]
    fn test_function_return_scoped() {
        test(
            "function f(a) {
              if (a) {
                let a = Math.random();
                if (a < 0.5) {
                    return a;
                }
              }
              return a;
            }",
            "function f(a) {
              if (a) {
                let a = Math.random();
                if (a < 0.5) return a;
              }
              return a;
            }",
        );
    }

    #[test]
    #[ignore]
    fn test_while_continue_optimization() {
        test("while(true){if(x)continue; x=3; continue; }", "while(true)if(x);else x=3");
        test_same("while(true){a();continue;b();}");
        test(
            "while(true){if(true){a();continue;}else;b();}",
            "while(true){if(true){a();}else{b()}}",
        );
        test(
            "while(true){if(false){a();continue;}else;b();continue;}",
            "while(true){if(false){a()}else{b();}}",
        );
        test(
            "while(true){if(a()){b();continue;}else;c();}",
            "while(true){if(a()){b();}else{c();}}",
        );
        test(
            "while(true){if(a()){b();}else{c();continue;}}",
            "while(true){if(a()){b();}else{c();}}",
        );
        test("while(true){if(a()){b();continue;}else;}", "while(true){if(a()){b();}else;}");
        test(
            "while(true){if(a()){continue;}else{continue;} continue;}",
            "while(true){if(a()){}else{}}",
        );
        test(
            "while(true){if(a()){continue;}else{continue;} b();}",
            "while(true){if(a()){}else{continue;b();}}",
        );

        test("while(true)while(a())continue;", "while(true)while(a());");
        test("while(true)for(x in a())continue", "while(true)for(x in a());");

        test("while(true)while(a())break;", "while(true)while(a())break");
        test_same("while(true)for(x in a())break");

        test("while(true){try{continue;}catch(e){continue;}}", "while(true){try{}catch(e){}}");
        test(
            "while(true){try{if(a()){continue;}else{continue;} continue;}catch(e){}}",
            "while(true){try{if(a()){}else{}}catch(e){}}",
        );

        test("while(true){g:continue}", "while(true){}");
        // This case could be improved.
        test(
            "while(true){g:if(a()){continue;}else{continue;} continue;}",
            "while(true){g:if(a());else;}",
        );
    }

    #[test]
    #[ignore]
    fn test_do_continue_optimization() {
        test("do{if(x)continue; x=3; continue; }while(true)", "do if(x); else x=3; while(true)");
        test_same("do{a();continue;b()}while(true)");
        test(
            "do{if(true){a();continue;}else;b();}while(true)",
            "do{if(true){a();}else{b();}}while(true)",
        );
        test(
            "do{if(false){a();continue;}else;b();continue;}while(true)",
            "do{if(false){a();}else{b();}}while(true)",
        );
        test(
            "do{if(a()){b();continue;}else;c();}while(true)",
            "do{if(a()){b();}else{c()}}while(true)",
        );
        test(
            "do{if(a()){b();}else{c();continue;}}while(true)",
            "do{if(a()){b();}else{c();}}while(true)",
        );
        test("do{if(a()){b();continue;}else;}while(true)", "do{if(a()){b();}else;}while(true)");
        test(
            "do{if(a()){continue;}else{continue;} continue;}while(true)",
            "do{if(a()){}else{}}while(true)",
        );
        test(
            "do{if(a()){continue;}else{continue;} b();}while(true)",
            "do{if(a()){}else{continue; b();}}while(true)",
        );

        test("do{while(a())continue;}while(true)", "do while(a());while(true)");
        test("do{for(x in a())continue}while(true)", "do for(x in a());while(true)");

        test("do{while(a())break;}while(true)", "do while(a())break;while(true)");
        test_same("do for(x in a())break;while(true)");

        test("do{try{continue;}catch(e){continue;}}while(true)", "do{try{}catch(e){}}while(true)");
        test(
            "do{try{if(a()){continue;}else{continue;} continue;}catch(e){}}while(true)",
            "do{try{if(a()){}else{}}catch(e){}}while(true)",
        );

        test("do{g:continue}while(true)", "do{}while(true)");
        // This case could be improved.
        test(
            "do{g:if(a()){continue;}else{continue;} continue;}while(true)",
            "do{g:if(a());else;}while(true)",
        );

        test("do { foo(); continue; } while(false)", "do { foo(); } while(false)");
        test("do { foo(); break; } while(false)", "do { foo(); } while(false)");

        test("do{break}while(!new Date());", "do{}while(!new Date());");

        test_same("do { foo(); switch (x) { case 1: break; default: f()}; } while(false)");
    }

    #[test]
    #[ignore]
    fn test_for_continue_optimization() {
        test("for(x in y){if(x)continue; x=3; continue; }", "for(x in y)if(x);else x=3");
        test_same("for(x in y){a();continue;b()}");
        test("for(x in y){if(true){a();continue;}else;b();}", "for(x in y){if(true)a();else b();}");
        test(
            "for(x in y){if(false){a();continue;}else;b();continue;}",
            "for(x in y){if(false){a();}else{b()}}",
        );
        test(
            "for(x in y){if(a()){b();continue;}else;c();}",
            "for(x in y){if(a()){b();}else{c();}}",
        );
        test(
            "for(x in y){if(a()){b();}else{c();continue;}}",
            "for(x in y){if(a()){b();}else{c();}}",
        );

        test("for(x of y){if(x)continue; x=3; continue; }", "for(x of y)if(x);else x=3");
        test_same("for(x of y){a();continue;b()}");
        test("for(x of y){if(true){a();continue;}else;b();}", "for(x of y){if(true)a();else b();}");
        test(
            "for(x of y){if(false){a();continue;}else;b();continue;}",
            "for(x of y){if(false){a();}else{b()}}",
        );
        test(
            "for(x of y){if(a()){b();continue;}else;c();}",
            "for(x of y){if(a()){b();}else{c();}}",
        );
        test(
            "for(x of y){if(a()){b();}else{c();continue;}}",
            "for(x of y){if(a()){b();}else{c();}}",
        );

        test(
            "async () => { for await (x of y){if(x)continue; x=3; continue; }}",
            "async () => { for await (x of y)if(x);else x=3 }",
        );
        test_same("async () => { for await (x of y){a();continue;b()}}");
        test(
            "async () => { for await (x of y){if(true){a();continue;}else;b();}}",
            "async () => { for await (x of y){if(true)a();else b();}}",
        );
        test(
            "async () => { for await (x of y){if(false){a();continue;}else;b();continue;}}",
            "async () => { for await (x of y){if(false){a();}else{b()}}}",
        );
        test(
            "async () => { for await (x of y){if(a()){b();continue;}else;c();}}",
            "async () => { for await (x of y){if(a()){b();}else{c();}}}",
        );
        test(
            "async () => { for await (x of y){if(a()){b();}else{c();continue;}}}",
            "async () => { for await (x of y){if(a()){b();}else{c();}}}",
        );

        test(
            "for(x=0;x<y;x++){if(a()){b();continue;}else;}",
            "for(x=0;x<y;x++){if(a()){b();}else;}",
        );
        test(
            "for(x=0;x<y;x++){if(a()){continue;}else{continue;} continue;}",
            "for(x=0;x<y;x++){if(a()){}else{}}",
        );
        test(
            "for(x=0;x<y;x++){if(a()){continue;}else{continue;} b();}",
            "for(x=0;x<y;x++){if(a()){}else{continue; b();}}",
        );

        test("for(x=0;x<y;x++)while(a())continue;", "for(x=0;x<y;x++)while(a());");
        test("for(x=0;x<y;x++)for(x in a())continue", "for(x=0;x<y;x++)for(x in a());");

        test("for(x=0;x<y;x++)while(a())break;", "for(x=0;x<y;x++)while(a())break");
        test_same("for(x=0;x<y;x++)for(x in a())break");

        test(
            "for(x=0;x<y;x++){try{continue;}catch(e){continue;}}",
            "for(x=0;x<y;x++){try{}catch(e){}}",
        );
        test(
            "for(x=0;x<y;x++){try{if(a()){continue;}else{continue;} continue;}catch(e){}}",
            "for(x=0;x<y;x++){try{if(a()){}else{}}catch(e){}}",
        );

        test("for(x=0;x<y;x++){g:continue}", "for(x=0;x<y;x++){}");
        test(
            "for(x=0;x<y;x++){g:if(a()){continue;}else{continue;} continue;}",
            "for(x=0;x<y;x++){g:if(a());else;}",
        );
    }

    #[test]
    #[ignore]
    fn test_code_motion_doesnt_break_function_hoisting() {
        test(
            "function f() { if (x) return; foo(); function foo() {} }",
            "function f() { if (x); else { function foo() {} foo(); } }",
        );
    }

    #[test]
    fn test_dont_remove_break_in_try_finally() {
        test_same("function f() {b:try{throw 9} finally {break b} return 1;}");
    }

    /**
     * The 'break' prevents the 'b=false' from being evaluated. If we test the do-while to
     * 'do;while(b=false)' the code will be incorrect.
     *
     * @see https://github.com/google/closure-compiler/issues/554
     */
    #[test]
    fn test_dont_test_break_in_do_while_if_condition_has_side_effects() {
        test("var b=true;do{break}while(b=false);", "var b = !0; do break; while (b = !1);");
    }

    #[test]
    #[ignore]
    fn test_switch_exit_points1() {
        test("switch (x) { case 1: f(); break; }", "switch (x) { case 1: f();        }");
        test(
            "switch (x) { case 1: f(); break; case 2: g(); break; }",
            "switch (x) { case 1: f(); break; case 2: g();        }",
        );
        test(
            "switch (x) { case 1: if (x) { f(); break; } break; default: g(); break; }",
            "switch (x) { case 1: if (x) { f();        } break; default: g();        }",
        );
    }

    #[test]
    #[ignore]
    fn test_test_block_scoped_variables() {
        // When moving block-scoped variable declarations into inner blocks, first convert them to
        // "var" declarations to avoid breaking any references in inner functions.

        // For example, in the following test case, moving "let c = 3;" directly inside the else block
        // would break the function "g"'s reference to "c".
        test(
            "function f() { function g() { return c; } if (x) {return;} let c = 3; }",
            "function f() { function g() { return c; } if (x){} else {var c = 3;} }",
        );
        test(
            "function f() { function g() { return c; } if (x) {return;} const c = 3; }",
            "function f() { function g() { return c; } if (x) {} else {var c = 3;} }",
        );
        // Convert let and const even they're if not referenced by any functions.
        test(
            "function f() { if (x) {return;} const c = 3; }",
            "function f() { if (x) {} else { var c = 3; } }",
        );
        test(
            "function f() { if (x) {return;} let a = 3; let b = () => a; }",
            "function f() { if (x) {} else { var a = 3; var b = () => a;} }",
        );
        test(
            "function f() { if (x) { if (y) {return;} let c = 3; } }",
            "function f() { if (x) { if (y) {} else { var c = 3; } } }",
        );
    }

    #[test]
    #[ignore]
    fn test_dont_test_block_scoped_variables_in_loops() {
        // Don't move block-scoped declarations into inner blocks inside a loop, since converting
        // let/const declarations to vars in a loop can cause incorrect semantics.
        // See the following test case for an example.
        test_same(
            "function f(param) {
              let arr = [];
              for (let x of param) {
                if (x < 0) continue;
                let y = x * 2;
                arr.push(() => y); // If y was a var, this would capture the wrong value.
               }
              return arr;
            }",
        );

        // Additional tests for different kinds of loops.
        test_same("function f() { while (true) { if (true) {return;} let c = 3; } }");
        test_same("function f() { do { if (true) {return;} let c = 3; } while (x); }");
        test_same("function f() { for (;;) { if (true) { return; } let c = 3; } }");
        test_same("function f(y) { for(x in []){ if(x) { return; } let c = 3; } }");
        test_same("async function f(y) { for await (x in []){ if(x) { return; } let c = 3; } }");
    }
}
