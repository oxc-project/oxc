use crate::{test, test_same};

#[test]
#[ignore = "TODO: Labelled break optimization not yet implemented"]
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
fn test_function_return_optimization() {
    test("function f(){return}", "function f(){}");
    test("function f(){if(a())return;}", "function f(){a()}");
    test("function f(){if(a()){b();if(c())return;}}", "function f(){a()&&(b(),c())}");
    test("function f(){try{return;}catch(e){return;}}", "function f(){}");
    test("function f(){if(a()){return;}else{return;} return;}", "function f(){a()}");
    test_same("function f(){try{return;}finally{g()}}");
    test("function f(){if(x)return; x=3; return; }", "function f(){x ||= 3;}");
    test("function f(){if(true){a();return;}else;b();}", "function f(){a();}");
    test("function f(){if(false){a();return;}else;b();return;}", "function f(){b();}");
    test("function f(){if(a()){b()}else{c();return;}}", "function f(){a() ? b() : c();}");
    test("function f(){if(a()){b();return;}else;}", "function f(){a() && b();}");
    test("function f(){if(a()){return;}else{return;} b();}", "function f(){ a();}");
    test("function f(){a:{return;}}", "function f(){}");
    test(
        "function f(){ if (x) return; if (y) return; if (z) return; w(); }",
        "function f() { x || y || z || w(); }",
    );
    test("function f(){while(a())return;}", "function f(){for(;a();)return}");
    test_same("function f(){for(x in a())return}");
    test("function f(){while(a())break;}", "function f(){for(;a();)break}");
    test_same("function f(){for(x in a())break}");

    test("function f(){g:return}", "function f(){}");
    test("function f(){g:if(a()){return;}else{return;} return;}", "function f(){g:a();}");
}

#[test]
#[ignore = "TODO: Function return optimization not yet implemented"]
fn test_function_return_optimization_todo() {
    test("function f(){if(a()){b();return;}else;c();}", "function f(){if(a()){b();}else{c();}}");

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
fn test_while_continue_optimization() {
    test("while(a())continue;", "for(;a(););");
    test("while(true){a();continue;}", "for (;;) a();");
    test("while(true){x:continue x;}", "for (;;) x: continue x;");
    test("while(true){if(x)continue; x=3; continue; }", "for(;;)x||=3");
    test("while(true){a();continue;b();}", "for (;;) a();");
    test("while(true){if(true){a();continue;}else;b();}", "for(;;)a();");
    test("while(true){if(false){a();continue;}else;b();continue;}", "for (;;) b();");
    test("while(true)while(a())continue;", "for(;;)for(;a(););");
    test("while(true)for(x in a())continue", "for(;;)for(x in a());");
    test("while(true)while(a())break;", "for(;;)for(;a();)break");
    test("while(true)for(x in a())break", "for(;;)for(x in a())break");
    test("while(true){if(a()){b();continue;}else;c();}", "for(;;){if(a()){b();continue;}c()}");
    test("while(true){if(a()){b();}else{c();continue;}}", "for (;;) a() ? b() : c();");
    test("while(true){if(a()){b();continue;}else;}", "for (;;) a() && b();");
    test("while(true){if(a()){continue;}else{continue;} continue;}", "for (;;) a();");
    test("while(true){if(a()){continue;}else{continue;} b();}", "for (;;) a();");

    test("while(true){try{continue;}catch(e){continue;}}", "for (;;);");
    test(
        "while(true){try{if(a()){continue;}else{continue;} continue;}catch(e){}}",
        "for(;;) try{a();}catch{}",
    );
}

#[test]
#[ignore = "TODO: While continue optimization not yet implemented"]
fn test_while_continue_optimization_todo() {
    test("while(true){g:continue}", "while(true){}");
    // This case could be improved.
    test(
        "while(true){g:if(a()){continue;}else{continue;} continue;}",
        "while(true){g:if(a());else;}",
    );
}

#[test]
fn test_do_continue_optimization() {
    test("do{a();continue;}while(x)", "do a();while(x);");
    test("do{if(x)continue; x=3; continue; }while(true)", "do{if(x)continue;x=3}while(!0)");
    test("do{a();continue;b()}while(true)", "do a();while(!0);");
    test("do{while(a())break;}while(true)", "do for(;a();)break;while(!0)");
    test("do for(x in a())break;while(true)", "do for(x in a())break;while(!0)");
    test("do{if(true){a();continue;}else;b();}while(true)", "do a(); while(!0)");
    test("do{if(false){a();continue;}else;b();continue;}while(true)", "do b();while(!0)");
    test("do{if(a()){b();}else{c();continue;}}while(true)", "do a()?b():c();while(!0)");
    test("do{if(a()){b();continue;}else;}while(true)", "do a() && b();while(!0)");
    test("do{if(a()){continue;}else{continue;} continue;}while(true)", "do a();while(!0)");
    test("do{if(a()){continue;}else{continue;} b();}while(true)", "do a();while(!0)");
    test("do{while(a())continue;}while(true)", "do for(;a(););while(!0)");
    test("do{for(x in a())continue}while(true)", "do for(x in a());while(!0)");
    test("do{try{continue;}catch(e){continue;}}while(true)", "do;while (!0);");
    test(
        "do{try{if(a()){continue;}else{continue;} continue;}catch(e){}}while(true)",
        "do try{a()}catch{} while(!0)",
    );
    test("do{foo();continue;}while(false)", "do foo();while(!1)");
    test("do{foo();break;}while(false)", "do foo();while(!1)");
    test("do{g:continue}while(true)", "do;while(!0);");
    test("do{g:if(a()){continue;}else{continue;} continue;}while(true)", "do g:a(); while(!0)");
    test("do{break}while(!new Date());", "do;while(!1);");
    test(
        "do { foo(); switch (x) { case 1: break; default: f()}; } while(!1)",
        "do switch (foo(), x) { case 1: break; default: f(); } while (!1);",
    );
}

#[test]
#[ignore = "TODO: Do-while continue optimization not yet implemented"]
fn test_do_continue_optimization_todo() {
    test("do{if(x)continue; x=3; continue; }while(true)", "do if(x); else x=3; while(true)");
    test("do{if(a()){b();continue;}else;c();}while(true)", "do{if(a()){b();}else{c()}}while(true)");
}

#[test]
fn test_for_continue_optimization() {
    test("for(;;){a();continue;}", "for (;;) a();");
    test("for(x in y){if(x)continue; x=3; continue; }", "for(x in y)x||=3");
    test("for(x in y){a();continue;b()}", "for(x in y)a();");
    test("for(x in y){if(x)continue; x=3; continue; }", "for (x in y) x ||= 3");
    test("for(x in y){if(true){a();continue;}else;b();}", "for (x in y) a()");
    test("for(x in y){if(false){a();continue;}else;b();continue;}", "for(x in y) b()");
    test("for(x in y){if(a()){b();}else{c();continue;}}", "for(x in y) a()?b():c()");
    test("for(x of y){if(x)continue; x=3; continue; }", "for(x of y) x||=3");
    test("for(x of y){a();continue;b()}", "for(x of y) a()");
    test("for(x of y){if(true){a();continue;}else;b();}", "for(x of y) a()");
    test("for(x of y){if(a()){b();}else{c();continue;}}", "for(x of y) a()?b():c()");

    test("for(x=0;x<y;x++){if(a()){b();continue;}else;}", "for(x=0;x<y;x++) a()&&b()");
    test("for(x=0;x<y;x++){if(a()){continue;}else{continue;} continue;}", "for(x=0;x<y;x++) a()");
    test("for(x=0;x<y;x++){if(a()){continue;}else{continue;} b();}", "for(x=0;x<y;x++) a()");

    test("for(x=0;x<y;x++)while(a())continue;", "for(x=0;x<y;x++)for(;a(););");
    test("for(x=0;x<y;x++)for(x in a())continue", "for(x=0;x<y;x++)for(x in a());");

    test("for(x=0;x<y;x++)while(a())break;", "for(x=0;x<y;x++)for(;a();)break");
    test_same("for(x=0;x<y;x++)for(x in a())break");

    test("for(x=0;x<y;x++){try{continue;}catch(e){continue;}}", "for(x=0;x<y;x++);");
    test(
        "for(x=0;x<y;x++){try{if(a()){continue;}else{continue;} continue;}catch(e){}}",
        "for(x=0;x<y;x++) try{a();}catch {}",
    );
    test("for(x of y){if(false){a();continue;}else;b();continue;}", "for(x of y) b()");

    test(
        "(async ()=>{ for await (x of y){if(x)continue; x=3; continue; }})()",
        "(async ()=>{ for await (x of y) x ||= 3; })()",
    );
    test(
        "(async ()=>{ for await (x of y){a();continue;b()}})()",
        "(async ()=>{ for await (x of y) a(); })()",
    );
    test(
        "(async ()=>{ for await (x of y){if(true){a();continue;}else;b();}})()",
        "(async ()=>{ for await (x of y) a(); })()",
    );
    test(
        "(async ()=>{ for await (x of y){if(false){a();continue;}else;b();continue;}})()",
        "(async ()=>{ for await (x of y) b(); })()",
    );
    test(
        "(async ()=>{ for await (x of y){if(a()){b();}else{c();continue;}}})()",
        "(async ()=>{ for await (x of y) a()?b():c(); })()",
    );
    test("for(x=0;x<y;x++){g:continue}", "for(x=0;x<y;x++);");
    test(
        "for(x=0;x<y;x++){g:if(a()){continue;}else{continue;} continue;}",
        "for(x=0;x<y;x++) g: a();",
    );
}

#[test]
#[ignore = "TODO: For loop continue optimization not yet implemented"]
fn test_for_continue_optimization_todo() {
    test("for(x in y){if(a()){b();continue;}else;c();}", "for(x in y){if(a()){b();}else{c();}}");

    test("for(x of y){if(false){a();continue;}else;b();continue;}", "for(x of y) b()");
    test("for(x of y){if(a()){b();continue;}else;c();}", "for(x of y){if(a()){b();}else{c();}}");

    test(
        "(async ()=>{ for await (x of y){if(a()){b();continue;}else;c();}})()",
        "(async ()=>{ for await (x of y) a()?b():c(); })()",
    );
}

#[test]
#[ignore = "TODO: Code motion with function hoisting not yet implemented"]
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
fn test_switch_exit_points() {
    test("switch (x) { case 1: f(); break; }", "switch (x) { case 1: f();        }");
    test("function f(){switch(x){case 1:return;}}", "function f(){switch(x){case 1:}}");
    test("function f(){switch(x){case 1:break;}}", "function f(){switch(x){case 1:}}");
    test(
        "function f(){switch(x){case 1:return;case 2:return;}}",
        "function f(){switch(x){case 1:return;case 2:}}",
    );
}

#[test]
// #[ignore = "TODO: Switch exit point optimization not yet implemented"]
fn test_switch_exit_points_todo() {
    test(
        "switch (x) { case 1: if (x) { f(); break; } break; default: g(); break; }",
        "switch (x) { case 1: if (x) { f();        } break; default: g();        }",
    );
}

#[test]
#[ignore = "TODO: Block scoped variable optimization not yet implemented"]
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
#[ignore = "TODO: Block scoped variables in loops optimization not yet implemented"]
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
