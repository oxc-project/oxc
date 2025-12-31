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
        let default_pos = cases.iter().rposition(SwitchCase::is_default_case)?;

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

        // Reject only when a non-empty default lies inside the removable suffix, and it is not the last case.
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

        if let Some(last) = switch_stmt.cases.last_mut()
            && !Self::is_empty_switch_case(&last.consequent)
        {
            last.test = None;
            end -= 1;
            switch_stmt.cases.drain(start..end);
        } else {
            switch_stmt.cases.truncate(start);
        }
        ctx.state.changed = true;
    }

    fn remove_empty_switch(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };
        if switch_stmt.cases.is_empty() {
            if switch_stmt.discriminant.may_have_side_effects(ctx) {
                *stmt = ctx.ast.statement_expression(
                    switch_stmt.span,
                    switch_stmt.discriminant.take_in(ctx.ast),
                );
            } else {
                *stmt = ctx.ast.statement_empty(switch_stmt.span);
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
            Self::remove_last_break(&mut first.consequent);
            Self::remove_last_break(&mut second.consequent);

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
        if vec.len() == 1 && matches!(vec.first(), Some(Statement::BlockStatement(_))) {
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
            let Some(first_case) = switch_stmt.cases.first() else { return };
            if !Self::can_case_be_inlined(first_case) {
                return;
            }
            let Some(mut case) = switch_stmt.cases.pop() else {
                return;
            };

            ctx.state.changed = true;
            let discriminant = switch_stmt.discriminant.take_in(ctx.ast);
            Self::remove_last_break(&mut case.consequent);

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
                if !case.consequent.is_empty() {
                    stmts.extend(case.consequent);
                }
                *stmt = ctx.ast.statement_block_with_scope_id(
                    switch_stmt.span,
                    stmts,
                    ctx.create_child_scope_of_current(ScopeFlags::empty()),
                );
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

    fn remove_last_break(stmt: &mut Vec<'a, Statement<'a>>) {
        if stmt.is_empty() {
            return;
        }

        let len = stmt.len();
        match stmt.last_mut() {
            Some(Statement::BreakStatement(break_stmt)) => {
                if break_stmt.label.is_none() {
                    stmt.truncate(len - 1);
                }
            }
            Some(Statement::BlockStatement(block_stmt)) => {
                Self::remove_last_break(&mut block_stmt.body);
            }
            _ => {}
        }
    }

    fn is_terminated(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::BlockStatement(block_stmt) => {
                block_stmt.body.last().is_some_and(Self::is_terminated)
            }
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
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

impl BreakFinder {
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
            // TODO: this is not fully correct, we should allow termination if this if is a last statement
            Statement::IfStatement(it) => {
                if self.top_level {
                    self.top_level = false;
                    self.visit_if_statement(it);
                    self.top_level = true;
                } else {
                    self.visit_if_statement(it);
                }
            }
            Statement::TryStatement(it) => {
                if self.top_level {
                    self.top_level = false;
                    self.visit_try_statement(it);
                    self.top_level = true;
                } else {
                    self.visit_try_statement(it);
                }
            }
            Statement::WithStatement(it) => {
                if self.top_level {
                    self.top_level = false;
                    self.visit_with_statement(it);
                    self.top_level = true;
                } else {
                    self.visit_with_statement(it);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tester::{test, test_same};

    #[expect(clippy::literal_string_with_formatting_args)]
    #[test]
    fn minimize_switch() {
        test("switch(a()){}", "a()");
        test("switch(a){default: }", "a;");
        test("switch(a){default: break;}", "a;");
        test("switch(a){default: var b; break;}", "a;var b");
        test("switch(a){default: b()}", "a, b();");
        test("switch(a){default: b(); return;}", "a, b(); return");

        test("switch(a){case 1: break;}", "a;");
        test("switch(a){case 1: b();}", "a === 1 && b()");
        test("switch(a){case 1: b();break; }", "a === 1 && b()");
        test("switch(a){case 1: b();return; }", "if (a === 1) { b(); return; }");

        test("switch(a){case 1: default: }", "a;");
        test_same("switch(a){case 1: default: break; case 2: b()}");
        test_same("switch(a){case 1: b(); default: break; case 2: c()}");
        test_same("switch(a){case 1: b(); case 2: break; case 3: c()}");
        test_same("switch(a){case 1: b(); break; case 2: c();break;}");
        test_same("switch(a){case 1: b(); case 2: b();}");
        test("switch(a){case 1: var c=2; break;}", "if (a === 1) var c = 2;");

        test("switch(a){default: case 1: }", "if (a === 1) {} else {}");
        test("switch(a){default: break; case 1: break;}", "if (a === 1) {} else {}");
        test("switch(a){default: b();break;case 1: c();break;}", "if (a === 1) {c();} else {b();}");
        test("switch(a){default: {b();break;} case 1: {c();break;}}", "a === 1 ? c() : b()");

        test("switch(a){case b(): default:}", "if (a === b()) {} else {}");
        test_same("switch(a){case 2: case 1: break; default: break }");
        test_same("switch(a){case 3: b(); case 2: case 1: break; default: break }");

        test("var x=1;switch(x){case 1: var y;}", "var y;");
        test("function f(){switch(a){case 1: return;}}", "function f() {a;}");
        test("x:switch(a){case 1: break x;}", "x: if (a === 1) break x;");
        test("switch(a()) { default: {let y;} }", "a();{let y;}");
        test_same("function f(){switch('x'){case 'x': var x = 1;break; case 'y': break; }}");
        test("switch(a){default: if(a) {break;}bar();}", "switch(a){default: if(a) break;bar();}");
        test("switch ('\\v') {case '\\u000B': foo();}", "foo()");

        test_same("switch('r'){case 'r': a();break; case 'r': var x=0;break;}");
        test_same("switch('r'){case 'r': a();break; case 'r': bar();break;}");
        test("switch(2) {default: a; case 1: b()}", "if(2===1) {b();} else {a;}");
        test("switch(1) {case 1: a();break; default: b();}", "if(1===1) {a();} else {b();}");
        test_same("switch('e') {case 'e': case 'f': a();}");
        test_same("switch('a') {case 'a': a();break; case 'b': b();break;}");
        test_same("switch('c') {case 'a': a();break; case 'b': b();break;}");
        test_same("switch(1) {case 1: a();break; case 2: bar();break;}");
        test_same("switch('f') {case 'f': a(); case 'b': b();}");
        test_same("switch('f') {case 'f': if (a() > 0) {b();break;} c(); case 'd': f();}");
        test_same("switch('f') {case 'b': bar();break; case x: x();break; case 'f': f();break;}");
        test(
            "switch(1){case 1: case 2: {break;} case 3: case 4: default: b(); break;}",
            "switch(1){case 1: case 2: break; default: b(); break;}",
        );
        test(
            "switch ('d') {case 'foo': foo();break; default: bar();break;}",
            "if('d'==='foo') {foo();} else {bar();}",
        );
        test("outer: switch (2) {case 2: f(); break outer; }", "outer: {f(); break outer;}");
        test_same(
            "switch(0){case NaN: foobar();break;case -0.0: foo();break; case 2: bar();break;}",
        );
        test("let x = 1; switch('x') { case 'x': let x = 2; break;}", "let x = 1; { let x = 2 }");
        test("switch(1){case 2: var x=0;}", "if (0) var x;");
    }
}
