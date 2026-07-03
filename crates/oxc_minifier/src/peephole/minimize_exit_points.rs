use crate::{TraverseCtx, generated::ancestor::Ancestor};
use bitflags::bitflags;
use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_ecmascript::{constant_evaluation::ConstantEvaluation, side_effects::MayHaveSideEffects};
use oxc_span::SPAN;

use super::PeepholeOptimizations;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct RemoveFlag: u8 {
        const RETURN = 1;
        const CONTINUE = 1 << 1;
        const BREAK = 1 << 2;
    }
}

impl<'a> PeepholeOptimizations {
    /// Remove exit statements (return, continue, break) if they are the last statement in a block and are not needed.
    pub fn remove_exit_statements(
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let flags = match ctx.parent() {
            Ancestor::FunctionBodyStatements(_) => RemoveFlag::RETURN,
            Ancestor::DoWhileStatementBody(_)
            | Ancestor::WhileStatementBody(_)
            | Ancestor::ForStatementBody(_)
            | Ancestor::ForInStatementBody(_)
            | Ancestor::ForOfStatementBody(_) => RemoveFlag::CONTINUE,
            _ => RemoveFlag::empty(),
        };
        Self::prune_tail_exit_in_statement_list(stmts, flags, ctx);
    }

    fn is_removable_exit_statement(stmt: &Statement<'a>, flags: RemoveFlag) -> bool {
        match stmt {
            Statement::ContinueStatement(s) => {
                flags.contains(RemoveFlag::CONTINUE) && s.label.is_none()
            }
            Statement::ReturnStatement(s) => {
                flags.contains(RemoveFlag::RETURN) && s.argument.is_none()
            }
            Statement::BreakStatement(s) => flags.contains(RemoveFlag::BREAK) && s.label.is_none(),
            _ => false,
        }
    }

    fn prune_tail_exit_in_child_statement(
        stmt: &mut Statement<'a>,
        flags: RemoveFlag,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if Self::is_removable_exit_statement(stmt, flags) {
            let empty = Statement::new_empty_statement(SPAN, ctx);
            ctx.replace_statement(stmt, empty);
            return;
        }
        Self::visit_tail_exit_contexts(stmt, flags, ctx);
    }

    fn prune_tail_exit_in_statement_list(
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        flags: RemoveFlag,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(last_stmt) = stmts.last_mut() {
            if Self::is_removable_exit_statement(last_stmt, flags) {
                let dropped = stmts.pop().unwrap();
                ctx.drop_statement(&dropped);
                return;
            }
            Self::visit_tail_exit_contexts(last_stmt, flags, ctx);
        }
    }

    fn visit_tail_exit_contexts(
        stmt: &mut Statement<'a>,
        flags: RemoveFlag,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match stmt {
            Statement::BlockStatement(s) => {
                Self::prune_tail_exit_in_statement_list(&mut s.body, flags, ctx);
            }
            Statement::IfStatement(s) => {
                // "if (x) { return; } else { return; }"
                Self::prune_tail_exit_in_child_statement(&mut s.consequent, flags, ctx);
                if let Some(alternate) = s.alternate.as_mut() {
                    Self::prune_tail_exit_in_child_statement(alternate, flags, ctx);
                }
            }
            Statement::TryStatement(s) => {
                // If there is a finalizer, we cannot remove exit statements in the try or catch blocks
                // "try { return; } catch { return; } finally { return; }"
                let nested_flags = if s.finalizer.is_none() { flags } else { RemoveFlag::empty() };
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
                        RemoveFlag::empty(),
                        ctx,
                    );
                }
            }
            Statement::WhileStatement(s) => {
                // "while (true) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::ForStatement(s) => {
                // "for (;;) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::DoWhileStatement(s) => {
                // "do { continue; } while (*)"
                let mut body_flags = RemoveFlag::CONTINUE;
                if s.test.evaluate_value_to_boolean(ctx) == Some(false)
                    && !s.test.may_have_side_effects(ctx)
                {
                    // "do { break; } while (false)"
                    body_flags |= RemoveFlag::BREAK;
                }
                Self::prune_tail_exit_in_child_statement(&mut s.body, body_flags, ctx);
            }
            Statement::ForInStatement(s) => {
                // "for (var x in y) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::ForOfStatement(s) => {
                // "for (var x of y) { continue; }"
                Self::prune_tail_exit_in_child_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::SwitchStatement(s) => {
                if let Some(last_case) = s.cases.last_mut() {
                    // "switch (x) { case 1: break; }"
                    Self::prune_tail_exit_in_statement_list(
                        &mut last_case.consequent,
                        flags | RemoveFlag::BREAK,
                        ctx,
                    );
                }
            }
            Statement::LabeledStatement(s) => {
                Self::prune_tail_exit_in_child_statement(&mut s.body, flags, ctx);
            }
            _ => {}
        }
    }
}
