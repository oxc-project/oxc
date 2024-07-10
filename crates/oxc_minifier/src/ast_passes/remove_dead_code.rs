use oxc_allocator::{Allocator, Vec};
use oxc_ast::{ast::*, visit::walk_mut, AstBuilder, VisitMut};
use oxc_span::SPAN;

use crate::{compressor::ast_util::get_boolean_value, folder::Folder};

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
pub struct RemoveDeadCode<'a> {
    ast: AstBuilder<'a>,
    folder: Folder<'a>,
}

impl<'a> VisitMut<'a> for RemoveDeadCode<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.dead_code_elimintation(stmts);
        walk_mut::walk_statements(self, stmts);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.fold_conditional_expression(expr);
    }
}

impl<'a> RemoveDeadCode<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        Self { ast, folder: Folder::new(ast) }
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    /// Removes dead code thats comes after `return` statements after inlining `if` statements
    fn dead_code_elimintation(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        let mut removed = true;
        for stmt in stmts.iter_mut() {
            if self.fold_if_statement(stmt) {
                removed = true;
                break;
            }
        }

        if !removed {
            return;
        }

        let mut index = None;
        for (i, stmt) in stmts.iter().enumerate() {
            if matches!(stmt, Statement::ReturnStatement(_)) {
                index.replace(i);
            }
        }
        if let Some(index) = index {
            stmts.drain(index + 1..);
        }
    }

    #[must_use]
    fn fold_if_statement(&mut self, stmt: &mut Statement<'a>) -> bool {
        let Statement::IfStatement(if_stmt) = stmt else { return false };
        match self.fold_expression_and_get_boolean_value(&mut if_stmt.test) {
            Some(true) => {
                *stmt = self.ast.move_statement(&mut if_stmt.consequent);
                true
            }
            Some(false) => {
                *stmt = if let Some(alternate) = &mut if_stmt.alternate {
                    self.ast.move_statement(alternate)
                } else {
                    self.ast.statement_empty(SPAN)
                };
                true
            }
            _ => false,
        }
    }

    fn fold_expression_and_get_boolean_value(&mut self, expr: &mut Expression<'a>) -> Option<bool> {
        self.folder.fold_expression(expr);
        get_boolean_value(expr)
    }

    fn fold_conditional_expression(&mut self, expr: &mut Expression<'a>) {
        let Expression::ConditionalExpression(conditional_expr) = expr else {
            return;
        };
        match self.fold_expression_and_get_boolean_value(&mut conditional_expr.test) {
            Some(true) => {
                *expr = self.ast.move_expression(&mut conditional_expr.consequent);
            }
            Some(false) => {
                *expr = self.ast.move_expression(&mut conditional_expr.alternate);
            }
            _ => {}
        }
    }
}
