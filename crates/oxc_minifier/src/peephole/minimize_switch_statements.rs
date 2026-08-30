use super::PeepholeOptimizations;
use crate::keep_var::KeepVar;
use crate::{TraverseCtx, is_terminated::IsTerminated};
use oxc_allocator::ArenaVec;
use oxc_ast::ast::{Statement, SwitchCase};
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};

impl<'a> PeepholeOptimizations {
    /// Drops cases that cannot run for the chosen discriminant while preserving any
    /// observable case tests and hoisted `var` bindings.
    pub fn drop_unreachable_switch_cases(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else { return };
        if switch_stmt.cases.len() <= 1 {
            return;
        }
        let Some(discriminant) = switch_stmt.discriminant.evaluate_value(ctx) else {
            return;
        };

        let mut default_index = None;
        let mut start = None;
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
                start = Some(index);
                break;
            }
        }

        let is_default_entry = start.is_none();
        let Some(start) = start.or(default_index) else {
            // no case runs, but tests are still evaluated in order
            switch_stmt.cases.retain_mut(|case| {
                if case.test.may_have_side_effects(ctx) {
                    return true;
                }
                !Self::try_drop_content_from_case(case, ctx)
            });
            return;
        };

        let end = switch_stmt.cases[start..]
            .iter()
            .position(|case| case.consequent.is_terminated())
            .map_or(switch_stmt.cases.len(), |index| start + index + 1);

        let len = switch_stmt.cases.len();
        let old_cases =
            std::mem::replace(&mut switch_stmt.cases, ArenaVec::with_capacity_in(len, ctx));

        for (index, mut case) in old_cases.into_iter().enumerate() {
            // when `default` is selected, all of `test` are executed
            let test_is_observable =
                (is_default_entry || index < start) && case.test.may_have_side_effects(ctx);

            if index == start {
                // selected match/default entry
                switch_stmt.cases.push(case);
            } else if index > start && index < end && !test_is_observable {
                // same live fallthrough path
                switch_stmt.cases.last_mut().unwrap().consequent.extend(case.consequent.drain(..));
                ctx.drop_switch_case(&case);
            } else if test_is_observable {
                // this test is still evaluated before entering the selected body
                switch_stmt.cases.push(case);
            } else if !Self::try_drop_content_from_case(&mut case, ctx) {
                switch_stmt.cases.push(case);
            }
        }
    }

    fn try_drop_content_from_case(case: &mut SwitchCase<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        if case.consequent.len() == 1 && Self::is_keep_var_canonical(&case.consequent[0]) {
            return false;
        }
        let mut keep_var = KeepVar::new();
        keep_var.visit_switch_case(case);
        if let Some(var_decl) = keep_var.get_variable_declaration_statement(ctx) {
            for dropped_stmt in &case.consequent {
                ctx.drop_statement(dropped_stmt);
            }
            case.consequent = ArenaVec::from_array_in([var_decl], ctx);
            false
        } else {
            // unreachable cases
            ctx.drop_switch_case(case);
            true
        }
    }
}
