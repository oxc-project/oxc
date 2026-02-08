use crate::generated::ancestor::Ancestor;
use oxc_allocator::{TakeIn, Vec};
use oxc_ast::ast::*;
use oxc_ecmascript::{
    constant_evaluation::{DetermineValueType, ValueType},
    side_effects::is_valid_regexp,
};
use oxc_semantic::IsGlobalReference;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;

use crate::{ReusableTraverseCtx, Traverse, TraverseCtx, minifier_traverse::traverse_mut_with_ctx};

#[derive(Default)]
pub struct NormalizeOptions {
    pub convert_while_to_fors: bool,
    pub convert_const_to_let: bool,
    pub remove_unnecessary_use_strict: bool,
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
    pub fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for Normalize {
    fn exit_program(&mut self, node: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.options.remove_unnecessary_use_strict && node.source_type.is_module() {
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
        if self.options.remove_unnecessary_use_strict {
            Self::remove_unused_use_strict_directive(body, ctx);
        }
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
        if decl.kind.is_const()
            && ctx.current_scope_id() != ctx.scoping().root_scope_id()
            // direct eval may have a assignment inside
            && !ctx.current_scope_flags().contains_direct_eval()
        {
            let all_declarations_are_only_read = decl.declarations.iter().all(|d| {
                d.id.all_binding_identifiers(&mut |id| {
                    ctx.scoping()
                        .get_resolved_references(id.symbol_id())
                        .all(|reference| reference.flags().is_read_only())
                })
            });
            if all_declarations_are_only_read {
                // mark all declarations as `let`
                decl.kind = VariableDeclarationKind::Let;
                for decl in &mut decl.declarations {
                    decl.kind = VariableDeclarationKind::Let;
                }
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
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let Expression::Identifier(ident) = &e.object else { return None };
        if ident.name != "Number" {
            return None;
        }
        if e.property.name != "NaN" {
            return None;
        }
        if !ctx.is_global_reference(ident) {
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
        ctx: &TraverseCtx<'a>,
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
            "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError" | "SyntaxError"
            | "TypeError" | "URIError" | "Number" | "Object" | "String" => (false, false, &[]),
            // RegExp needs special validation using the regex parser
            "RegExp" => {
                if Self::can_set_pure(ident, ctx) && is_valid_regexp(&new_expr.arguments) {
                    new_expr.pure = true;
                }
                return;
            }
            _ => return,
        };

        if !Self::can_set_pure(ident, ctx) {
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

    fn can_set_pure(ident: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
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
