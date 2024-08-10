use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{CompressOptions, CompressorPass};

/// Remove syntax from the AST.
///
/// * Parenthesized Expression
/// * `debugger`
/// * `console.log`
pub struct RemoveSyntax<'a> {
    ast: AstBuilder<'a>,
    options: CompressOptions,
}

impl<'a> CompressorPass<'a> for RemoveSyntax<'a> {}

impl<'a> Traverse<'a> for RemoveSyntax<'a> {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| {
            !(matches!(stmt, Statement::EmptyStatement(_))
                || self.drop_debugger(stmt)
                || self.drop_console(stmt))
        });
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.strip_parenthesized_expression(expr);
        self.compress_console(expr);
    }

    fn exit_arrow_function_expression(
        &mut self,
        expr: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.recover_arrow_expression_after_drop_console(expr);
    }
}

impl<'a> RemoveSyntax<'a> {
    pub fn new(ast: AstBuilder<'a>, options: CompressOptions) -> Self {
        Self { ast, options }
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

    fn recover_arrow_expression_after_drop_console(&self, expr: &mut ArrowFunctionExpression<'a>) {
        if self.options.drop_console && expr.expression && expr.body.is_empty() {
            expr.expression = false;
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
