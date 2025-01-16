use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::ConstantEvaluation;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{ctx::Ctx, CompressOptions, CompressorPass};

#[derive(Default)]
pub struct NormalizeOptions {
    pub convert_while_to_fors: bool,
}

/// Normalize AST
///
/// Make subsequent AST passes easier to analyze:
///
/// * remove `Statement::EmptyStatement`
/// * remove `ParenthesizedExpression`
/// * convert whiles to fors
/// * convert `Infinity` to `f64::INFINITY`
/// * convert `NaN` to `f64::NaN`
/// * convert `var x; void x` to `void 0`
///
/// Also
///
/// * remove `debugger` and `console.log` (optional)
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/Normalize.java>
pub struct Normalize {
    options: NormalizeOptions,
    compress_options: CompressOptions,
}

impl<'a> CompressorPass<'a> for Normalize {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for Normalize {
    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| {
            !(matches!(stmt, Statement::EmptyStatement(_))
                || self.drop_debugger(stmt)
                || self.drop_console(stmt))
        });
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::IfStatement(s) => Self::wrap_to_avoid_ambiguous_else(s, ctx),
            Statement::WhileStatement(_) if self.options.convert_while_to_fors => {
                Self::convert_while_to_for(stmt, ctx);
            }
            _ => {}
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::ParenthesizedExpression(paren_expr) = expr {
            *expr = ctx.ast.move_expression(&mut paren_expr.expression);
        }
        match expr {
            Expression::Identifier(_) => {
                Self::convert_infinity_or_nan_into_number(expr, ctx);
            }
            Expression::UnaryExpression(e) if e.operator.is_void() => {
                Self::convert_void_ident(e, ctx);
            }
            Expression::ArrowFunctionExpression(e) => {
                self.recover_arrow_expression_after_drop_console(e);
            }
            Expression::CallExpression(_) if self.compress_options.drop_console => {
                self.compress_console(expr, ctx);
            }
            _ => {}
        }
    }
}

impl<'a> Normalize {
    pub fn new(options: NormalizeOptions, compress_options: CompressOptions) -> Self {
        Self { options, compress_options }
    }

    /// Drop `drop_debugger` statement.
    ///
    /// Enabled by `compress.drop_debugger`
    fn drop_debugger(&mut self, stmt: &Statement<'a>) -> bool {
        matches!(stmt, Statement::DebuggerStatement(_)) && self.compress_options.drop_debugger
    }

    fn compress_console(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        debug_assert!(self.compress_options.drop_console);
        if Self::is_console(expr) {
            *expr = ctx.ast.void_0(expr.span());
        }
    }

    fn drop_console(&mut self, stmt: &Statement<'a>) -> bool {
        self.compress_options.drop_console
            && matches!(stmt, Statement::ExpressionStatement(expr) if Self::is_console(&expr.expression))
    }

    fn recover_arrow_expression_after_drop_console(&self, expr: &mut ArrowFunctionExpression<'a>) {
        if self.compress_options.drop_console && expr.expression && expr.body.is_empty() {
            expr.expression = false;
        }
    }

    fn is_console(expr: &Expression<'_>) -> bool {
        let Expression::CallExpression(call_expr) = &expr else { return false };
        let Some(member_expr) = call_expr.callee.as_member_expression() else { return false };
        let obj = member_expr.object();
        let Some(ident) = obj.get_identifier_reference() else { return false };
        ident.name == "console"
    }

    fn convert_while_to_for(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::WhileStatement(while_stmt) = ctx.ast.move_statement(stmt) else { return };
        let while_stmt = while_stmt.unbox();
        let for_stmt = ctx.ast.alloc_for_statement_with_scope_id(
            while_stmt.span,
            None,
            Some(while_stmt.test),
            None,
            while_stmt.body,
            ctx.create_child_scope_of_current(ScopeFlags::empty()),
        );
        *stmt = Statement::ForStatement(for_stmt);
    }

    fn convert_infinity_or_nan_into_number(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::Identifier(ident) = expr {
            let ctx = Ctx(ctx);
            let value = if ctx.is_identifier_infinity(ident) {
                f64::INFINITY
            } else if ctx.is_identifier_nan(ident) {
                f64::NAN
            } else {
                return;
            };
            *expr =
                ctx.ast.expression_numeric_literal(ident.span, value, None, NumberBase::Decimal);
        }
    }

    // Wrap to avoid ambiguous else.
    // `if (foo) if (bar) baz else quaz` ->  `if (foo) { if (bar) baz else quaz }`
    fn wrap_to_avoid_ambiguous_else(if_stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::IfStatement(if2) = &mut if_stmt.consequent {
            if if2.alternate.is_some() {
                let scope_id = ctx.create_child_scope_of_current(ScopeFlags::empty());
                if_stmt.consequent =
                    Statement::BlockStatement(ctx.ast.alloc_block_statement_with_scope_id(
                        if_stmt.consequent.span(),
                        ctx.ast.vec1(ctx.ast.move_statement(&mut if_stmt.consequent)),
                        scope_id,
                    ));
            }
        }
    }

    fn convert_void_ident(e: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        debug_assert!(e.operator.is_void());
        let Expression::Identifier(ident) = &e.argument else { return };
        if Ctx(ctx).is_global_reference(ident) {
            return;
        }
        e.argument = ctx.ast.expression_numeric_literal(ident.span, 0.0, None, NumberBase::Decimal);
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use super::NormalizeOptions;
    use crate::{tester, CompressOptions};

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let compress_options = CompressOptions {
            drop_debugger: true,
            drop_console: true,
            ..CompressOptions::default()
        };
        let options = NormalizeOptions { convert_while_to_fors: true };
        let mut pass = super::Normalize::new(options, compress_options);
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    #[test]
    fn test_while() {
        // Verify while loops are converted to FOR loops.
        test("while(c < b) foo()", "for(; c < b;) foo()");
    }

    #[test]
    fn test_void_ident() {
        test("var x; void x", "var x; void 0");
        test("void x", "void x"); // reference error
    }

    #[test]
    fn parens() {
        test("(((x)))", "x");
        test("(((a + b))) * c", "(a + b) * c");
    }

    #[test]
    fn drop_console() {
        test("console.log()", "void 0;\n");
        test("() => console.log()", "() => void 0");
    }

    #[test]
    fn drop_debugger() {
        test("debugger", "");
    }
}
