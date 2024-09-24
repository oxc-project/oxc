//! ES2017: Async / Await
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
//! ```
//!
//! Output:
//! ```js
//! function foo() {
//!   return _asyncToGenerator(this, null, function* () {
//!     yield bar();
//!   })
//! }
//! const foo2 = () => _asyncToGenerator(this, null, function* () {
//!   yield bar();
//! }
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-async-to-generator](https://babel.dev/docs/babel-plugin-transform-async-to-generator) and [esbuild](https://github.com/evanw/esbuild/blob/main/internal/js_parser/js_parser_lower.go#L392).
//!
//!
//! Reference:
//! * Babel docs: <https://babeljs.io/docs/en/babel-plugin-transform-async-to-generator>
//! * Esbuild implementation: <https://github.com/evanw/esbuild/blob/main/internal/js_parser/js_parser_lower.go#L392>
//! * Babel implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-async-to-generator>
//! * Babel helper implementation: <https://github.com/babel/babel/blob/main/packages/babel-helper-remap-async-to-generator>
//! * Async / Await TC39 proposal: <https://github.com/tc39/proposal-async-await>
//!

use crate::context::Ctx;
use oxc_allocator::CloneIn;
use oxc_ast::ast::{ArrowFunctionExpression, Expression, FormalParameterKind, Function, FunctionType, Statement, YieldExpression};
use oxc_ast::NONE;
use oxc_span::{Atom, SPAN};
use oxc_syntax::reference::ReferenceFlags;
use oxc_syntax::symbol::SymbolId;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

pub struct AsyncToGenerator<'a> {
    _ctx: Ctx<'a>,
}

impl<'a> AsyncToGenerator<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { _ctx: ctx }
    }

    fn get_helper_callee(symbol_id: Option<SymbolId>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
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
}

impl<'a> Traverse<'a> for AsyncToGenerator<'a> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::AwaitExpression(await_expr) = expr {
            // Do not transform top-level await, or in async generator functions.
            let in_async_function = ctx
                .ancestry
                .ancestors()
                .find_map(|ance| {
                    if let Ancestor::FunctionBody(body) = ance {
                        if *body.r#async() {
                            Some(!body.generator())
                        } else {
                            None
                        }
                    } else if let Ancestor::ArrowFunctionExpressionBody(body) = ance {
                        Some(true)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);
            if in_async_function {
                let yield_expression = YieldExpression {
                    span: SPAN,
                    delegate: false,
                    argument: Some(ctx.ast.move_expression(&mut await_expr.argument)),
                };
                let expression = ctx.ast.alloc(yield_expression);
                *expr = Expression::YieldExpression(expression);
            }
        }
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        let babel_helpers_id = ctx.scopes().find_binding(ctx.current_scope_id(), "babelHelpers");
        let callee = Self::get_helper_callee(babel_helpers_id, ctx);
        let target = ctx.ast.function(
            func.r#type.clone(),
            SPAN,
            func.id.clone(),
            true,
            false,
            false,
            func.type_parameters.take(),
            func.this_param.take(),
            ctx.ast.alloc(ctx.ast.formal_parameters(
                SPAN,
                FormalParameterKind::FormalParameter,
                ctx.ast.vec(),
                NONE,
            )),
            func.return_type.take(),
            func.body.take()
        );
        let parameters = {
            let mut items = ctx.ast.vec();
            items.push(ctx.ast.argument_expression(ctx.ast.expression_this(SPAN)));
            items.push(ctx.ast.argument_expression(ctx.ast.expression_null_literal(SPAN)));
            items.push(ctx.ast.argument_expression(ctx.ast.expression_from_function(target)));
            items
        };
        let call = ctx.ast.expression_call(SPAN, callee, NONE, parameters, false);
        let returns = ctx.ast.return_statement(SPAN, Some(call));
        let body = Statement::ReturnStatement(ctx.ast.alloc(returns));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), ctx.ast.vec1(body));
        let body = ctx.ast.alloc(body);
        func.r#async = false;
        func.body = Some(body);
    }

    fn exit_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let babel_helpers_id = ctx.scopes().find_binding(ctx.current_scope_id(), "babelHelpers");
        let callee = Self::get_helper_callee(babel_helpers_id, ctx);
        let body = ctx.ast.function_body(SPAN, ctx.ast.move_vec(&mut arrow.body.directives), ctx.ast.move_vec(&mut arrow.body.statements));
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
                FormalParameterKind::FormalParameter,
                ctx.ast.vec(),
                NONE,
            )),
            arrow.return_type.take(),
            Some(body)
        );
        let parameters = {
            let mut items = ctx.ast.vec();
            items.push(ctx.ast.argument_expression(ctx.ast.expression_this(SPAN)));
            items.push(ctx.ast.argument_expression(ctx.ast.expression_null_literal(SPAN)));
            items.push(ctx.ast.argument_expression(ctx.ast.expression_from_function(target)));
            items
        };
        let call = ctx.ast.expression_call(SPAN, callee, NONE, parameters, false);
        let returns = ctx.ast.return_statement(SPAN, Some(call));
        let body = Statement::ReturnStatement(ctx.ast.alloc(returns));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), ctx.ast.vec1(body));
        arrow.body = ctx.ast.alloc(body);
        arrow.r#async = false;
    }
}
