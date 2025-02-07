use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_semantic::IsGlobalReference;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::{traverse_mut_with_ctx, Ancestor, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{ctx::Ctx, CompressOptions};

#[derive(Default)]
pub struct NormalizeOptions {
    pub convert_while_to_fors: bool,
    pub convert_const_to_let: bool,
}

/// Normalize AST
///
/// Make subsequent AST passes easier to analyze:
///
/// * remove `Statement::EmptyStatement`
/// * remove `ParenthesizedExpression`
/// * convert whiles to fors
/// * convert `const` to `let` for non-exported variables
/// * convert `Infinity` to `f64::INFINITY`
/// * convert `NaN` to `f64::NaN`
/// * convert `var x; void x` to `void 0`
/// * convert `undefined` to `void 0`
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

impl<'a> Normalize {
    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
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

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.convert_const_to_let {
            Self::convert_const_to_let(decl, ctx);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
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
        if let Some(e) = match expr {
            Expression::Identifier(ident) => Self::try_compress_identifier(ident, ctx),
            Expression::UnaryExpression(e) if e.operator.is_void() => {
                Self::convert_void_ident(e, ctx);
                None
            }
            Expression::ArrowFunctionExpression(e) => {
                self.recover_arrow_expression_after_drop_console(e);
                None
            }
            Expression::CallExpression(_) if self.compress_options.drop_console => {
                self.compress_console(expr, ctx)
            }
            _ => None,
        } {
            *expr = e;
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

    fn compress_console(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        debug_assert!(self.compress_options.drop_console);
        Self::is_console(expr).then(|| ctx.ast.void_0(expr.span()))
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

    fn convert_const_to_let(decl: &mut VariableDeclaration<'a>, ctx: &mut TraverseCtx<'a>) {
        // checking whether the current scope is the root scope instead of
        // checking whether any variables are exposed to outside (e.g. `export` in ESM)
        if decl.kind.is_const() && ctx.current_scope_id() != ctx.scopes().root_scope_id() {
            let all_declarations_are_only_read =
                decl.declarations.iter().flat_map(|d| d.id.get_binding_identifiers()).all(|id| {
                    ctx.symbols()
                        .get_resolved_references(id.symbol_id())
                        .all(|reference| reference.flags().is_read_only())
                });
            if all_declarations_are_only_read {
                decl.kind = VariableDeclarationKind::Let;
            }
            for decl in &mut decl.declarations {
                decl.kind = VariableDeclarationKind::Let;
            }
        }
    }

    /// Transforms `undefined` => `void 0`, `Infinity` => `f64::Infinity`, `NaN` -> `f64::NaN`.
    /// So subsequent passes don't need to look up whether these variables are shadowed or not.
    fn try_compress_identifier(
        ident: &IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match ident.name.as_str() {
            "undefined" if ident.is_global_reference(ctx.symbols()) => {
                // `delete undefined` returns `false`
                // `delete void 0` returns `true`
                if matches!(ctx.parent(), Ancestor::UnaryExpressionArgument(e) if e.operator().is_delete())
                {
                    return None;
                }
                Some(ctx.ast.void_0(ident.span))
            }
            "Infinity" if ident.is_global_reference(ctx.symbols()) => {
                Some(ctx.ast.expression_numeric_literal(
                    ident.span,
                    f64::INFINITY,
                    None,
                    NumberBase::Decimal,
                ))
            }
            "NaN" if ident.is_global_reference(ctx.symbols()) => Some(
                ctx.ast.expression_numeric_literal(ident.span, f64::NAN, None, NumberBase::Decimal),
            ),
            _ => None,
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
    use crate::tester::{test, test_same};

    #[test]
    fn test_while() {
        // Verify while loops are converted to FOR loops.
        test("while(c < b) foo()", "for(; c < b;) foo()");
    }

    #[test]
    fn test_const_to_let() {
        test_same("const x = 1"); // keep top-level (can be replaced with "let" if it's ESM and not exported)
        test("{ const x = 1 }", "{ let x = 1 }");
        test_same("{ const x = 1; x = 2 }"); // keep assign error
        test("{ const x = 1, y = 2 }", "{ let x = 1, y = 2 }");
        test("{ const { x } = { x: 1 } }", "{ let { x } = { x: 1 } }");
        test("{ const [x] = [1] }", "{ let [x] = [1] }");
        test("{ const [x = 1] = [] }", "{ let [x = 1] = [] }");
        test("for (const x in y);", "for (let x in y);");
        // TypeError: Assignment to constant variable.
        test_same("for (const i = 0; i < 1; i++);");
        test_same("for (const x in [1, 2, 3]) x++");
        test_same("for (const x of [1, 2, 3]) x++");
        test("{ let foo; const bar = undefined; }", "{ let foo, bar; }");
    }

    #[test]
    fn test_void_ident() {
        test("var x; void x", "var x");
        test("void x", "x"); // reference error
    }

    #[test]
    fn parens() {
        test("(((x)))", "x");
        test("(((a + b))) * c", "(a + b) * c");
    }

    #[test]
    fn drop_console() {
        test("console.log()", "");
        test("(() => console.log())()", "(() => void 0)()");
    }

    #[test]
    fn drop_debugger() {
        test("debugger", "");
    }
}
