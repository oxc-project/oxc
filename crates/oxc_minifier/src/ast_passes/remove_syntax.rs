use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

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
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for RemoveSyntax {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::drop_use_strict_directives_in_program(program, ctx);
    }

    fn enter_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::drop_use_strict_directives_in_function_body(body, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::drop_use_strict_directives_if_function_is_empty(body, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| {
            !(matches!(stmt, Statement::EmptyStatement(_))
                || self.drop_debugger(stmt)
                || self.drop_console(stmt))
        });
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.compress_console(expr, ctx);
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

    /// Drop `"use strict";` directives if the input is strict mode (e.g. written in ESM).
    fn drop_use_strict_directives_in_program(
        program: &mut Program<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if program.source_type.is_strict() {
            program.directives.retain(|directive| !directive.is_use_strict());
        }
    }

    /// Drop `"use strict";` directives if the parent scope is already strict mode.
    fn drop_use_strict_directives_in_function_body(
        body: &mut FunctionBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let current_scope_id = ctx.current_scope_id();
        let Some(parent_scope_id) = ctx.scopes().get_parent_id(current_scope_id) else { return };
        if ctx.scopes().get_flags(parent_scope_id).is_strict_mode() {
            body.directives.retain(|directive| !directive.is_use_strict());
        }
    }

    /// Drop `"use strict";` directives if the function is empty.
    fn drop_use_strict_directives_if_function_is_empty(
        body: &mut FunctionBody<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if body.statements.is_empty() {
            body.directives.retain(|directive| !directive.is_use_strict());
        }
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

    fn test_script(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let mut pass = super::RemoveSyntax::new(CompressOptions::all_true());
        tester::test_impl(
            &allocator,
            source_text,
            expected,
            &mut pass,
            oxc_span::SourceType::cjs(),
            true,
        );
    }

    fn test_script_same(source_text: &str) {
        test_script(source_text, source_text);
    }

    #[test]
    fn parens() {
        test("(((x)))", "x");
        test("(((a + b))) * c", "(a + b) * c");
    }

    #[test]
    fn drop_console() {
        test("console.log()", "void 0;\n");
    }

    #[test]
    fn drop_debugger() {
        test("debugger", "");
    }

    #[test]
    fn use_strict() {
        test("'use strict';", "");

        test_script(
            "'use strict'; function foo() { 'use strict'; alert(1); }",
            "'use strict'; function foo() { alert(1); }",
        );
        test_script(
            "'use strict'; const foo = () => { 'use strict'; alert(1); }",
            "'use strict'; const foo = () => { alert(1); }",
        );
        test_script_same("function foo() { 'use strict'; alert(1); }");
        test_script(
            "function foo() { 'use strict'; return function foo() { 'use strict'; alert(1); }; } ",
            "function foo() { 'use strict'; return function foo() { alert(1); }; } ",
        );
        test_script(
            "class Foo { foo() { 'use strict'; alert(1); } } ",
            "class Foo { foo() { alert(1); } } ",
        );
        test_script(
            "const Foo = class { foo() { 'use strict'; alert(1); } } ",
            "const Foo = class { foo() { alert(1); } } ",
        );

        test_script("function foo() { 'use strict';}", "function foo() {}");
    }
}
