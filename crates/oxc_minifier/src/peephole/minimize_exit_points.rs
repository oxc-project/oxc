use crate::TraverseCtx;
use bitflags::bitflags;
use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::ConstantEvaluation;
use oxc_span::GetSpan;

use super::PeepholeOptimizations;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct JumpType: u8 {
        const Return = 1;
        const Continue = 1 << 1;
        const Break = 1 << 2;
    }
}

impl<'a> PeepholeOptimizations {
    /// Remove exit statements (return, continue, break) if they are the last statement in a block and are not needed.
    pub fn remove_exit_statements(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::WhileStatement(s) => {
                // "while (true) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, JumpType::Continue, ctx);
                Self::try_minify_statements(&mut s.body, JumpType::Continue, ctx);
            }
            Statement::ForStatement(s) => {
                // "for (;;) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, JumpType::Continue, ctx);
                Self::try_minify_statements(&mut s.body, JumpType::Continue, ctx);
            }
            Statement::ForInStatement(s) => {
                // "for (var x in y) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, JumpType::Continue, ctx);
                Self::try_minify_statements(&mut s.body, JumpType::Continue, ctx);
            }
            Statement::ForOfStatement(s) => {
                // "for (var x of y) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, JumpType::Continue, ctx);
                Self::try_minify_statements(&mut s.body, JumpType::Continue, ctx);
            }
            Statement::IfStatement(s) => {
                Self::try_minify_statements(&mut s.consequent, JumpType::empty(), ctx);
                if let Some(alternate) = &mut s.alternate {
                    Self::try_minify_statements(alternate, JumpType::empty(), ctx);
                }
            }
            Statement::DoWhileStatement(s) => {
                // "do { continue; } while (*)"
                let jump_type = if s.test.get_side_free_boolean_value(ctx) == Some(false) {
                    JumpType::Continue | JumpType::Break
                } else {
                    JumpType::Continue
                };
                Self::prune_tail_exit_in_child_statement(&mut s.body, jump_type, ctx);
                Self::try_minify_statements(&mut s.body, jump_type, ctx);
            }
            // "switch (x) { case 1: break; }"
            Statement::SwitchStatement(s) => {
                if let Some(last_case) = s.cases.last_mut() {
                    Self::prune_tail_exit_in_statement_list(
                        &mut last_case.consequent,
                        JumpType::Break,
                        ctx,
                    );
                }
            }
            Statement::FunctionDeclaration(s) => {
                if let Some(body) = &mut s.body {
                    Self::prune_tail_exit_in_statement_list(
                        &mut body.statements,
                        JumpType::Return,
                        ctx,
                    );
                }
            }
            _ => return,
        };
    }

    /// Returns `true` if the statement is an unconditional termination that can be
    /// safely removed:
    /// - Unlabeled `continue` statements that terminate a loop body
    /// - Bare `return` statements that terminate a function body
    pub fn can_remove_termination_statement(stmt: &Statement<'a>, jump_type: JumpType) -> bool {
        match stmt {
            // unlabeled `continue;` that terminates a `for`, `for...in`, `for...of`, or `while` body.
            Statement::ContinueStatement(stmt) if stmt.label.is_none() => {
                jump_type.contains(JumpType::Continue)
            }
            // unlabeled `break;` that terminates a `do...while` body if test is false.
            Statement::BreakStatement(stmt) if stmt.label.is_none() => {
                jump_type.contains(JumpType::Break)
            }
            // bare `return;` in function-body scope.
            Statement::ReturnStatement(stmt) if stmt.argument.is_none() => {
                jump_type.contains(JumpType::Return)
            }
            _ => false,
        }
    }

    fn prune_tail_exit_in_child_statement(
        stmt: &mut Statement<'a>,
        jump_type: JumpType,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if Self::can_remove_termination_statement(stmt, jump_type) {
            let empty = Statement::new_empty_statement(stmt.span(), ctx);
            ctx.replace_statement(stmt, empty);
            return;
        }
        Self::visit_tail_exit_contexts(stmt, jump_type, ctx);
    }

    fn prune_tail_exit_in_statement_list(
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        jump_type: JumpType,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(last_stmt) = stmts.last_mut() {
            if Self::can_remove_termination_statement(last_stmt, jump_type) {
                let dropped = stmts.pop().unwrap();
                ctx.drop_statement(&dropped);
                return;
            }
            Self::visit_tail_exit_contexts(last_stmt, jump_type, ctx);
        }
    }

    fn visit_tail_exit_contexts(
        stmt: &mut Statement<'a>,
        jump_type: JumpType,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match stmt {
            Statement::BlockStatement(s) => {
                Self::prune_tail_exit_in_statement_list(&mut s.body, jump_type, ctx);
            }
            Statement::IfStatement(s) => {
                // "if (x) { return; } else { return; }"
                Self::prune_tail_exit_in_child_statement(&mut s.consequent, jump_type, ctx);
                if let Some(alternate) = s.alternate.as_mut() {
                    Self::prune_tail_exit_in_child_statement(alternate, jump_type, ctx);
                }
            }
            Statement::TryStatement(s) => {
                // If there is a finalizer, we cannot remove exit statements in the try or catch blocks
                // "try { return; } catch { return; } finally { return; }"
                let nested_flags =
                    if s.finalizer.is_none() { jump_type } else { JumpType::empty() };
                Self::prune_tail_exit_in_statement_list(&mut s.block.body, nested_flags, ctx);
                if let Some(handler) = s.handler.as_mut() {
                    Self::prune_tail_exit_in_statement_list(
                        &mut handler.body.body,
                        nested_flags,
                        ctx,
                    );
                }
                if let Some(finalizer) = s.finalizer.as_mut() {
                    Self::prune_tail_exit_in_statement_list(
                        &mut finalizer.body,
                        JumpType::empty(),
                        ctx,
                    );
                }
            }
            Statement::LabeledStatement(s) => {
                Self::prune_tail_exit_in_child_statement(&mut s.body, jump_type, ctx);
            }
            _ => {}
        }
    }
}
