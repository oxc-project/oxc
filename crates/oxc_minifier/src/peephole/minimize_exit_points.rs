use crate::{TraverseCtx, generated::ancestor::Ancestor};
use bitflags::bitflags;
use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::ConstantEvaluation;
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
        Self::remove_exit_points_in_statements(stmts, flags, ctx);
    }

    fn remove_exit_points_in_statements(
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        flags: RemoveFlag,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(last_stmt) = stmts.last_mut() {
            Self::remove_exit_points_in_statement(last_stmt, flags, ctx);
        }
    }

    fn remove_exit_points_in_statement(
        stmt: &mut Statement<'a>,
        flags: RemoveFlag,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match stmt {
            Statement::ContinueStatement(s)
                if flags.contains(RemoveFlag::CONTINUE) && s.label.is_none() =>
            {
                ctx.replace_statement(stmt, Statement::new_empty_statement(SPAN, ctx));
            }
            Statement::ReturnStatement(s)
                if flags.contains(RemoveFlag::RETURN) && s.argument.is_none() =>
            {
                ctx.replace_statement(stmt, Statement::new_empty_statement(SPAN, ctx));
            }
            Statement::BreakStatement(s)
                if flags.contains(RemoveFlag::BREAK) && s.label.is_none() =>
            {
                ctx.replace_statement(stmt, Statement::new_empty_statement(SPAN, ctx));
            }
            Statement::BlockStatement(s) => {
                Self::remove_exit_points_in_statements(&mut s.body, flags, ctx);
            }
            Statement::IfStatement(s) => {
                Self::remove_exit_points_in_statement(&mut s.consequent, flags, ctx);
                if let Some(alternate) = s.alternate.as_mut() {
                    Self::remove_exit_points_in_statement(alternate, flags, ctx);
                }
            }
            Statement::TryStatement(s) => {
                let nested_flags = if s.finalizer.is_none() { flags } else { RemoveFlag::empty() };
                Self::remove_exit_points_in_statements(&mut s.block.body, nested_flags, ctx);
                if let Some(handler) = s.handler.as_mut() {
                    Self::remove_exit_points_in_statements(
                        &mut handler.body.body,
                        nested_flags,
                        ctx,
                    );
                }
                if let Some(finalizer) = s.finalizer.as_mut() {
                    Self::remove_exit_points_in_statements(
                        &mut finalizer.body,
                        RemoveFlag::empty(),
                        ctx,
                    );
                }
            }
            Statement::WhileStatement(s) => {
                Self::remove_exit_points_in_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::ForStatement(s) => {
                Self::remove_exit_points_in_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::DoWhileStatement(s) => {
                let mut body_flags = RemoveFlag::CONTINUE;
                if s.test.evaluate_value_to_boolean(ctx) == Some(false) {
                    body_flags |= RemoveFlag::BREAK;
                }
                Self::remove_exit_points_in_statement(&mut s.body, body_flags, ctx);
            }
            Statement::ForInStatement(s) => {
                Self::remove_exit_points_in_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::ForOfStatement(s) => {
                Self::remove_exit_points_in_statement(&mut s.body, RemoveFlag::CONTINUE, ctx);
            }
            Statement::SwitchStatement(s) => {
                if let Some(last_case) = s.cases.last_mut() {
                    Self::remove_exit_points_in_statements(
                        &mut last_case.consequent,
                        flags | RemoveFlag::BREAK,
                        ctx,
                    );
                }
            }
            Statement::LabeledStatement(s) => {
                Self::remove_exit_points_in_statement(&mut s.body, flags, ctx);
            }
            _ => {}
        }
    }
}
