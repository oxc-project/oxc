use oxc_allocator::Vec;
use oxc_ast::{ast::*, visit::walk_mut, AstBuilder, VisitMut};

use crate::CompressOptions;

/// Remove syntax from the AST.
///
/// * Parenthesized Expression
/// * `debugger`
/// * `console.log`
pub struct RemoveSyntax<'a> {
    ast: AstBuilder<'a>,
    options: CompressOptions,
}

impl<'a> VisitMut<'a> for RemoveSyntax<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        stmts.retain(|stmt| {
            !(matches!(stmt, Statement::EmptyStatement(_))
                || self.drop_debugger(stmt)
                || self.drop_console(stmt))
        });
        walk_mut::walk_statements(self, stmts);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.strip_parenthesized_expression(expr);
        self.compress_console(expr);
        walk_mut::walk_expression(self, expr);
    }
}

impl<'a> RemoveSyntax<'a> {
    pub fn new(ast: AstBuilder<'a>, options: CompressOptions) -> Self {
        Self { ast, options }
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

    /// Drop `drop_debugger` statement.
    ///
    /// Enabled by `compress.drop_debugger`
    fn drop_debugger(&mut self, stmt: &Statement<'a>) -> bool {
        matches!(stmt, Statement::DebuggerStatement(_)) && self.options.drop_debugger
    }

    /// Drop `console.*` expressions.
    ///
    /// Enabled by `compress.drop_console
    fn drop_console(&mut self, stmt: &Statement<'a>) -> bool {
        self.options.drop_console
            && matches!(stmt, Statement::ExpressionStatement(expr) if Self::is_console(&expr.expression))
    }

    fn compress_console(&mut self, expr: &mut Expression<'a>) {
        if self.options.drop_console && Self::is_console(expr) {
            *expr = self.ast.void_0();
        }
    }

    fn is_console(expr: &Expression<'_>) -> bool {
        // let Statement::ExpressionStatement(expr) = stmt else { return false };
        let Expression::CallExpression(call_expr) = &expr else { return false };
        let Some(member_expr) = call_expr.callee.as_member_expression() else { return false };
        let obj = member_expr.object();
        let Some(ident) = obj.get_identifier_reference() else { return false };
        ident.name == "console"
    }
}
