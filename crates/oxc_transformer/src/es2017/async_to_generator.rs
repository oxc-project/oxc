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
//! Reference:
//! * Babel docs: <https://babeljs.io/docs/en/babel-plugin-transform-async-to-generator>
//! * Esbuild implementation: <https://github.com/evanw/esbuild/blob/main/internal/js_parser/js_parser_lower.go#L392>
//! * Babel implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-async-to-generator>
//! * Babel helper implementation: <https://github.com/babel/babel/blob/main/packages/babel-helper-remap-async-to-generator>
//! * Async / Await TC39 proposal: <https://github.com/tc39/proposal-async-await>

use std::cell::Cell;

use oxc_allocator::Box;
use oxc_ast::{
    ast::{
        ArrowFunctionExpression, Expression, FormalParameters, Function, FunctionBody,
        FunctionType, Statement,
    },
    NONE,
};
use oxc_semantic::{ScopeFlags, ScopeId};
use oxc_span::{GetSpan, SPAN};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{common::helper_loader::Helper, TransformCtx};

pub struct AsyncToGenerator<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> AsyncToGenerator<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for AsyncToGenerator<'a, 'ctx> {
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
            if let Some(new_expr) = self.transform_function(func, ctx) {
                *expr = new_expr;
            }
        } else if let Expression::ArrowFunctionExpression(arrow) = expr {
            if !arrow.r#async {
                return;
            }
            *expr = self.transform_arrow_function(arrow, ctx);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, _ctx: &mut TraverseCtx<'a>) {
        if let Statement::FunctionDeclaration(_func) = stmt {
            // TODO: Waiting process
            // if !func.r#async || func.generator {
            //     return;
            // }
            // let new_function = self.transform_function(func, ctx);
            // if let Some(id) = func.id.take() {
            //     *stmt = ctx.ast.statement_declaration(ctx.ast.declaration_variable(
            //         SPAN,
            //         VariableDeclarationKind::Const,
            //         ctx.ast.vec1(ctx.ast.variable_declarator(
            //             SPAN,
            //             VariableDeclarationKind::Const,
            //             ctx.ast.binding_pattern(
            //                 ctx.ast.binding_pattern_kind_from_binding_identifier(id),
            //                 NONE,
            //                 false,
            //             ),
            //             Some(ctx.ast.expression_from_function(new_function)),
            //             false,
            //         )),
            //         false,
            //     ));
            // } else {
            //     *stmt =
            //         ctx.ast.statement_declaration(ctx.ast.declaration_from_function(new_function));
            // }
        }
    }
}

impl<'a, 'ctx> AsyncToGenerator<'a, 'ctx> {
    fn transform_function(
        &self,
        func: &mut Box<'a, Function<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if func.is_typescript_syntax() {
            return None;
        }

        if func.id.is_none() || !func.params.has_parameter() {
            let params = ctx.ast.move_formal_parameters(&mut func.params);
            let body = ctx.ast.move_function_body(func.body.as_mut().unwrap());
            return Some(self.wrap_helper_function(func.scope_id.get(), params, body, ctx));
        }

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
        let call = self.ctx.helper_call_expr(Helper::AsyncToGenerator, parameters, ctx);
        let returns = ctx.ast.return_statement(SPAN, Some(call));
        let body = Statement::ReturnStatement(ctx.ast.alloc(returns));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), ctx.ast.vec1(body));
        let body = ctx.ast.alloc(body);
        let params = ctx.ast.formal_parameters(SPAN, func.params.kind, ctx.ast.vec(), NONE);
        // TODO: Waiting process
        let _function = ctx.ast.function(
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
        );

        None
    }

    fn transform_arrow_function(
        &self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut body = ctx.ast.function_body(
            SPAN,
            ctx.ast.move_vec(&mut arrow.body.directives),
            ctx.ast.move_vec(&mut arrow.body.statements),
        );

        // If the arrow's expression is true, we need to wrap the only one expression with return statement.
        if arrow.expression {
            let statement = body.statements.first_mut().unwrap();
            let expression = match statement {
                Statement::ExpressionStatement(es) => ctx.ast.move_expression(&mut es.expression),
                _ => unreachable!(),
            };
            *statement = ctx.ast.statement_return(expression.span(), Some(expression));
        }

        let scope_id = arrow.scope_id.get();
        if let Some(scope_id) = scope_id {
            ctx.scopes_mut().get_flags_mut(scope_id).remove(ScopeFlags::Arrow);
        }

        self.wrap_helper_function(
            scope_id,
            ctx.ast.move_formal_parameters(&mut arrow.params),
            body,
            ctx,
        )
    }

    fn wrap_helper_function(
        &self,
        scope_id: Option<ScopeId>,
        parameters: FormalParameters<'a>,
        body: FunctionBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let r#type = FunctionType::FunctionExpression;
        let mut function = ctx.ast.function(
            r#type,
            SPAN,
            None,
            true,
            false,
            false,
            NONE,
            NONE,
            parameters,
            NONE,
            Some(body),
        );
        function.scope_id = Cell::new(scope_id);
        let arguments =
            ctx.ast.vec1(ctx.ast.argument_expression(ctx.ast.expression_from_function(function)));
        self.ctx.helper_call_expr(Helper::AsyncToGenerator, arguments, ctx)
    }
}
