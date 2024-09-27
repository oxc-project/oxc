use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{CompressOptions, CompressorPass};

/// Remove syntax from the AST.
///
/// * Parenthesized Expression
/// * `debugger`
/// * `console.log`
pub struct RemoveSyntax {
    options: CompressOptions,
}

impl<'a> CompressorPass<'a> for RemoveSyntax {
    fn changed(&self) -> bool {
        false
    }

    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        oxc_traverse::walk_program(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for RemoveSyntax {
    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| {
            !(matches!(stmt, Statement::EmptyStatement(_))
                || self.drop_debugger(stmt)
                || self.drop_console(stmt))
        });
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.compress_console(expr, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::strip_parenthesized_expression(expr, ctx);
    }

    fn exit_arrow_function_expression(
        &mut self,
        expr: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.recover_arrow_expression_after_drop_console(expr);
    }
}

impl<'a> RemoveSyntax {
    pub fn new(options: CompressOptions) -> Self {
        Self { options }
    }

    fn strip_parenthesized_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::ParenthesizedExpression(paren_expr) = expr {
            *expr = ctx.ast.move_expression(&mut paren_expr.expression);
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

    fn compress_console(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.drop_console && Self::is_console(expr) {
            *expr = ctx.ast.void_0(expr.span());
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

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{tester, CompressOptions};

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::RemoveSyntax::new(CompressOptions::all_true());
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    #[test]
    fn parens() {
        test("(((x)))", "x");
        test("(((a + b))) * c", "(a + b) * c");
    }

    #[test]
    fn drop_console() {
        test("console.log()", "");
    }

    #[test]
    fn drop_debugger() {
        test("debugger", "");
    }
}
