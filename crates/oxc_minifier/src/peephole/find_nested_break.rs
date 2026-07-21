use crate::peephole::PeepholeOptimizations;
use oxc_ast::ast::{Expression, Statement, SwitchCase};
use oxc_ast_visit::{VisitJs, walk_js};

impl<'a> PeepholeOptimizations {
    /// Check if a switch case can be inlined by verifying:
    /// - The test expression has no side effects
    /// - All statements can be safely inlined (no unlabeled breaks)
    pub fn can_switch_case_be_inlined(case: &SwitchCase<'a>) -> bool {
        if !case.test.as_ref().is_none_or(Expression::is_literal) {
            return false;
        }

        if case.consequent.is_empty() {
            return true;
        }

        let mut break_finder = FindNestedBreak { found_unlabelled_break: false };
        break_finder.visit_switch_case(case);
        !break_finder.found_unlabelled_break
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
            Statement::BreakStatement(it) if it.label.is_none() => {
                self.found_unlabelled_break = true;
            }
            _ => walk_js::walk_statement(self, it),
        }
    }
}
