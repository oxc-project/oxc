use crate::peephole::PeepholeOptimizations;
use oxc_ast::ast::{BreakStatement, Expression, Statement, SwitchCase};
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_ecmascript::side_effects::MayHaveSideEffects;

use crate::TraverseCtx;

impl<'a> PeepholeOptimizations {
    /// Check if a switch case can be inlined by verifying:
    /// - The test expression has no side effects
    /// - All statements can be safely inlined (no unlabeled breaks outside terminal position)
    pub fn can_switch_case_be_inlined(case: &SwitchCase<'a>, ctx: &TraverseCtx<'a>) -> bool {
        if case.test.may_have_side_effects(ctx) {
            return false;
        }

        if case.consequent.is_empty() {
            return true;
        }

        let mut break_finder = FindNestedBreak { found_unlabelled_break: false };
        let last_idx = case.consequent.len() - 1;

        // Iterate backwards through consequent statements
        for (idx, stmt) in case.consequent.iter().enumerate().rev() {
            // Skip terminal statement if it's an unlabeled break
            if idx == last_idx
                && matches!(stmt, Statement::BreakStatement(break_stmt) if break_stmt.label.is_none())
            {
                continue;
            }

            break_finder.visit_statement(stmt);

            if break_finder.found_unlabelled_break {
                return false;
            }
        }

        true
    }
}

struct FindNestedBreak {
    found_unlabelled_break: bool,
}

impl<'a> VisitJs<'a> for FindNestedBreak {
    fn visit_expression(&mut self, _it: &Expression<'a>) {
        // do nothing
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if self.found_unlabelled_break || it.is_declaration() || it.is_iteration_statement() {
            return;
        }
        match it {
            Statement::ThrowStatement(_)
            | Statement::SwitchStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::ExpressionStatement(_) => {}
            _ => walk_js::walk_statement(self, it),
        }
    }

    fn visit_break_statement(&mut self, it: &BreakStatement<'a>) {
        if it.label.is_none() {
            self.found_unlabelled_break = true;
        }
    }
}
