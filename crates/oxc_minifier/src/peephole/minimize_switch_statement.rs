use super::PeepholeOptimizations;
use crate::ctx::Ctx;
use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::{Statement, SwitchCase};
use oxc_ast_visit::Visit;
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::{operator::BinaryOperator, scope::ScopeFlags};

impl<'a> PeepholeOptimizations {
    pub fn try_minimize_switch(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        Self::collapse_empty_switch_cases(stmt, ctx);
        Self::remove_empty_switch(stmt, ctx);
        Self::fold_switch_with_one_case(stmt, ctx);
        Self::fold_switch_with_two_cases(stmt, ctx);
    }

    fn find_trailing_removable_switch_cases_range(
        cases: &Vec<'_, SwitchCase<'a>>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<(usize, usize)> {
        let case_count = cases.len();
        if case_count <= 1 {
            return None;
        }

        // Position of the (last) default if any
        let Some(default_pos) = cases.iter().rposition(|c| c.is_default_case()) else {
            return None;
        };

        // Find the last non-removable case (any case whose consequent is non-empty).
        let last_non_empty_before_last = cases[..case_count - 1].iter().rposition(|c| {
            !c.consequent.is_empty()
                || c.test.as_ref().is_none_or(|test| test.may_have_side_effects(ctx))
        });

        // start is the first index of the removable suffix
        let start = match last_non_empty_before_last {
            Some(pos) => pos + 1,
            None => 0,
        };

        // nothing removable
        if start >= case_count - 1 {
            return None;
        }

        // Reject only when a non-empty default lies inside the removable suffix and it is not the last case.
        if default_pos >= start
            && default_pos != case_count - 1
            && !&cases[default_pos].consequent.is_empty()
        {
            return None;
        }
        Some((start, case_count))
    }

    fn collapse_empty_switch_cases(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };

        let Some((start, mut end)) =
            Self::find_trailing_removable_switch_cases_range(&switch_stmt.cases, ctx)
        else {
            return;
        };

        if let Some(last) = switch_stmt.cases.last_mut() {
            if !Self::is_empty_switch_case(&last.consequent) {
                last.test = None;
                end -= 1;
            }
        }
        switch_stmt.cases.drain(start..end);
        ctx.state.changed = true;
    }

    fn remove_empty_switch(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };
        if switch_stmt.cases.is_empty() {
            if !switch_stmt.discriminant.may_have_side_effects(ctx) {
                *stmt = ctx.ast.statement_empty(switch_stmt.span);
            } else {
                *stmt = ctx.ast.statement_expression(
                    switch_stmt.span,
                    switch_stmt.discriminant.take_in(ctx.ast),
                );
            }
            ctx.state.changed = true;
        }
    }

    fn fold_switch_with_two_cases(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };
        if switch_stmt.cases.len() == 2 {
            // check whatever its default + case
            if switch_stmt.cases[0].test.is_some() == switch_stmt.cases[1].test.is_some()
                || !Self::can_case_be_inlined(&switch_stmt.cases[0])
                || !Self::can_case_be_inlined(&switch_stmt.cases[1])
            {
                return;
            }

            let mut first = switch_stmt.cases.pop().unwrap();
            let mut second = switch_stmt.cases.pop().unwrap();
            Self::drop_break_and_postfix(&mut first.consequent);
            Self::drop_break_and_postfix(&mut second.consequent);

            let (test, consequent, alternate) = if first.test.is_some() {
                (first.test.unwrap(), first.consequent, second.consequent)
            } else {
                (second.test.unwrap(), second.consequent, first.consequent)
            };

            *stmt = ctx.ast.statement_if(
                switch_stmt.span,
                ctx.ast.expression_binary(
                    SPAN,
                    switch_stmt.discriminant.take_in(ctx.ast),
                    BinaryOperator::StrictEquality,
                    test,
                ),
                Self::create_if_block_from_switch_case(consequent, ctx),
                Some(Self::create_if_block_from_switch_case(alternate, ctx)),
            );
        }
    }

    fn create_if_block_from_switch_case(
        mut vec: Vec<'a, Statement<'a>>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Statement<'a> {
        if vec.len() == 1 && matches!(vec.get(0), Some(Statement::BlockStatement(_))) {
            vec.pop().unwrap()
        } else {
            ctx.ast.statement_block_with_scope_id(
                SPAN,
                vec,
                ctx.create_child_scope_of_current(ScopeFlags::empty()),
            )
        }
    }

    fn fold_switch_with_one_case(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };
        if switch_stmt.cases.len() == 1 {
            let Some(first_case) = switch_stmt.cases.get(0) else { return };
            if !Self::can_case_be_inlined(&first_case) {
                return;
            }
            let Some(mut case) = switch_stmt.cases.pop() else {
                return;
            };

            ctx.state.changed = true;
            let discriminant = switch_stmt.discriminant.take_in(ctx.ast);
            Self::drop_break_and_postfix(&mut case.consequent);

            if let Some(test) = case.test {
                *stmt = ctx.ast.statement_if(
                    switch_stmt.span,
                    ctx.ast.expression_binary(
                        SPAN,
                        discriminant,
                        BinaryOperator::StrictEquality,
                        test,
                    ),
                    Self::create_if_block_from_switch_case(case.consequent, ctx),
                    None,
                );
            } else {
                let mut stmts = ctx.ast.vec();
                if discriminant.may_have_side_effects(ctx) {
                    stmts.push(ctx.ast.statement_expression(discriminant.span(), discriminant));
                }
                if case.consequent.len() > 0 {
                    stmts.extend(case.consequent);
                }
                *stmt = ctx.ast.statement_block_with_scope_id(
                    switch_stmt.span,
                    stmts,
                    ctx.create_child_scope_of_current(ScopeFlags::empty()),
                )
            }
        }
    }

    pub fn drop_break_and_postfix(cons: &mut Vec<'a, Statement<'a>>) {
        // TODO: this is wrong, as it doesn't take into account BlockStatement
        if cons.len() > 0
            && let Some(terminates_rpos) = cons.iter().rposition(Self::is_terminated)
        {
            if cons
                .get(terminates_rpos)
                .is_some_and(|stmt| matches!(stmt, Statement::BreakStatement(_)))
            {
                cons.truncate(terminates_rpos);
            } else {
                cons.truncate(terminates_rpos + 1);
            }
        }
    }

    fn is_empty_switch_case(stmt: &Vec<Statement>) -> bool {
        if stmt.len() != 1 {
            return stmt.is_empty();
        }
        match stmt.last() {
            Some(Statement::BlockStatement(block_stmt)) => block_stmt.body.is_empty(),
            _ => false,
        }
    }

    pub fn is_terminated(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::BreakStatement(break_stmt) => break_stmt.label.is_none(),
            Statement::BlockStatement(block_stmt) => {
                block_stmt.body.last().is_some_and(Self::is_terminated)
            }
            Statement::ContinueStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_) => true,
            _ => false,
        }
    }

    pub fn can_case_be_inlined(case: &SwitchCase) -> bool {
        let mut break_finder = BreakFinder::new();
        break_finder.visit_switch_case(case);
        !break_finder.nested_unlabelled_break
    }
}

struct BreakFinder {
    top_level: bool,
    nested_unlabelled_break: bool,
}

impl<'a> BreakFinder {
    pub fn new() -> Self {
        Self { top_level: true, nested_unlabelled_break: false }
    }
}

impl<'a> Visit<'a> for BreakFinder {
    fn visit_statement(&mut self, it: &Statement<'a>) {
        // Only visit blocks where vars could be hoisted
        match it {
            Statement::BlockStatement(it) => self.visit_block_statement(it),
            Statement::BreakStatement(it) => {
                if !self.top_level && it.label.is_none() {
                    self.nested_unlabelled_break = true;
                }
            }
            Statement::IfStatement(it) => {
                if self.top_level {
                    self.top_level = false;
                    self.visit_if_statement(it);
                    self.top_level = true;
                } else {
                    self.visit_if_statement(it);
                }
            }
            Statement::TryStatement(it) => self.visit_try_statement(it),
            Statement::WithStatement(it) => self.visit_with_statement(it),
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[test]
    fn minimize_switch() {
        // remove empty
        test("switch (empty) {}", "empty");
        test("switch (!true) {}", "");
        test("switch (true) {}", "");
        test("switch (fn()) {}", "fn()");
        // truncate cases
        test("switch (test) { default: break; case 1: break; }", "if (test === 1) {} else {}");
        test("switch (test) { case 1: break; }", "test;");
        test_same("switch (test) { case 2: case 1: break; default: break }");
        test_same("switch (test) { case 3: foo(); case 2: case 1: break; default: break }");
        // single switch case
        test("switch (fn()) { default: a(); break; }", "fn(),a()");
        test("switch (test) { default: fn(); return; }", "test, fn(); return");
        test("switch (fn()) { default: a(); }", "fn(), a()");
        test("switch (test) { case 1: fn(); break; }", "test === 1 && fn()");
        test("switch (test) { case 1: fn(); return; }", "if (test === 1) { fn(); return; }");
        test("switch (test) { case 1: fn(); }", "test === 1 && fn()");
        test("switch (a) { case log(1): default:}", "if (a === log(1)) {} else {}");
        // default + case
        test(
            "switch (c) { default: a(); break; case 1: b(); break; }",
            "if (c === 1) { b(); } else { a(); }",
        );
        test(
            "switch (c) { default: { a(); break; } case 1: { b(); break; } }",
            "c === 1 ? b() : a()",
        );
        // evaluate static case
        test(
            "switch (1) { case 1: a(); break; default: b(); }",
            "if (1 === 1) { a(); } else { b(); }",
        );
        test(
            "switch (1) { case 2: foo(); break; default: bar(); }",
            "if (1 === 2) { foo(); } else { bar(); }",
        );
        // special cases
        test_same("switch (test) { case 1: fn(); break; case 2: bar(); break; }");
        test_same("switch (x) { case 1: a(); case 2: b(); }");
        test(
            "switch (2) { default: log(3); case 1: log(2) }",
            "if (2 === 1) {log(2);} else {log(3);}",
        );

        test("switch(a){}", "a;");
        test("switch(foo()){}", "foo()");
        test("switch(a){default:}", "a;");
        test("switch(a){default:break;}", "a;");
        test("switch(a){default:var b;break;}", "a;var b");
        test("switch(a){case 1: default:}", "a;");
        test("switch(a){default: case 1:}", "if (a === 1) {} else {}");
        test("switch(a){default: break; case 1:break;}", "if (a === 1) {} else {}");
        test(
            "switch(a){default: var b; break; case 1: var c; break;}",
            "if (a === 1) { var c; } else { var b; }",
        );
        test("var x=1; switch(x) { case 1: var y; }", "var y;");
        test(
            "function f() {switch(a){default: return; case 1: break;}}",
            "function f() { if (a === 1) {} else { return; } }",
        );
        test(
            "function f() {switch(1){default: return; case 1: break;}}",
            "function f() {if (1 === 1) {} else {return;}}",
        );
        test("function f() {switch(a){case 1: foo();}}", "function f() {a === 1 && foo();}");
        test_same("function f() {switch(a){case 3: case 2: case 1: foo();}}");
        test(
            "function f() {switch(a){case 2: case 1: default: foo();}}",
            "function f() {a, foo();}",
        );
        // test cases from closure-compiler
        // <https://github.com/google/closure-compiler/blob/736315149f0fdacf0a4c75be5fe51fdc8f01232d/test/com/google/javascript/jscomp/PeepholeRemoveDeadCodeTest.java#L545>
        test_same("switch(a){case 1: default:break; case 2: foo()}");
        test_same("switch(a){case 1: goo(); default:break; case 2: foo()}");
        test_same("switch(a){case 1: goo(); case 2:break; case 3: foo()}");
        test("switch(1){case 2: var x=0;}", "if (0) var x;");
        test_same(
            "switch ('repeated') { case 'repeated': foo(); break; case 'repeated': var x=0; break;}",
        );
        test_same(
            "switch ('repeated') { case 'repeated': foo(); break; case 'repeated': bar(); break;}",
        );
        test("switch(a){case 1: var c =2; break;}", "if (a === 1) var c = 2;");
        test("function f() {switch(a){case 1: return;}}", "function f() {a;}");
        test("x:switch(a){case 1: break x;}", "x: if (a === 1) break x;");
        test("let x;switch (use(x)) { default: {let y;} }", "let x;use(void 0);{let y;}");
        test("let x;switch (use?.(x)) {default: {let y;}}", "let x;use?.(void 0);{let y;}");
        test("let x;switch (use(x)) {default: let y;}", "let x;{use(void 0);let y;}");
        test_same(" function f() { switch('x') { case 'x': var x = 1; break; case 'y': break; } }");
        test(
            "function f() { switch(x) { case 'y': break; default: var x = 1; } }",
            "function f() { if (x === 'y') {} else { var x = 1; } }",
        );
        test("let x = 1; switch('x') { case 'x': let x = 2; break;}", "let x = 1; { let x = 2 }");
        test(
            "switch (x) { default: if (a) { break; } bar(); }",
            "switch (x) { default: if (a) break; bar(); }",
        );
    }

    #[test]
    fn minimize_switch_with_literal() {
        test("switch ('\\v') {case '\\u000B': foo();}", "foo()");
        test_same("switch ('empty') {case 'empty':case 'foo': foo();}");
        test(
            "switch (1) { case 1: case 2: case 3: { break; } case 4: case 5: case 6: default: fail('Should not get here'); break; }",
            "switch (1) { case 1: case 2: case 3: break; default: fail('Should not get here'); break; }",
        );
        test_same("switch (1) {case 1: foo();break;case 2: bar();break;}");
        test_same("switch (1) {case 1.1: foo();break;case 2: bar();break;}");
        test("outer: switch (2) {case 2: f(); break outer; }", "outer: {f(); break outer;}");
        test("outer: {switch (2) {case 2:f();break outer;}}", "outer: {f(); break outer;}");
        test_same("switch ('foo') {case 'foo': foo();break;case 'bar': bar();break;}");
        test_same("switch ('noMatch') {case 'foo': foo();break;case 'bar': bar();break;}");
        test_same(
            "switch ('fallThru') {case 'fallThru': if (foo(123) > 0) {foobar(1);break;}  foobar(2);case 'bar': bar();}",
        );
        test_same("switch ('fallThru') {case 'fallThru': foo();case 'bar': bar();}");
        test(
            "switch ('hasDefaultCase') { case 'foo': foo(); break; default: bar(); break; }",
            "if ('hasDefaultCase' === 'foo') { foo(); } else { bar(); }",
        );
        test_same(
            "switch ('foo') {case 'bar': bar();break;case notConstant: foobar();break;case 'foo': foo();break;}",
        );
        test_same(
            "switch (0) {case NaN: foobar();break;case -0.0: foo();break;case 2: bar();break;}",
        );
    }
}
