use super::PeepholeOptimizations;
use crate::keep_var::KeepVar;
use crate::{TraverseCtx, is_terminated::IsTerminated};
use oxc_allocator::{ArenaVec, ReplaceWith, TakeIn};
use oxc_ast::ast::{Expression, Statement, SwitchCase};
use oxc_ast_visit::{VisitJs, walk_js};
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};
use oxc_span::{GetSpan, SPAN};
use oxc_syntax::operator::BinaryOperator;

impl<'a> PeepholeOptimizations {
    /// Collapses a one-case switch into an if condition when the case can be safely inlined.
    pub fn try_minimize_switch(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else {
            return;
        };

        if switch_stmt.cases.len() == 1
            && Self::can_switch_case_be_inlined(&switch_stmt.cases[0])
            && let Some(mut case) = switch_stmt.cases.pop()
        {
            let block_stmt = if case.consequent.len() == 1
                && matches!(case.consequent.last(), Some(Statement::BlockStatement(_)))
            {
                case.consequent.pop().unwrap()
            } else {
                Statement::new_block_statement_with_scope_id(
                    case.span,
                    case.consequent,
                    switch_stmt.scope_id(),
                    ctx,
                )
            };

            ctx.notice_change();
            stmt.replace_with(|stmt| {
                let Statement::SwitchStatement(switch_stmt) = stmt else {
                    unreachable!();
                };
                let switch_stmt = switch_stmt.unbox();
                let expression = if let Some(test) = case.test {
                    Expression::new_binary_expression(
                        SPAN,
                        switch_stmt.discriminant,
                        BinaryOperator::StrictEquality,
                        test,
                        ctx,
                    )
                } else {
                    let span = switch_stmt.discriminant.span();
                    Expression::new_sequence_expression(
                        SPAN,
                        [
                            switch_stmt.discriminant,
                            Expression::new_boolean_literal(span, true, ctx),
                        ],
                        ctx,
                    )
                };

                Statement::new_if_statement(switch_stmt.span, expression, block_stmt, None, ctx)
            });
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

    /// Drops cases that cannot run for the chosen discriminant while preserving any
    /// observable case tests and hoisted `var` bindings.
    pub fn drop_unreachable_switch_cases(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else { return };
        if switch_stmt.cases.len() <= 1 {
            return;
        }

        if switch_stmt
            .cases
            .iter()
            .any(|case| case.consequent.iter().any(Self::statement_cares_about_scope))
        {
            // preserve any potential TDZ issues
            return;
        }

        let Some(discriminant) = switch_stmt.discriminant.evaluate_value(ctx) else {
            return;
        };

        let mut default_index = None;
        let mut matched_index = None;
        for (index, case) in switch_stmt.cases.iter().enumerate() {
            let Some(test) = &case.test else {
                default_index = Some(index);
                continue;
            };
            let Some(test_value) = test.evaluate_value(ctx) else {
                return;
            };

            // ECMAScript comparison semantics for switch matching (NaN/-0 aware).
            if discriminant == test_value {
                matched_index = Some(index);
                break;
            }
        }

        let mut kept_tests = ArenaVec::new_in(ctx);
        let mut keep_var = KeepVar::new();
        let mut selected_case = None;

        let Some(entry_index) = matched_index.or(default_index) else {
            // no body runs; retain observable tests.
            for (index, mut case) in switch_stmt.cases.drain(..).enumerate() {
                if let Some(test) = case.test.take()
                    && (index == 0 || test.may_have_side_effects(ctx))
                {
                    kept_tests.push(test);
                }
                keep_var.visit_switch_case(&case);
                ctx.drop_switch_case(&case);
                case.consequent.truncate(0);
                if index == 0 {
                    // retained test case.
                    selected_case = Some(case);
                }
            }

            let mut selected_case = selected_case.unwrap();
            selected_case.test = Some(Self::make_sequence_expression(kept_tests, ctx));
            if let Some(var_decl) = keep_var.get_variable_declaration_statement(ctx) {
                selected_case.consequent.push(var_decl);
            }
            switch_stmt.cases.push(selected_case);
            return;
        };

        let is_default_entry = matched_index.is_none();
        // fallthrough ends at termination.
        let fallthrough_end = switch_stmt.cases[entry_index..]
            .iter()
            .position(|case| case.consequent.is_terminated())
            .map_or(switch_stmt.cases.len(), |index| entry_index + index + 1);

        for (index, mut case) in switch_stmt.cases.take_in(ctx).into_iter().enumerate() {
            // tests evaluated before entering the body.
            let keep_test = index == entry_index
                || (is_default_entry || index < entry_index)
                    && case.test.may_have_side_effects(ctx);

            if keep_test && let Some(test) = case.test.take() {
                kept_tests.push(test);
            }

            if index == entry_index {
                // retained case
                selected_case = Some(case);
            } else if index >= entry_index && index < fallthrough_end {
                // same live fallthrough path
                selected_case.as_mut().unwrap().consequent.extend(case.consequent.drain(..));
                ctx.drop_switch_case(&case);
            } else {
                // keep only hoisted `var`s.
                keep_var.visit_switch_case(&case);
                ctx.drop_switch_case(&case);
            }
        }

        let mut selected_case = selected_case.unwrap();

        if !kept_tests.is_empty() {
            let test_expr = Self::make_sequence_expression(kept_tests, ctx);
            if is_default_entry {
                // old tests run before the default body.
                selected_case
                    .consequent
                    .insert(0, Statement::new_expression_statement(SPAN, test_expr, ctx));
            } else {
                selected_case.test = Some(test_expr);
            }
        }
        if let Some(var_decl) = keep_var.get_variable_declaration_statement(ctx) {
            selected_case.consequent.push(var_decl);
        }
        switch_stmt.cases = ArenaVec::from_value_in(selected_case, ctx);
    }

    fn make_sequence_expression(
        mut expressions: ArenaVec<'a, Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        if expressions.len() == 1 {
            expressions.pop().unwrap()
        } else {
            Expression::new_sequence_expression(SPAN, expressions, ctx)
        }
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
