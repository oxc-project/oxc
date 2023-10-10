use oxc_allocator::{Allocator, Vec};

#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, VisitMut};

pub struct Prepass<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> Prepass<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { ast: AstBuilder::new(allocator) }
    }

    fn strip_parenthesized_expression<'b>(&self, expr: &'b mut Expression<'a>) {
        if let Expression::ParenthesizedExpression(paren_expr) = expr {
            *expr = self.ast.move_expression(&mut paren_expr.expression);
            self.strip_parenthesized_expression(expr);
        }
    }
}

impl<'a, 'b> VisitMut<'a, 'b> for Prepass<'a> {
    fn visit_statements(&mut self, stmts: &'b mut Vec<'a, Statement<'a>>) {
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        for stmt in stmts.iter_mut() {
            self.visit_statement(stmt);
        }
    }

    fn visit_expression(&mut self, expr: &'b mut Expression<'a>) {
        self.strip_parenthesized_expression(expr);
        self.visit_expression_match(expr);
    }
}
