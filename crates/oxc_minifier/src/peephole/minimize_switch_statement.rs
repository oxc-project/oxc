use crate::TraverseCtx;
use crate::peephole::PeepholeOptimizations;
use oxc_allocator::TakeIn;
use oxc_ast::ast::{Expression, Statement, SwitchCase};

use oxc_ast_visit::{VisitJs, walk_js};
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::operator::BinaryOperator;

impl<'a> PeepholeOptimizations {
    /// `MangleIf`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser.go#L9860>
    pub fn try_minimize_switch(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };

        if switch_stmt.cases.len() == 1
            && Self::can_switch_case_be_inlined(&switch_stmt.cases[0])
            && let Some(mut case) = switch_stmt.cases.pop()
        {
            ctx.notice_change();

            let block_stmt = if case.consequent.len() == 1
                && matches!(case.consequent[0], Statement::BlockStatement(_))
            {
                case.consequent.pop().unwrap()
            } else {
                Statement::new_block_statement_with_scope_id(
                    case.span,
                    case.consequent.take_in(ctx),
                    switch_stmt.scope_id(),
                    ctx,
                )
            };

            let expression = if let Some(test) = case.test {
                Expression::new_binary_expression(
                    SPAN,
                    switch_stmt.discriminant.take_in(ctx),
                    BinaryOperator::StrictEquality,
                    test,
                    ctx,
                )
            } else {
                Expression::new_sequence_expression(
                    SPAN,
                    [
                        switch_stmt.discriminant.take_in(ctx),
                        Expression::new_boolean_literal(switch_stmt.discriminant.span(), true, ctx),
                    ],
                    ctx,
                )
            };

            let new_if =
                Statement::new_if_statement(switch_stmt.span, expression, block_stmt, None, ctx);

            ctx.replace_statement(stmt, new_if);
        }
    }

    /// Check if a switch case can be inlined by verifying:
    /// - The test expression has no side effects
    /// - All statements can be safely inlined (no unlabeled breaks)
    fn can_switch_case_be_inlined(case: &SwitchCase<'a>) -> bool {
        if !case.test.as_ref().is_none_or(Expression::is_literal) {
            return false;
        }

        case.consequent.is_empty() || !FindNestedBreak::has_unlabelled_break_in_switch_case(case)
    }
}

#[derive(Default)]
struct FindNestedBreak {
    found_unlabelled_break: bool,
}

impl FindNestedBreak {
    fn has_unlabelled_break_in_switch_case(node: &SwitchCase) -> bool {
        let mut visitor = Self::default();
        visitor.visit_switch_case(node);
        visitor.found_unlabelled_break
    }
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
