use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder, Visit};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{keep_var::KeepVar, CompressorPass};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
pub struct RemoveDeadCode<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> CompressorPass<'a> for RemoveDeadCode<'a> {}

impl<'a> Traverse<'a> for RemoveDeadCode<'a> {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        self.dead_code_elimination(stmts);
    }
}

impl<'a> RemoveDeadCode<'a> {
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast }
    }

    /// Removes dead code thats comes after `return` statements after inlining `if` statements
    fn dead_code_elimination(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
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

        let mut keep_var = KeepVar::new(self.ast);

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
}
