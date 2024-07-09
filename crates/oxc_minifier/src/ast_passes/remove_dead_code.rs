use oxc_allocator::Allocator;
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

impl<'a> RemoveDeadCode<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        let ast = AstBuilder::new(allocator);
        Self { ast, folder: Folder::new(ast) }
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    fn test_expression(&mut self, expr: &mut Expression<'a>) -> Option<bool> {
        self.folder.fold_expression(expr);
        get_boolean_value(expr)
    }

    pub fn remove_if(&mut self, stmt: &mut Statement<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };
        match self.test_expression(&mut if_stmt.test) {
            Some(true) => {
                *stmt = self.ast.move_statement(&mut if_stmt.consequent);
            }
            Some(false) => {
                *stmt = if let Some(alternate) = &mut if_stmt.alternate {
                    self.ast.move_statement(alternate)
                } else {
                    self.ast.statement_empty(SPAN)
                };
            }
            _ => {}
        }
    }

    pub fn remove_conditional(&mut self, stmt: &mut Statement<'a>) {
        let Statement::ExpressionStatement(expression_stmt) = stmt else { return };
        let Expression::ConditionalExpression(conditional_expr) = &mut expression_stmt.expression
        else {
            return;
        };
        match self.test_expression(&mut conditional_expr.test) {
            Some(true) => {
                expression_stmt.expression =
                    self.ast.move_expression(&mut conditional_expr.consequent);
            }
            Some(false) => {
                expression_stmt.expression =
                    self.ast.move_expression(&mut conditional_expr.alternate);
            }
            _ => {}
        }
    }
}

impl<'a> VisitMut<'a> for RemoveDeadCode<'a> {
    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.remove_if(stmt);
        self.remove_conditional(stmt);
        walk_mut::walk_statement(self, stmt);
    }
}
