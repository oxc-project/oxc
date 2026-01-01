use super::PeepholeOptimizations;
use crate::ctx::Ctx;
use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::{operator::BinaryOperator, scope::ScopeFlags};

impl<'a> PeepholeOptimizations {
    /// Attempts to minimize a `switch` statement by applying a series of transformations
    /// - Removes the trailing `break` statement from the last case of the `switch`, if it's unnecessary.
    /// - Merges or removes consecutive empty cases within the switch to simplify its structure.
    /// - Eliminates the entire `switch` statement if it contains no meaningful cases or logic.
    /// - Converts the `switch` if it contains only one or two cases to `if`/`else` statements.
    pub fn try_minimize_switch(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        Self::try_remove_last_break_from_case(stmt, ctx);
        Self::collapse_empty_switch_cases(stmt, ctx);
        Self::remove_empty_switch(stmt, ctx);
        Self::fold_switch_with_one_case(stmt, ctx);
        Self::fold_switch_with_two_cases(stmt, ctx);
    }

    /// Attempts to remove the last `break` statement from the last case of a switch statement.
    fn try_remove_last_break_from_case(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };

        if let Some(last_case) = switch_stmt.cases.last_mut()
            && Self::remove_last_break(&mut last_case.consequent, ctx)
        {
            ctx.state.changed = true;
        }
    }

    /// Collapses empty cases in a `SwitchStatement` by removing redundant cases with empty
    /// consequent's and consolidating them into a more concise representation.
    ///
    /// - If the switch statement contains one or fewer cases, it is considered already optimal, and no actions are taken.
    /// - If the `default` case is the last case, it is treated as a special case where its emptiness directly
    ///   influences the analysis of the rest of the cases.
    /// - The function identifies a `removable suffix` of cases at the end of the statement, starting from the first
    ///   non-empty case or case with side-effect-producing expressions backward to the last case.
    /// - All cases in the identified removable suffix are eliminated, except for the last case,
    ///   which is preserved and its test is removed (if applicable).
    fn collapse_empty_switch_cases(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };

        let case_count = switch_stmt.cases.len();
        if case_count <= 1 {
            return;
        }

        // if a default case is last we can skip checking if it has body
        let (end, allow_break) = if let Some(default_pos) =
            switch_stmt.cases.iter().rposition(SwitchCase::is_default_case)
        {
            if default_pos == case_count - 1 {
                (
                    case_count - 1,
                    Self::is_empty_switch_case(&switch_stmt.cases[default_pos].consequent, true),
                )
            } else {
                (case_count, false)
            }
        } else {
            (case_count, true)
        };

        // Find the last non-removable case (any case whose consequent is non-empty).
        let last_non_empty_before_last = switch_stmt.cases[..end].iter().rposition(|case| {
            !Self::is_empty_switch_case(&case.consequent, allow_break)
                || case.test.as_ref().is_some_and(|test| test.may_have_side_effects(ctx))
        });

        // start is the first index of the removable suffix
        let start = match last_non_empty_before_last {
            Some(pos) => pos + 1,
            None => 0,
        };

        // nothing removable
        if start >= case_count - 1 {
            return;
        }

        let Some(mut last) = switch_stmt.cases.pop() else {
            return;
        };
        switch_stmt.cases.truncate(start);

        if !Self::is_empty_switch_case(&last.consequent, true) {
            last.test = None;
            switch_stmt.cases.push(last);
        }
        ctx.state.changed = true;
    }

    /// Removes an empty switch statement from the given AST statement.
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

    /// Simplifies a `switch` statement with exactly two cases into an equivalent `if` statement.
    ///
    /// This transformation is applicable when the `switch` statement meets the following criteria:
    /// - It contains exactly two cases.
    /// - One of the cases represents the `default` case, and the other defines a condition (`test`).
    /// - Both cases can be safely inlined without reordering or modifying program behavior.
    /// - Both cases are terminated properly (e.g., with a `break` statement).
    fn fold_switch_with_two_cases(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };

        // check whatever its default + case
        if switch_stmt.cases.len() != 2
            || switch_stmt.cases[0].test.is_some() == switch_stmt.cases[1].test.is_some()
            || !Self::is_terminated_switch_case(&switch_stmt.cases[0].consequent)
            || !Self::can_case_be_inlined(&switch_stmt.cases[0], ctx)
            || !Self::can_case_be_inlined(&switch_stmt.cases[1], ctx)
        {
            return;
        }

        let mut first = switch_stmt.cases.pop().unwrap();
        let mut second = switch_stmt.cases.pop().unwrap();
        Self::remove_last_break(&mut first.consequent, ctx);
        Self::remove_last_break(&mut second.consequent, ctx);

        let (test, consequent, alternate) = if first.test.is_some() {
            (first.test.unwrap(), first.consequent, second.consequent)
        } else {
            (second.test.unwrap(), second.consequent, first.consequent)
        };

        ctx.state.changed = true;
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
            if !Self::can_case_be_inlined(first_case, ctx) {
                return;
            }
            let mut case = switch_stmt.cases.pop().unwrap();

            ctx.state.changed = true;
            let discriminant = switch_stmt.discriminant.take_in(ctx.ast);
            Self::remove_last_break(&mut case.consequent, ctx);

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
                if !Self::is_empty_switch_case(&case.consequent, true) {
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

    fn is_empty_switch_case(stmt: &Vec<'a, Statement<'a>>, allow_break: bool) -> bool {
        if stmt.len() != 1 {
            return stmt.is_empty();
        }
        match stmt.last() {
            Some(Statement::EmptyStatement(_)) => true,
            Some(Statement::BlockStatement(block_stmt)) => {
                Self::is_empty_switch_case(&block_stmt.body, allow_break)
            }
            Some(Statement::BreakStatement(break_stmt)) => {
                break_stmt.label.is_none() && allow_break
            }
            _ => false,
        }
    }

    fn remove_break_from_statement(stmt: &mut Statement<'a>, ctx: &Ctx<'a, '_>) -> bool {
        match stmt {
            Statement::BreakStatement(break_stmt) => {
                if break_stmt.label.is_none() {
                    *stmt = ctx.ast.statement_empty(break_stmt.span);
                    true
                } else {
                    false
                }
            }
            Statement::BlockStatement(block_stmt) => {
                Self::remove_last_break(&mut block_stmt.body, ctx)
            }
            Statement::IfStatement(if_stmt) => {
                let mut changed = Self::remove_break_from_statement(&mut if_stmt.consequent, ctx);
                if let Some(alternate) = &mut if_stmt.alternate {
                    changed |= Self::remove_break_from_statement(alternate, ctx);
                }
                changed
            }
            _ => false,
        }
    }

    fn remove_last_break(stmt: &mut Vec<'a, Statement<'a>>, ctx: &Ctx<'a, '_>) -> bool {
        if stmt.is_empty() {
            return false;
        }

        let len = stmt.len();
        match stmt.last_mut() {
            Some(Statement::BreakStatement(break_stmt)) => {
                if break_stmt.label.is_none() {
                    stmt.truncate(len - 1);
                    true
                } else {
                    false
                }
            }
            Some(stmt) => Self::remove_break_from_statement(stmt, ctx),
            _ => false,
        }
    }

    fn is_terminated_switch_case(stmt: &Vec<'a, Statement<'a>>) -> bool {
        if stmt.is_empty() {
            return false;
        }
        match stmt.last() {
            Some(Statement::BlockStatement(block_stmt)) => {
                Self::is_terminated_switch_case(&block_stmt.body)
            }
            Some(last) => last.is_jump_statement(),
            _ => false,
        }
    }

    pub fn can_case_be_inlined(case: &SwitchCase<'a>, ctx: &Ctx<'a, '_>) -> bool {
        if case.test.as_ref().is_some_and(|test| test.may_have_side_effects(ctx)) {
            return false;
        }

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

// TODO: This is to aggressive, we should allow `break` for last elements in statements
impl<'a> Visit<'a> for BreakFinder {
    fn visit_expression(&mut self, _it: &Expression<'a>) {
        // do nothing
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if it.is_declaration() || it.is_iteration_statement() {
            return;
        }
        match it {
            Statement::ThrowStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::ExpressionStatement(_) => {}
            _ => walk::walk_statement(self, it),
        }
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        let was_top = self.top_level;
        self.top_level = false;
        walk::walk_if_statement(self, it);
        self.top_level = was_top;
    }

    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        if !self.top_level && it.label.is_none() {
            self.nested_unlabelled_break = true;
        }
    }

    fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
        let was_top = self.top_level;
        self.top_level = false;
        walk::walk_with_statement(self, it);
        self.top_level = was_top;
    }

    fn visit_try_statement(&mut self, it: &TryStatement<'a>) {
        let was_top = self.top_level;
        self.top_level = false;
        walk::walk_try_statement(self, it);
        self.top_level = was_top;
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

        test("switch(a){default: case 1: }", "a");
        test("switch(a){case 1: default: }", "a");
        test_same("switch(a){case 1: default: break; case 2: b()}");
        test_same("switch(a){case 1: b(); default: c()}");
        test_same("switch(a){case 1: b(); default: break; case 2: c()}");
        test_same("switch(a){case 1: b(); case 2: break; case 3: c()}");
        test(
            "switch(a){case 1: b(); break; case 2: c();break;}",
            "switch(a){case 1: b(); break; case 2: c();}",
        );
        test_same("switch(a){case 1: b(); case 2: b();}");
        test("switch(a){case 1: var c=2; break;}", "if (a === 1) var c = 2");
        test("switch(a){case 1: case 2: default: b(); break;}", "a, b()");

        test("switch(a){default: break; case 1: break;}", "a");
        test("switch(a){default: b();break;case 1: c();break;}", "a === 1 ? c() : b()");
        test("switch(a){default: {b();break;} case 1: {c();break;}}", "a === 1 ? c() : b()");

        test_same("switch(a){case b(): default:}");
        test("switch(a){case 2: case 1: break; default: break;}", "a;");
        test("switch(a){case 3: b(); case 2: case 1: break;}", "a === 3 && b()");
        test("switch(a){case 3: b(); case 2: case 1: }", "a === 3 && b()");
        test("switch(a){case 3: if (b) break }", "a === 3 && b");
        test("switch(a){case 3: { if (b) break } }", "a === 3 && b");
        test("switch(a){case 3: { if(b) {c()} else {break;} }}", "a === 3 && b && c()");
        test(
            "switch(a){case 3: { if(b) {c(); break;} else { d(); break;} }}",
            "a === 3 && (b ? c() : d())",
        );
        test("switch(a){case 3: { for (;;) break } }", "if(a === 3) for (;;) break;");
        test("switch(a){case 3: { for (b of c) break } }", "if (a === 3) for (b of c) break;");
        test_same("switch(a){case 3: with(b) break}");
        test("switch(a){case 3: while(!0) break}", "if (a === 3) for (;;) break;");

        test(
            "switch(a){case 1: c(); case 2: default: b();break;}",
            "switch(a){case 1: c(); default: b();}",
        );
        test("var x=1;switch(x){case 1: var y;}", "var y;");
        test("function f(){switch(a){case 1: return;}}", "function f() {a;}");
        test("switch(a()) { default: {let y;} }", "a();{let y;}");
        test(
            "function f(){switch('x'){case 'x': var x = 1;break; case 'y': break; }}",
            "function f(){switch('x'){case 'x': var x = 1;break; case 'y': }}",
        );
        test("switch(a){default: if(a) {break;}c();}", "switch(a){default: if(a) break;c();}");
        test("switch(a){case 1: if(a) {b();}c();}", "a === 1 && (a && b(), c())");
        test("switch ('\\v') {case '\\u000B': foo();}", "foo()");

        test("x: switch(a){case 1: break x;}", "x: if (a === 1) break x;");
        test_same("x: switch(a){case 2: break x; case 1: break x;}");
        test("x: switch(2){case 2: f(); break outer; }", "x: {f(); break outer;}");
        test(
            "x: switch(x){case 2: f(); for (;;){break outer;}}",
            "x: if(x===2) for(f();;) break outer",
        );
        test("x: switch(a){case 2: if(b) { break outer; } }", "x: if(a===2 && b) break outer;");

        test(
            "switch('r'){case 'r': a();break; case 'r': var x=0;break;}",
            "switch('r'){case 'r': a();break; case 'r': var x=0;}",
        );
        test(
            "switch('r'){case 'r': a();break; case 'r': bar();break;}",
            "switch('r'){case 'r': a();break; case 'r': bar()}",
        );
        test_same("switch(2) {default: a; case 1: b()}");
        test("switch(1) {case 1: a();break; default: b();}", "a()");
        test_same("switch('e') {case 'e': case 'f': a();}");
        test(
            "switch('a') {case 'a': a();break; case 'b': b();break;}",
            "switch('a') {case 'a': a();break; case 'b': b();}",
        );
        test(
            "switch('c') {case 'a': a();break; case 'b': b();break;}",
            "switch('c') {case 'a': a();break; case 'b': b();}",
        );
        test(
            "switch(1) {case 1: a();break; case 2: bar();break;}",
            "switch(1) {case 1: a();break; case 2: bar();}",
        );
        test_same("switch('f') {case 'f': a(); case 'b': b();}");
        test_same("switch('f') {case 'f': if (a() > 0) {b();break;} c(); case 'd': f();}");
        test(
            "switch('f') {case 'b': bar();break; case x: x();break; case 'f': f();break;}",
            "switch('f') {case 'b': bar();break; case x: x();break; case 'f': f();}",
        );
        test(
            "switch(1){case 1: case 2: {break;} case 3: case 4: default: b(); break;}",
            "switch(1){case 1: case 2: break; default: b();}",
        );
        test("switch ('d') {case 'foo': foo();break; default: bar();break;}", "bar()");
        test(
            "switch(0){case NaN: foobar();break;case -0.0: foo();break; case 2: bar();break;}",
            "switch(0){case NaN: foobar();break;case -0.0: foo();break; case 2: bar();}",
        );
        test("let x = 1; switch('x') { case 'x': let x = 2; break;}", "let x = 1; { let x = 2 }");
        test("switch(1){case 2: var x=0;}", "if (0) var x;");
    }
}
