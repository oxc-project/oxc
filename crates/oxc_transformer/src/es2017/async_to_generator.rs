//! ES2017: Async / Await \[WIP\]
//!
//! This plugin transforms async functions to generator functions.
//!
//! ## Example
//!
//! Input:
//! ```js
//! async function foo() {
//!   await bar();
//! }
//! const foo2 = async () => {
//!   await bar();
//! };
//! async () => {
//!   await bar();
//! }
//! ```
//!
//! Output (Currently):
//! ```js
//! function foo() {
//!   return _asyncToGenerator(function* () {
//!     yield bar();
//!   })
//! }
//! const foo2 = () => _asyncToGenerator(function* () {
//!   yield bar();
//! }
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-async-to-generator](https://babel.dev/docs/babel-plugin-transform-async-to-generator).
//!
//!
//! Reference:
//! * Babel docs: <https://babeljs.io/docs/en/babel-plugin-transform-async-to-generator>
//! * Esbuild implementation: <https://github.com/evanw/esbuild/blob/main/internal/js_parser/js_parser_lower.go#L392>
//! * Babel implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-async-to-generator>
//! * Babel helper implementation: <https://github.com/babel/babel/blob/main/packages/babel-helper-remap-async-to-generator>
//! * Async / Await TC39 proposal: <https://github.com/tc39/proposal-async-await>
//!

use oxc_ast::{
    ast::{
        ArrowFunctionExpression, Expression, Function, FunctionType, Statement,
        VariableDeclarationKind,
    },
    NONE,
};
use oxc_span::{Atom, SPAN};
use oxc_syntax::{reference::ReferenceFlags, symbol::SymbolId};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

pub struct AsyncToGenerator;

impl AsyncToGenerator {
    fn get_helper_callee<'a>(
        symbol_id: Option<SymbolId>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ident = ctx.create_reference_id(
            SPAN,
            Atom::from("babelHelpers"),
            symbol_id,
            ReferenceFlags::Read,
        );
        let object = ctx.ast.expression_from_identifier_reference(ident);
        let property = ctx.ast.identifier_name(SPAN, Atom::from("asyncToGenerator"));
        Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
    }

    fn transform_function<'a>(func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) -> Function<'a> {
        let babel_helpers_id = ctx.scopes().find_binding(ctx.current_scope_id(), "babelHelpers");
        let callee = Self::get_helper_callee(babel_helpers_id, ctx);
        let target = ctx.ast.function(
            func.r#type,
            SPAN,
            None,
            true,
            false,
            false,
            func.type_parameters.take(),
            func.this_param.take(),
            ctx.ast.alloc(ctx.ast.formal_parameters(
                SPAN,
                func.params.kind,
                ctx.ast.move_vec(&mut func.params.items),
                func.params.rest.take(),
            )),
            func.return_type.take(),
            func.body.take(),
        );
        let parameters =
            ctx.ast.vec1(ctx.ast.argument_expression(ctx.ast.expression_from_function(target)));
        let call = ctx.ast.expression_call(SPAN, callee, NONE, parameters, false);
        let returns = ctx.ast.return_statement(SPAN, Some(call));
        let body = Statement::ReturnStatement(ctx.ast.alloc(returns));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), ctx.ast.vec1(body));
        let body = ctx.ast.alloc(body);
        let params = ctx.ast.formal_parameters(SPAN, func.params.kind, ctx.ast.vec(), NONE);
        ctx.ast.function(
            FunctionType::FunctionExpression,
            SPAN,
            None,
            false,
            false,
            false,
            func.type_parameters.take(),
            func.this_param.take(),
            params,
            func.return_type.take(),
            Some(body),
        )
    }
}

impl<'a> Traverse<'a> for AsyncToGenerator {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::AwaitExpression(await_expr) = expr {
            // Do not transform top-level await, or in async generator functions.
            let in_async_function = ctx
                .ancestry
                .ancestors()
                .find_map(|ance| {
                    // We need to check if there's async generator or async function.
                    // If it is async generator, we should not transform the await expression here.
                    if let Ancestor::FunctionBody(body) = ance {
                        if *body.r#async() {
                            Some(!body.generator())
                        } else {
                            None
                        }
                    } else if let Ancestor::ArrowFunctionExpressionBody(_) = ance {
                        // Arrow function is never generator.
                        Some(true)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);
            if in_async_function {
                // Move the expression to yield.
                *expr = ctx.ast.expression_yield(
                    SPAN,
                    false,
                    Some(ctx.ast.move_expression(&mut await_expr.argument)),
                );
            }
        } else if let Expression::FunctionExpression(func) = expr {
            if !func.r#async || func.generator {
                return;
            }
            let new_function = Self::transform_function(func, ctx);
            *expr = ctx.ast.expression_from_function(new_function);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::FunctionDeclaration(func) = stmt {
            if !func.r#async || func.generator {
                return;
            }
            let new_function = Self::transform_function(func, ctx);
            if let Some(id) = func.id.take() {
                *stmt = ctx.ast.statement_declaration(ctx.ast.declaration_variable(
                    SPAN,
                    VariableDeclarationKind::Const,
                    ctx.ast.vec1(ctx.ast.variable_declarator(
                        SPAN,
                        VariableDeclarationKind::Const,
                        ctx.ast.binding_pattern(
                            ctx.ast.binding_pattern_kind_from_binding_identifier(id),
                            NONE,
                            false,
                        ),
                        Some(ctx.ast.expression_from_function(new_function)),
                        false,
                    )),
                    false,
                ));
            } else {
                *stmt =
                    ctx.ast.statement_declaration(ctx.ast.declaration_from_function(new_function));
            }
        }
    }

    fn exit_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !arrow.r#async {
            return;
        }
        let babel_helpers_id = ctx.scopes().find_binding(ctx.current_scope_id(), "babelHelpers");
        let callee = Self::get_helper_callee(babel_helpers_id, ctx);
        let body = ctx.ast.function_body(
            SPAN,
            ctx.ast.move_vec(&mut arrow.body.directives),
            ctx.ast.move_vec(&mut arrow.body.statements),
        );
        let target = ctx.ast.function(
            FunctionType::FunctionExpression,
            SPAN,
            None,
            true,
            false,
            false,
            arrow.type_parameters.take(),
            NONE,
            ctx.ast.alloc(ctx.ast.formal_parameters(
                SPAN,
                arrow.params.kind,
                ctx.ast.move_vec(&mut arrow.params.items),
                arrow.params.rest.take(),
            )),
            arrow.return_type.take(),
            Some(body),
        );
        let parameters =
            ctx.ast.vec1(ctx.ast.argument_expression(ctx.ast.expression_from_function(target)));
        let call = ctx.ast.expression_call(SPAN, callee, NONE, parameters, false);
        let body = ctx.ast.function_body(
            SPAN,
            ctx.ast.vec(),
            ctx.ast.vec1(ctx.ast.statement_expression(SPAN, call)),
        );
        arrow.body = ctx.ast.alloc(body);
        arrow.r#async = false;
        arrow.expression = true;
    }
}
