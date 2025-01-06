use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_semantic::ScopeFlags;
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::CompressorPass;

/// Transform the structure of the AST so that the number of explicit exits
/// are minimized and instead flows to implicit exits conditions.
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/MinimizeExitPoints.java>
pub struct MinimizeExitPoints {
    pub(crate) changed: bool,
}

impl<'a> CompressorPass<'a> for MinimizeExitPoints {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for MinimizeExitPoints {
    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.remove_last_return(&mut body.statements);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.fold_if_return(stmts, ctx);
    }
}

impl<'a> MinimizeExitPoints {
    pub fn new() -> Self {
        Self { changed: false }
    }

    // `function foo() { return }` -> `function foo() {}`
    fn remove_last_return(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if let Some(last) = stmts.last() {
            if matches!(last, Statement::ReturnStatement(ret) if ret.argument.is_none()) {
                stmts.pop();
                self.changed = true;
            }
        }
    }

    // `if(x)return;foo` -> `if(!x)foo;`
    fn fold_if_return(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        if stmts.len() <= 1 {
            return;
        }
        let Some(index) = stmts.iter().position(|stmt| {
            if let Statement::IfStatement(if_stmt) = stmt {
                if if_stmt.alternate.is_none()
                    && matches!(
                        if_stmt.consequent.get_one_child(),
                        Some(Statement::ReturnStatement(s)) if s.argument.is_none()
                    )
                {
                    return true;
                }
            }
            false
        }) else {
            return;
        };
        let Some(stmts_rest) = stmts.get_mut(index + 1..) else { return };
        let body = ctx.ast.vec_from_iter(stmts_rest.iter_mut().map(|s| ctx.ast.move_statement(s)));
        let Statement::IfStatement(if_stmt) = &mut stmts[index] else { unreachable!() };
        let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
        if_stmt.test = match ctx.ast.move_expression(&mut if_stmt.test) {
            Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                unary_expr.unbox().argument
            }
            e => ctx.ast.expression_unary(e.span(), UnaryOperator::LogicalNot, e),
        };
        if_stmt.alternate = None;
        if_stmt.consequent = Statement::BlockStatement(
            ctx.ast.alloc_block_statement_with_scope_id(SPAN, body, scope_id),
        );
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        self.changed = true;
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::tester;

    fn fold(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::MinimizeExitPoints::new();
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn fold_same(source_text: &str) {
        fold(source_text, source_text);
    }

    // oxc

    #[test]
    fn simple() {
        fold(
            "function foo() { if (foo) return; bar; quaz; }",
            "function foo() { if (!foo) { bar; quaz; } }",
        );
        fold(
            "function foo() { if (!foo) return; bar; quaz; }",
            "function foo() { if (foo) { bar; quaz; } }",
        );
        fold(
            "function foo() { x; if (foo) return; bar; quaz; }",
            "function foo() { x; if (!foo) { bar; quaz; } }",
        );
        fold(
            "function foo() { x; if (!foo) return; bar; quaz; }",
            "function foo() { x; if (foo) { bar; quaz; } }",
        );
        fold_same("function foo() { if (foo) return }");
        fold_same("function foo() { if (foo) return bar; baz }");
    }

    #[test]
    fn remove_last_return() {
        fold("function () {return}", "function () {}");
        fold("function () {a;b;return}", "function () {a;b;}");
        fold_same("function () { if(foo) { return } }");
    }

    // closure

    #[test]
    #[ignore]
    fn test_break_optimization() {
        fold("f:{if(true){a();break f;}else;b();}", "f:{if(true){a()}else{b()}}");
        fold("f:{if(false){a();break f;}else;b();break f;}", "f:{if(false){a()}else{b()}}");
        fold("f:{if(a()){b();break f;}else;c();}", "f:{if(a()){b();}else{c();}}");
        fold("f:{if(a()){b()}else{c();break f;}}", "f:{if(a()){b()}else{c();}}");
        fold("f:{if(a()){b();break f;}else;}", "f:{if(a()){b();}else;}");
        fold("f:{if(a()){break f;}else;}", "f:{if(a()){}else;}");

        fold("f:while(a())break f;", "f:while(a())break f");
        fold_same("f:for(x in a())break f");

        fold_same("f:{while(a())break;}");
        fold_same("f:{for(x in a())break}");

        fold("f:try{break f;}catch(e){break f;}", "f:try{}catch(e){}");
        fold(
            "f:try{if(a()){break f;}else{break f;} break f;}catch(e){}",
            "f:try{if(a()){}else{}}catch(e){}",
        );

        fold("f:g:break f", "");
        fold("f:g:{if(a()){break f;}else{break f;} break f;}", "f:g:{if(a()){}else{}}");
        fold("function f() { a: break a; }", "function f() {}");
        fold("function f() { a: { break a; } }", "function f() { a: {} }");
    }

    #[test]
    #[ignore]
    fn test_function_return_optimization1() {
        fold("function f(){return}", "function f(){}");
    }

    #[test]
    #[ignore]
    fn test_function_return_optimization2() {
        fold("function f(){if(a()){b();if(c())return;}}", "function f(){if(a()){b();if(c());}}");
        fold("function f(){if(x)return; x=3; return; }", "function f(){if(x); else x=3}");
        fold(
            "function f(){if(true){a();return;}else;b();}",
            "function f(){if(true){a();}else{b();}}",
        );
        fold(
            "function f(){if(false){a();return;}else;b();return;}",
            "function f(){if(false){a();}else{b();}}",
        );
        fold(
            "function f(){if(a()){b();return;}else;c();}",
            "function f(){if(a()){b();}else{c();}}",
        );
        fold("function f(){if(a()){b()}else{c();return;}}", "function f(){if(a()){b()}else{c();}}");
        fold("function f(){if(a()){b();return;}else;}", "function f(){if(a()){b();}else;}");
        fold(
            "function f(){if(a()){return;}else{return;} return;}",
            "function f(){if(a()){}else{}}",
        );
        fold(
            "function f(){if(a()){return;}else{return;} b();}",
            "function f(){if(a()){}else{return;b()}}",
        );
        fold(
            "function f(){ if (x) return; if (y) return; if (z) return; w(); }",
            "function f() {
              if (x) {} else { if (y) {} else { if (z) {} else w(); }}
            }",
        );

        fold("function f(){while(a())return;}", "function f(){while(a())return}");
        fold_same("function f(){for(x in a())return}");

        fold("function f(){while(a())break;}", "function f(){while(a())break}");
        fold_same("function f(){for(x in a())break}");

        fold(
            "function f(){try{return;}catch(e){throw 9;}finally{return}}",
            "function f(){try{}catch(e){throw 9;}finally{return}}",
        );
        fold_same("function f(){try{throw 9;}finally{return;}}");

        fold("function f(){try{return;}catch(e){return;}}", "function f(){try{}catch(e){}}");
        fold(
            "function f(){try{if(a()){return;}else{return;} return;}catch(e){}}",
            "function f(){try{if(a()){}else{}}catch(e){}}",
        );

        fold("function f(){g:return}", "function f(){}");
        fold(
            "function f(){g:if(a()){return;}else{return;} return;}",
            "function f(){g:if(a()){}else{}}",
        );
        fold(
            "function f(){try{g:if(a()){throw 9;} return;}finally{return}}",
            "function f(){try{g:if(a()){throw 9;}}finally{return}}",
        );
    }

    #[test]
    #[ignore]
    fn test_function_return_scoped() {
        fold_same(
            "function f(a) {
              if (a) {
                const a = Math.random();
                if (a < 0.5) {
                    return a;
                }
              }
              return a;
            }",
        );
    }

    #[test]
    #[ignore]
    fn test_while_continue_optimization() {
        fold("while(true){if(x)continue; x=3; continue; }", "while(true)if(x);else x=3");
        fold_same("while(true){a();continue;b();}");
        fold(
            "while(true){if(true){a();continue;}else;b();}",
            "while(true){if(true){a();}else{b()}}",
        );
        fold(
            "while(true){if(false){a();continue;}else;b();continue;}",
            "while(true){if(false){a()}else{b();}}",
        );
        fold(
            "while(true){if(a()){b();continue;}else;c();}",
            "while(true){if(a()){b();}else{c();}}",
        );
        fold(
            "while(true){if(a()){b();}else{c();continue;}}",
            "while(true){if(a()){b();}else{c();}}",
        );
        fold("while(true){if(a()){b();continue;}else;}", "while(true){if(a()){b();}else;}");
        fold(
            "while(true){if(a()){continue;}else{continue;} continue;}",
            "while(true){if(a()){}else{}}",
        );
        fold(
            "while(true){if(a()){continue;}else{continue;} b();}",
            "while(true){if(a()){}else{continue;b();}}",
        );

        fold("while(true)while(a())continue;", "while(true)while(a());");
        fold("while(true)for(x in a())continue", "while(true)for(x in a());");

        fold("while(true)while(a())break;", "while(true)while(a())break");
        fold_same("while(true)for(x in a())break");

        fold("while(true){try{continue;}catch(e){continue;}}", "while(true){try{}catch(e){}}");
        fold(
            "while(true){try{if(a()){continue;}else{continue;} continue;}catch(e){}}",
            "while(true){try{if(a()){}else{}}catch(e){}}",
        );

        fold("while(true){g:continue}", "while(true){}");
        // This case could be improved.
        fold(
            "while(true){g:if(a()){continue;}else{continue;} continue;}",
            "while(true){g:if(a());else;}",
        );
    }

    #[test]
    #[ignore]
    fn test_do_continue_optimization() {
        fold("do{if(x)continue; x=3; continue; }while(true)", "do if(x); else x=3; while(true)");
        fold_same("do{a();continue;b()}while(true)");
        fold(
            "do{if(true){a();continue;}else;b();}while(true)",
            "do{if(true){a();}else{b();}}while(true)",
        );
        fold(
            "do{if(false){a();continue;}else;b();continue;}while(true)",
            "do{if(false){a();}else{b();}}while(true)",
        );
        fold(
            "do{if(a()){b();continue;}else;c();}while(true)",
            "do{if(a()){b();}else{c()}}while(true)",
        );
        fold(
            "do{if(a()){b();}else{c();continue;}}while(true)",
            "do{if(a()){b();}else{c();}}while(true)",
        );
        fold("do{if(a()){b();continue;}else;}while(true)", "do{if(a()){b();}else;}while(true)");
        fold(
            "do{if(a()){continue;}else{continue;} continue;}while(true)",
            "do{if(a()){}else{}}while(true)",
        );
        fold(
            "do{if(a()){continue;}else{continue;} b();}while(true)",
            "do{if(a()){}else{continue; b();}}while(true)",
        );

        fold("do{while(a())continue;}while(true)", "do while(a());while(true)");
        fold("do{for(x in a())continue}while(true)", "do for(x in a());while(true)");

        fold("do{while(a())break;}while(true)", "do while(a())break;while(true)");
        fold_same("do for(x in a())break;while(true)");

        fold("do{try{continue;}catch(e){continue;}}while(true)", "do{try{}catch(e){}}while(true)");
        fold(
            "do{try{if(a()){continue;}else{continue;} continue;}catch(e){}}while(true)",
            "do{try{if(a()){}else{}}catch(e){}}while(true)",
        );

        fold("do{g:continue}while(true)", "do{}while(true)");
        // This case could be improved.
        fold(
            "do{g:if(a()){continue;}else{continue;} continue;}while(true)",
            "do{g:if(a());else;}while(true)",
        );

        fold("do { foo(); continue; } while(false)", "do { foo(); } while(false)");
        fold("do { foo(); break; } while(false)", "do { foo(); } while(false)");

        fold("do{break}while(!new Date());", "do{}while(!new Date());");

        fold_same("do { foo(); switch (x) { case 1: break; default: f()}; } while(false)");
    }

    #[test]
    #[ignore]
    fn test_for_continue_optimization() {
        fold("for(x in y){if(x)continue; x=3; continue; }", "for(x in y)if(x);else x=3");
        fold_same("for(x in y){a();continue;b()}");
        fold("for(x in y){if(true){a();continue;}else;b();}", "for(x in y){if(true)a();else b();}");
        fold(
            "for(x in y){if(false){a();continue;}else;b();continue;}",
            "for(x in y){if(false){a();}else{b()}}",
        );
        fold(
            "for(x in y){if(a()){b();continue;}else;c();}",
            "for(x in y){if(a()){b();}else{c();}}",
        );
        fold(
            "for(x in y){if(a()){b();}else{c();continue;}}",
            "for(x in y){if(a()){b();}else{c();}}",
        );

        fold("for(x of y){if(x)continue; x=3; continue; }", "for(x of y)if(x);else x=3");
        fold_same("for(x of y){a();continue;b()}");
        fold("for(x of y){if(true){a();continue;}else;b();}", "for(x of y){if(true)a();else b();}");
        fold(
            "for(x of y){if(false){a();continue;}else;b();continue;}",
            "for(x of y){if(false){a();}else{b()}}",
        );
        fold(
            "for(x of y){if(a()){b();continue;}else;c();}",
            "for(x of y){if(a()){b();}else{c();}}",
        );
        fold(
            "for(x of y){if(a()){b();}else{c();continue;}}",
            "for(x of y){if(a()){b();}else{c();}}",
        );

        fold(
            "async () => { for await (x of y){if(x)continue; x=3; continue; }}",
            "async () => { for await (x of y)if(x);else x=3 }",
        );
        fold_same("async () => { for await (x of y){a();continue;b()}}");
        fold(
            "async () => { for await (x of y){if(true){a();continue;}else;b();}}",
            "async () => { for await (x of y){if(true)a();else b();}}",
        );
        fold(
            "async () => { for await (x of y){if(false){a();continue;}else;b();continue;}}",
            "async () => { for await (x of y){if(false){a();}else{b()}}}",
        );
        fold(
            "async () => { for await (x of y){if(a()){b();continue;}else;c();}}",
            "async () => { for await (x of y){if(a()){b();}else{c();}}}",
        );
        fold(
            "async () => { for await (x of y){if(a()){b();}else{c();continue;}}}",
            "async () => { for await (x of y){if(a()){b();}else{c();}}}",
        );

        fold(
            "for(x=0;x<y;x++){if(a()){b();continue;}else;}",
            "for(x=0;x<y;x++){if(a()){b();}else;}",
        );
        fold(
            "for(x=0;x<y;x++){if(a()){continue;}else{continue;} continue;}",
            "for(x=0;x<y;x++){if(a()){}else{}}",
        );
        fold(
            "for(x=0;x<y;x++){if(a()){continue;}else{continue;} b();}",
            "for(x=0;x<y;x++){if(a()){}else{continue; b();}}",
        );

        fold("for(x=0;x<y;x++)while(a())continue;", "for(x=0;x<y;x++)while(a());");
        fold("for(x=0;x<y;x++)for(x in a())continue", "for(x=0;x<y;x++)for(x in a());");

        fold("for(x=0;x<y;x++)while(a())break;", "for(x=0;x<y;x++)while(a())break");
        fold_same("for(x=0;x<y;x++)for(x in a())break");

        fold(
            "for(x=0;x<y;x++){try{continue;}catch(e){continue;}}",
            "for(x=0;x<y;x++){try{}catch(e){}}",
        );
        fold(
            "for(x=0;x<y;x++){try{if(a()){continue;}else{continue;} continue;}catch(e){}}",
            "for(x=0;x<y;x++){try{if(a()){}else{}}catch(e){}}",
        );

        fold("for(x=0;x<y;x++){g:continue}", "for(x=0;x<y;x++){}");
        fold(
            "for(x=0;x<y;x++){g:if(a()){continue;}else{continue;} continue;}",
            "for(x=0;x<y;x++){g:if(a());else;}",
        );
    }

    #[test]
    #[ignore]
    fn test_code_motion_doesnt_break_function_hoisting() {
        fold(
            "function f() { if (x) return; foo(); function foo() {} }",
            "function f() { if (x); else { function foo() {} foo(); } }",
        );
    }

    #[test]
    #[ignore]
    fn test_dont_remove_break_in_try_finally() {
        fold_same("function f() {b:try{throw 9} finally {break b} return 1;}");
    }

    /**
     * The 'break' prevents the 'b=false' from being evaluated. If we fold the do-while to
     * 'do;while(b=false)' the code will be incorrect.
     *
     * @see https://github.com/google/closure-compiler/issues/554
     */
    #[test]
    #[ignore]
    fn test_dont_fold_break_in_do_while_if_condition_has_side_effects() {
        fold_same("var b=true;do{break}while(b=false);");
    }

    #[test]
    #[ignore]
    fn test_switch_exit_points1() {
        fold("switch (x) { case 1: f(); break; }", "switch (x) { case 1: f();        }");
        fold(
            "switch (x) { case 1: f(); break; case 2: g(); break; }",
            "switch (x) { case 1: f(); break; case 2: g();        }",
        );
        fold(
            "switch (x) { case 1: if (x) { f(); break; } break; default: g(); break; }",
            "switch (x) { case 1: if (x) { f();        } break; default: g();        }",
        );
    }

    #[test]
    #[ignore]
    fn test_fold_block_scoped_variables() {
        // When moving block-scoped variable declarations into inner blocks, first convert them to
        // "var" declarations to avoid breaking any references in inner functions.

        // For example, in the following test case, moving "let c = 3;" directly inside the else block
        // would break the function "g"'s reference to "c".
        fold(
            "function f() { function g() { return c; } if (x) {return;} let c = 3; }",
            "function f() { function g() { return c; } if (x){} else {var c = 3;} }",
        );
        fold(
            "function f() { function g() { return c; } if (x) {return;} const c = 3; }",
            "function f() { function g() { return c; } if (x) {} else {var c = 3;} }",
        );
        // Convert let and const even they're if not referenced by any functions.
        fold(
            "function f() { if (x) {return;} const c = 3; }",
            "function f() { if (x) {} else { var c = 3; } }",
        );
        fold(
            "function f() { if (x) {return;} let a = 3; let b = () => a; }",
            "function f() { if (x) {} else { var a = 3; var b = () => a;} }",
        );
        fold(
            "function f() { if (x) { if (y) {return;} let c = 3; } }",
            "function f() { if (x) { if (y) {} else { var c = 3; } } }",
        );
    }

    #[test]
    #[ignore]
    fn test_dont_fold_block_scoped_variables_in_loops() {
        // Don't move block-scoped declarations into inner blocks inside a loop, since converting
        // let/const declarations to vars in a loop can cause incorrect semantics.
        // See the following test case for an example.
        fold_same(
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
        fold_same("function f() { while (true) { if (true) {return;} let c = 3; } }");
        fold_same("function f() { do { if (true) {return;} let c = 3; } while (x); }");
        fold_same("function f() { for (;;) { if (true) { return; } let c = 3; } }");
        fold_same("function f(y) { for(x in []){ if(x) { return; } let c = 3; } }");
        fold_same("async function f(y) { for await (x in []){ if(x) { return; } let c = 3; } }");
    }
}
