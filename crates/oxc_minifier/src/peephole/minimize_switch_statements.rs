use super::PeepholeOptimizations;
use crate::keep_var::KeepVar;
use crate::{TraverseCtx, is_terminated::IsTerminated};
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::{Declaration, Expression, Statement};
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};
use oxc_span::SPAN;

impl<'a> PeepholeOptimizations {
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

        // In sloppy mode, function declarations nested in a dropped case create an
        // Annex B.3.3 function-scoped binding that exists (uninitialized) even when
        // the case never runs, so later assignments to the name write that local
        // binding. Dropping the case would turn them into global writes. Keep the
        // switch in that case. (Modules are strict and have no Annex B binding.)
        if !ctx.source_type().is_module() {
            let mut visitor = ContainsFunctionDeclaration::default();
            for case in &switch_stmt.cases {
                visitor.visit_switch_case(case);
            }
            if visitor.0 {
                return;
            }
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

/// Walks dropped statements looking for function declarations, which get an
/// Annex B.3.3 function-scoped binding in sloppy mode.
#[derive(Default)]
struct ContainsFunctionDeclaration(bool);

impl<'a> VisitJs<'a> for ContainsFunctionDeclaration {
    // `walk_statement` routes function declarations through `visit_declaration`.
    fn visit_declaration(&mut self, it: &Declaration<'a>) {
        if matches!(it, Declaration::FunctionDeclaration(_)) {
            self.0 = true;
        }
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if self.0 {
            return;
        }
        // Same selective traversal as `KeepVar`: only descend into statements that
        // can nest a function declaration.
        match it {
            Statement::FunctionDeclaration(_) => self.0 = true,
            Statement::BlockStatement(it) => self.visit_block_statement(it),
            Statement::BreakStatement(it) => self.visit_break_statement(it),
            Statement::ContinueStatement(it) => self.visit_continue_statement(it),
            Statement::DoWhileStatement(it) => self.visit_do_while_statement(it),
            Statement::ForInStatement(it) => self.visit_for_in_statement(it),
            Statement::ForOfStatement(it) => self.visit_for_of_statement(it),
            Statement::ForStatement(it) => self.visit_for_statement(it),
            Statement::IfStatement(it) => self.visit_if_statement(it),
            Statement::LabeledStatement(it) => self.visit_labeled_statement(it),
            Statement::SwitchStatement(it) => self.visit_switch_statement(it),
            Statement::TryStatement(it) => self.visit_try_statement(it),
            Statement::WhileStatement(it) => self.visit_while_statement(it),
            Statement::WithStatement(it) => self.visit_with_statement(it),
            _ => {}
        }
    }
}
