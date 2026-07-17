use crate::peephole::PeepholeOptimizations;
use oxc_ast::ast::{
    BreakStatement, Expression, IfStatement, Statement, SwitchCase, TryStatement, WithStatement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_ecmascript::side_effects::MayHaveSideEffects;

use crate::TraverseCtx;

impl<'a> PeepholeOptimizations {
    pub fn can_switch_case_be_inlined(case: &SwitchCase<'a>, ctx: &TraverseCtx<'a>) -> bool {
        if case.test.may_have_side_effects(ctx) {
            return false;
        }

        // unlabeled top-level `break` is only valid as the terminal statement of the case body.
        if case
            .consequent
            .iter()
            .position(|stmt| matches!(stmt, Statement::BreakStatement(break_stmt) if break_stmt.label.is_none()))
            .is_some_and(|pos| pos + 1 != case.consequent.len())
        {
            return false;
        }

        let mut break_finder = FindNestedBreak::new();
        break_finder.visit_switch_case(case);
        !break_finder.nested_unlabelled_break
    }
}

struct FindNestedBreak {
    top_level: bool,
    nested_unlabelled_break: bool,
}

impl FindNestedBreak {
    pub fn new() -> Self {
        Self { top_level: true, nested_unlabelled_break: false }
    }
}

impl<'a> Visit<'a> for FindNestedBreak {
    fn visit_expression(&mut self, _it: &Expression<'a>) {
        // do nothing
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if it.is_declaration() || it.is_iteration_statement() {
            return;
        }
        match it {
            Statement::ThrowStatement(_)
            | Statement::SwitchStatement(_)
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
