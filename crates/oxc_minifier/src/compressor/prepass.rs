use oxc_allocator::{Allocator, Vec};

use oxc_ast::visit::walk_mut::{walk_expression_mut, walk_statements_mut};
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, VisitMut};

pub struct Prepass<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> Prepass<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { ast: AstBuilder::new(allocator) }
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    fn strip_parenthesized_expression(&self, expr: &mut Expression<'a>) {
        if let Expression::ParenthesizedExpression(paren_expr) = expr {
            *expr = self.ast.move_expression(&mut paren_expr.expression);
            self.strip_parenthesized_expression(expr);
        }
    }
}

impl<'a> VisitMut<'a> for Prepass<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        walk_statements_mut(self, stmts);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.strip_parenthesized_expression(expr);
        walk_expression_mut(self, expr);
    }
}
