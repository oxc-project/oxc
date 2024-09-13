use oxc_allocator::Vec;
use oxc_ast::{ast::*, Visit};
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{keep_var::KeepVar, node_util::NodeUtil, tri::Tri, CompressorPass};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
pub struct RemoveDeadCode;

impl<'a> CompressorPass<'a> for RemoveDeadCode {}

impl<'a> Traverse<'a> for RemoveDeadCode {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::fold_if_statement(stmt, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        self.dead_code_elimination(stmts, ctx);
    }
}

impl<'a> RemoveDeadCode {
    pub fn new() -> Self {
        Self {}
    }

    /// Removes dead code thats comes after `return` statements after inlining `if` statements
    fn dead_code_elimination(
        &mut self,
        stmts: &mut Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Remove code after `return` and `throw` statements
        let mut index = None;
        'outer: for (i, stmt) in stmts.iter().enumerate() {
            if matches!(stmt, Statement::ReturnStatement(_) | Statement::ThrowStatement(_)) {
                index.replace(i);
                break;
            }
            // Double check block statements folded by if statements above
            if let Statement::BlockStatement(block_stmt) = stmt {
                for stmt in &block_stmt.body {
                    if matches!(stmt, Statement::ReturnStatement(_) | Statement::ThrowStatement(_))
                    {
                        index.replace(i);
                        break 'outer;
                    }
                }
            }
        }

        let Some(index) = index else { return };
        if index == stmts.len() - 1 {
            return;
        }

        let mut keep_var = KeepVar::new(ctx.ast);

        for stmt in stmts.iter().skip(index + 1) {
            keep_var.visit_statement(stmt);
        }

        let mut i = 0;
        stmts.retain(|s| {
            i += 1;
            if i - 1 <= index {
                return true;
            }
            // keep function declaration
            if matches!(s.as_declaration(), Some(Declaration::FunctionDeclaration(_))) {
                return true;
            }
            false
        });

        if let Some(stmt) = keep_var.get_variable_declaration_statement() {
            stmts.push(stmt);
        }
    }

    fn fold_if_statement(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };

        // Descend and remove `else` blocks first.
        if let Some(alternate) = &mut if_stmt.alternate {
            Self::fold_if_statement(alternate, ctx);
            if matches!(alternate, Statement::EmptyStatement(_)) {
                if_stmt.alternate = None;
            }
        }

        match ctx.get_boolean_value(&if_stmt.test) {
            Tri::True => {
                *stmt = ctx.ast.move_statement(&mut if_stmt.consequent);
            }
            Tri::False => {
                *stmt = if let Some(alternate) = &mut if_stmt.alternate {
                    ctx.ast.move_statement(alternate)
                } else {
                    // Keep hoisted `vars` from the consequent block.
                    let mut keep_var = KeepVar::new(ctx.ast);
                    keep_var.visit_statement(&if_stmt.consequent);
                    keep_var
                        .get_variable_declaration_statement()
                        .unwrap_or_else(|| ctx.ast.statement_empty(SPAN))
                };
            }
            Tri::Unknown => {}
        }
    }
}
