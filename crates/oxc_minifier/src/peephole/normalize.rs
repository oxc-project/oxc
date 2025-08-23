use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{DetermineValueType, ValueType};
use oxc_semantic::IsGlobalReference;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::{Ancestor, ReusableTraverseCtx, Traverse, traverse_mut_with_ctx};

use crate::{
    ctx::{Ctx, TraverseCtx},
    state::MinifierState,
};

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
/// * convert `NaN` and `Number.NaN` to `f64::NaN`
/// * convert `var x; void x` to `void 0`
/// * convert `undefined` to `void 0`
/// * apply `pure` to side-effect free global constructors (e.g. `new WeakMap()`)
/// * remove unnecessary 'use strict' directive
///
/// Also
///
/// * remove `debugger` and `console.log` (optional)
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/Normalize.java>
pub struct Normalize {
    options: NormalizeOptions,
}

impl<'a> Normalize {
    pub fn build(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a, MinifierState<'a>>,
    ) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a, MinifierState<'a>> for Normalize {
    fn exit_program(&mut self, node: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        if node.source_type.is_module() {
            node.directives.drain_filter(|d| d.directive.as_str() == "use strict");
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        stmts.retain(|stmt| {
            !(matches!(stmt, Statement::EmptyStatement(_))
                || Self::drop_debugger(stmt, ctx)
                || Self::drop_console(stmt, ctx))
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
            *expr = paren_expr.expression.take_in(ctx.ast);
        }
        if let Some(e) = match expr {
            Expression::Identifier(ident) => Self::try_compress_identifier(ident, ctx),
            Expression::UnaryExpression(e) if e.operator.is_void() => {
                Self::fold_void_ident(e, ctx);
                None
            }
            Expression::ArrowFunctionExpression(e) => {
                Self::recover_arrow_expression_after_drop_console(e, ctx);
                None
            }
            Expression::CallExpression(_) if ctx.state.options.drop_console => {
                Self::compress_console(expr, ctx)
            }
            Expression::StaticMemberExpression(e) => Self::fold_number_nan_to_nan(e, ctx),
            _ => None,
        } {
            *expr = e;
        }
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::set_no_side_effects_to_call_expr(e, ctx);
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::set_pure_or_no_side_effects_to_new_expr(e, ctx);
    }

    fn exit_function_body(&mut self, body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::remove_unused_use_strict_directive(body, ctx);
    }
}

impl<'a> Normalize {
    pub fn new(options: NormalizeOptions) -> Self {
        Self { options }
    }

    /// Drop `drop_debugger` statement.
    ///
    /// Enabled by `compress.drop_debugger`
    fn drop_debugger(stmt: &Statement<'a>, ctx: &TraverseCtx<'a>) -> bool {
        matches!(stmt, Statement::DebuggerStatement(_)) && ctx.state.options.drop_debugger
    }

    fn compress_console(expr: &Expression<'a>, ctx: &TraverseCtx<'a>) -> Option<Expression<'a>> {
        debug_assert!(ctx.state.options.drop_console);
        Self::is_console(expr).then(|| ctx.ast.void_0(expr.span()))
    }

    fn drop_console(stmt: &Statement<'a>, ctx: &TraverseCtx<'a>) -> bool {
        ctx.state.options.drop_console
            && matches!(stmt, Statement::ExpressionStatement(expr) if Self::is_console(&expr.expression))
    }

    fn recover_arrow_expression_after_drop_console(
        expr: &mut ArrowFunctionExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if ctx.state.options.drop_console && expr.expression && expr.body.is_empty() {
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
        let Statement::WhileStatement(while_stmt) = stmt.take_in(ctx.ast) else { return };
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

    fn convert_const_to_let(decl: &mut VariableDeclaration<'a>, ctx: &TraverseCtx<'a>) {
        // checking whether the current scope is the root scope instead of
        // checking whether any variables are exposed to outside (e.g. `export` in ESM)
        if decl.kind.is_const() && ctx.current_scope_id() != ctx.scoping().root_scope_id() {
            let all_declarations_are_only_read =
                decl.declarations.iter().flat_map(|d| d.id.get_binding_identifiers()).all(|id| {
                    ctx.scoping()
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
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match ident.name.as_str() {
            "undefined" if ident.is_global_reference(ctx.scoping()) => {
                // `delete undefined` returns `false`
                // `delete void 0` returns `true`
                if Self::is_unary_delete_ancestor(ctx.ancestors()) {
                    return None;
                }
                Some(ctx.ast.void_0(ident.span))
            }
            "Infinity" if ident.is_global_reference(ctx.scoping()) => {
                // `delete Infinity` returns `false`
                // `delete 1/0` returns `true`
                if Self::is_unary_delete_ancestor(ctx.ancestors()) {
                    return None;
                }
                Some(ctx.ast.expression_numeric_literal(
                    ident.span,
                    f64::INFINITY,
                    None,
                    NumberBase::Decimal,
                ))
            }
            "NaN" if ident.is_global_reference(ctx.scoping()) => {
                // `delete NaN` returns `false`
                // `delete 0/0` returns `true`
                if Self::is_unary_delete_ancestor(ctx.ancestors()) {
                    return None;
                }
                Some(ctx.ast.nan(ident.span))
            }
            _ => None,
        }
    }

    fn is_unary_delete_ancestor<'t>(ancestors: impl Iterator<Item = Ancestor<'a, 't>>) -> bool {
        for ancestor in ancestors {
            match ancestor {
                Ancestor::UnaryExpressionArgument(e) if e.operator().is_delete() => {
                    return true;
                }
                Ancestor::ParenthesizedExpressionExpression(_)
                | Ancestor::SequenceExpressionExpressions(_) => {}
                _ => return false,
            }
        }
        false
    }

    fn fold_void_ident(e: &mut UnaryExpression<'a>, ctx: &TraverseCtx<'a>) {
        debug_assert!(e.operator.is_void());
        let Expression::Identifier(ident) = &e.argument else { return };
        if ident.is_global_reference(ctx.scoping()) {
            return;
        }
        e.argument = ctx.ast.expression_numeric_literal(ident.span, 0.0, None, NumberBase::Decimal);
    }

    fn fold_number_nan_to_nan(
        e: &StaticMemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let Expression::Identifier(ident) = &e.object else { return None };
        if ident.name != "Number" {
            return None;
        }
        if e.property.name != "NaN" {
            return None;
        }
        if !Ctx::new(ctx).is_global_reference(ident) {
            return None;
        }
        Some(ctx.ast.nan(ident.span))
    }

    fn set_no_side_effects_to_call_expr(call_expr: &mut CallExpression<'a>, ctx: &TraverseCtx<'a>) {
        if call_expr.pure {
            return;
        }
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };
        if let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() {
            // Apply `/* #__NO_SIDE_EFFECTS__ */`
            if ctx.scoping().no_side_effects().contains(&symbol_id) {
                call_expr.pure = true;
            }
        }
    }

    /// Set `pure` on side effect free `new Expr()`s.
    /// `PC` or `PC_WITH_ARRAY` in <https://github.com/rollup/rollup/blob/v4.42.0/src/ast/nodes/shared/knownGlobals.ts>
    fn set_pure_or_no_side_effects_to_new_expr(
        new_expr: &mut NewExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if new_expr.pure {
            return;
        }
        let Some(ident) = new_expr.callee.get_identifier_reference() else {
            return;
        };
        if let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() {
            // Apply `/* #__NO_SIDE_EFFECTS__ */`
            if ctx.scoping().no_side_effects().contains(&symbol_id) {
                new_expr.pure = true;
            }
            return;
        }
        // callee is a global reference.
        let ctx = Ctx::new(ctx);
        let len = new_expr.arguments.len();

        let (zero_arg_throws_error, one_arg_array_throws_error, one_arg_throws_error): (
            bool,
            bool,
            &'static [ValueType],
        ) = match ident.name.as_str() {
            "AggregateError" => (
                true,
                false,
                &[
                    ValueType::Undefined,
                    ValueType::Null,
                    ValueType::Number,
                    ValueType::BigInt,
                    ValueType::Boolean,
                    ValueType::Object,
                ],
            ),
            "DataView" => (
                true,
                true,
                &[
                    ValueType::Undefined,
                    ValueType::Null,
                    ValueType::Number,
                    ValueType::Boolean,
                    ValueType::BigInt,
                    ValueType::String,
                    ValueType::Boolean,
                    ValueType::Object,
                ],
            ),
            "Set" | "Map" | "WeakSet" | "WeakMap" => (
                false,
                false,
                &[
                    ValueType::Number,
                    ValueType::Boolean,
                    ValueType::BigInt,
                    ValueType::Boolean,
                    ValueType::Object,
                ],
            ),
            "ArrayBuffer" | "Date" => (false, false, &[ValueType::BigInt]),
            "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError" | "RegExp"
            | "SyntaxError" | "TypeError" | "URIError" | "Number" | "Object" | "String" => {
                (false, false, &[])
            }
            _ => return,
        };

        if !Self::can_set_pure(ident, &ctx) {
            return;
        }

        if match len {
            0 if !zero_arg_throws_error => true,
            1 => match new_expr.arguments[0].as_expression() {
                Some(Expression::ArrayExpression(array_expr)) => {
                    array_expr.elements.is_empty() && !one_arg_array_throws_error
                }
                Some(e) => {
                    if let Expression::NewExpression(new_expr) = e {
                        new_expr.pure
                    } else {
                        let value_type = e.value_type(&ctx);
                        if value_type.is_undetermined() {
                            false
                        } else {
                            !one_arg_throws_error.contains(&value_type)
                        }
                    }
                }
                _ => false,
            },
            _ => false,
        } {
            new_expr.pure = true;
        }
    }

    fn can_set_pure(ident: &IdentifierReference<'a>, ctx: &Ctx<'a, '_>) -> bool {
        ctx.is_global_reference(ident)
            // Throw is never pure.
            && !matches!(ctx.parent(), Ancestor::ThrowStatementArgument(_))
    }

    fn remove_unused_use_strict_directive(body: &mut FunctionBody<'a>, ctx: &TraverseCtx<'a>) {
        if !body.directives.is_empty()
            && ctx
                .scoping()
                .scope_parent_id(ctx.current_scope_id())
                .map(|scope_id| ctx.scoping().scope_flags(scope_id))
                .is_some_and(ScopeFlags::is_strict_mode)
        {
            body.directives.drain_filter(|d| d.directive.as_str() == "use strict");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        CompressOptions,
        tester::{default_options, test, test_options, test_options_source_type, test_same},
    };

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
        let options = CompressOptions { drop_console: true, ..default_options() };
        test_options("console.log()", "", &options);
        test_options("(() => console.log())()", "", &options);
        test_options(
            "(() => { try { return console.log() } catch {} })()",
            "(() => { try { return } catch {} })()",
            &options,
        );
    }

    #[test]
    fn drop_debugger() {
        test("debugger", "");
    }

    #[test]
    fn fold_number_nan() {
        test("foo(Number.NaN)", "foo(NaN)");
        test_same("var Number; foo(Number.NaN)");
        test_same("let Number; foo((void 0).NaN)");
    }

    #[test]
    fn pure_constructors() {
        test("new AggregateError", "AggregateError()");
        test("new ArrayBuffer", "");
        test("new Boolean", "");
        test("new DataView", "new DataView()");
        test("new Date", "");
        test("new Error", "");
        test("new EvalError", "");
        test("new Map", "");
        test("new Number", "");
        test("new Object", "");
        test("new RangeError", "");
        test("new ReferenceError", "");
        test("new RegExp", "");
        test("new Set", "");
        test("new String", "");
        test("new SyntaxError", "");
        test("new TypeError", "");
        test("new URIError", "");
        test("new WeakMap", "");
        test("new WeakSet", "");

        test("new AggregateError(null)", "AggregateError(null)");
        test("new ArrayBuffer(null)", "");
        test("new Boolean(null)", "");
        test_same("new DataView(null)");
        test("new Date(null)", "");
        test("new Error(null)", "");
        test("new EvalError(null)", "");
        test("new Map(null)", "");
        test("new Number(null)", "");
        test("new Object(null)", "");
        test("new RangeError(null)", "");
        test("new ReferenceError(null)", "");
        test("new RegExp(null)", "");
        test("new Set(null)", "");
        test("new String(null)", "");
        test("new SyntaxError(null)", "");
        test("new TypeError(null)", "");
        test("new URIError(null)", "");
        test("new WeakMap(null)", "");
        test("new WeakSet(null)", "");

        test("new AggregateError(undefined)", "AggregateError(void 0)");
        test("new ArrayBuffer(undefined)", "");
        test("new Boolean(undefined)", "");
        test_same("new DataView(void 0)");
        test("new Date(undefined)", "");
        test("new Error(undefined)", "");
        test("new EvalError(undefined)", "");
        test("new Map(undefined)", "");
        test("new Number(undefined)", "");
        test("new Object(undefined)", "");
        test("new RangeError(undefined)", "");
        test("new ReferenceError(undefined)", "");
        test("new RegExp(undefined)", "");
        test("new Set(undefined)", "");
        test("new String(undefined)", "");
        test("new SyntaxError(undefined)", "");
        test("new TypeError(undefined)", "");
        test("new URIError(undefined)", "");
        test("new WeakMap(undefined)", "");
        test("new WeakSet(undefined)", "");

        test("new AggregateError(0)", "AggregateError(0)");
        test("new ArrayBuffer(0)", "");
        test("new Boolean(0)", "");
        test_same("new DataView(0)");
        test("new Date(0)", "");
        test("new Error(0)", "");
        test("new EvalError(0)", "");
        test_same("new Map(0)");
        test("new Number(0)", "");
        test("new Object(0)", "");
        test("new RangeError(0)", "");
        test("new ReferenceError(0)", "");
        test("new RegExp(0)", "");
        test_same("new Set(0)");
        test("new String(0)", "");
        test("new SyntaxError(0)", "");
        test("new TypeError(0)", "");
        test("new URIError(0)", "");
        test_same("new WeakMap(0)");
        test_same("new WeakSet(0)");

        test("new AggregateError(10n)", "AggregateError(10n)");
        test_same("new ArrayBuffer(10n)");
        test("new Boolean(10n)", "");
        test_same("new DataView(10n)");
        test_same("new Date(10n)");
        test("new Error(10n)", "");
        test("new EvalError(10n)", "");
        test_same("new Map(10n)");
        test("new Number(10n)", "");
        test("new Object(10n)", "");
        test("new RangeError(10n)", "");
        test("new ReferenceError(10n)", "");
        test("new RegExp(10n)", "");
        test_same("new Set(10n)");
        test("new String(10n)", "");
        test("new SyntaxError(10n)", "");
        test("new TypeError(10n)", "");
        test("new URIError(10n)", "");
        test_same("new WeakMap(10n)");
        test_same("new WeakSet(10n)");

        test("new AggregateError('')", "");
        test("new ArrayBuffer('')", "");
        test("new Boolean('')", "");
        test_same("new DataView('')");
        test("new Date('')", "");
        test("new Error('')", "");
        test("new EvalError('')", "");
        test("new Map('')", "");
        test("new Number('')", "");
        test("new Object('')", "");
        test("new RangeError('')", "");
        test("new ReferenceError('')", "");
        test("new RegExp('')", "");
        test("new Set('')", "");
        test("new String('')", "");
        test("new SyntaxError('')", "");
        test("new TypeError('')", "");
        test("new URIError('')", "");
        test("new WeakMap('')", "");
        test("new WeakSet('')", "");

        test("new AggregateError(!0)", "AggregateError(!0)");
        test("new ArrayBuffer(!0)", "");
        test("new Boolean(!0)", "");
        test_same("new DataView(!0)");
        test("new Date(!0)", "");
        test("new Error(!0)", "");
        test("new EvalError(!0)", "");
        test_same("new Map(!0)");
        test("new Number(!0)", "");
        test("new Object(!0)", "");
        test("new RangeError(!0)", "");
        test("new ReferenceError(!0)", "");
        test("new RegExp(!0)", "");
        test_same("new Set(!0)");
        test("new String(!0)", "");
        test("new SyntaxError(!0)", "");
        test("new TypeError(!0)", "");
        test("new URIError(!0)", "");
        test_same("new WeakMap(!0)");
        test_same("new WeakSet(!0)");

        test("new AggregateError([])", "");
        test("new ArrayBuffer([])", "");
        test("new Boolean([])", "");
        test_same("new DataView([])");
        test("new Date([])", "");
        test("new Error([])", "");
        test("new EvalError([])", "");
        test("new Map([])", "");
        test("new Number([])", "");
        test("new Object([])", "");
        test("new RangeError([])", "");
        test("new ReferenceError([])", "");
        test("new RegExp([])", "");
        test("new Set([])", "");
        test("new String([])", "");
        test("new SyntaxError([])", "");
        test("new TypeError([])", "");
        test("new URIError([])", "");
        test("new WeakMap([])", "");
        test("new WeakSet([])", "");

        test("new AggregateError(a)", "AggregateError(a)");
        test_same("new ArrayBuffer(a)");
        test_same("new Boolean(a)");
        test_same("new DataView(a)");
        test_same("new Date(a)");
        test("new Error(a)", "Error(a)");
        test("new EvalError(a)", "EvalError(a)");
        test_same("new Map(a)");
        test_same("new Number(a)");
        test_same("new Object(a)");
        test("new RangeError(a)", "RangeError(a)");
        test("new ReferenceError(a)", "ReferenceError(a)");
        test_same("new RegExp(a)");
        test_same("new Set(a)");
        test_same("new String(a)");
        test("new SyntaxError(a)", "SyntaxError(a)");
        test("new TypeError(a)", "TypeError(a)");
        test("new URIError(a)", "URIError(a)");
        test_same("new WeakMap(a)");
        test_same("new WeakSet(a)");
    }

    #[test]
    fn remove_unused_use_strict_directive() {
        use oxc_span::SourceType;
        let options = default_options();
        let source_type = SourceType::cjs();
        test_options_source_type(
            "'use strict'; function _() { 'use strict' }",
            "'use strict'; function _() {  }",
            source_type,
            &options,
        );
        test_options_source_type(
            "function _() { 'use strict'; function __() { 'use strict' } }",
            "function _() { 'use strict'; function __() { } }",
            source_type,
            &options,
        );
        test("'use strict'; function _() { 'use strict' }", "function _() {}");
        test("'use strict';", "");
    }
}
