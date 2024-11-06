//! ES2018: Async Generator Functions
//!
//! This plugin mainly does the following transformations:
//!
//! 1. transforms async generator functions (async function *name() {}) to generator functions
//! and wraps them with `awaitAsyncGenerator` helper function.
//! 2. transforms `await expr` expression to `yield awaitAsyncGenerator(expr)`.
//! 3. transforms `yield * argument` expression to `yield asyncGeneratorDelegate(asyncIterator(argument))`.
//! 4. transforms `for await` statement to `for` statement, and inserts many code to handle async iteration.
//!
//! ## Example
//!
//! Input:
//! ```js
//! async function f() {
//!  for await (let x of y) {
//!    g(x);
//!  }
//!}
//! ```
//!
//! Output:
//! ```js
//! function f() {
//! return _f.apply(this, arguments);
//! }
//! function _f() {
//! _f = babelHelpers.asyncToGenerator(function* () {
//!     var _iteratorAbruptCompletion = false;
//!     var _didIteratorError = false;
//!     var _iteratorError;
//!     try {
//!     for (var _iterator = babelHelpers.asyncIterator(y), _step; _iteratorAbruptCompletion = !(_step = yield _iterator.next()).done; _iteratorAbruptCompletion = false) {
//!         let x = _step.value;
//!         {
//!         g(x);
//!         }
//!     }
//!     } catch (err) {
//!     _didIteratorError = true;
//!     _iteratorError = err;
//!     } finally {
//!     try {
//!         if (_iteratorAbruptCompletion && _iterator.return != null) {
//!         yield _iterator.return();
//!         }
//!     } finally {
//!         if (_didIteratorError) {
//!         throw _iteratorError;
//!         }
//!     }
//!     }
//! });
//! return _f.apply(this, arguments);
//! }
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-async-generator-functions](https://babel.dev/docs/babel-plugin-transform-async-generator-functions).
//!
//! Reference:
//! * Babel docs: <https://babeljs.io/docs/en/babel-plugin-transform-async-generator-functions>
//! * Babel implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-async-generator-functions>
//! * Async Iteration TC39 proposal: <https://github.com/tc39/proposal-async-iteration>

mod for_await;

use oxc_allocator::GetAddress;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::{common::helper_loader::Helper, context::TransformCtx, es2017::AsyncGeneratorExecutor};

pub struct AsyncGeneratorFunctions<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
    executor: AsyncGeneratorExecutor<'a, 'ctx>,
}

impl<'a, 'ctx> AsyncGeneratorFunctions<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx, executor: AsyncGeneratorExecutor::new(Helper::WrapAsyncGenerator, ctx) }
    }
}

impl<'a, 'ctx> Traverse<'a> for AsyncGeneratorFunctions<'a, 'ctx> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let new_expr = match expr {
            Expression::AwaitExpression(await_expr) => {
                self.transform_await_expression(await_expr, ctx)
            }
            Expression::YieldExpression(yield_expr) => {
                self.transform_yield_expression(yield_expr, ctx)
            }
            Expression::FunctionExpression(func) => {
                if func.r#async && func.generator {
                    Some(self.executor.transform_function_expression(func, ctx))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(new_expr) = new_expr {
            *expr = new_expr;
        }
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::ForOfStatement(for_of) = stmt {
            if !for_of.r#await {
                return;
            }

            // We need to replace the current statement with new statements,
            // but we don't have a such method to do it, so we leverage the statement injector.
            //
            // Now, we use below steps to workaround it:
            // 1. Use the last statement as the new statement.
            // 2. insert the rest of the statements before the current statement.
            // TODO: Once we have a method to replace the current statement, we can simplify this logic.
            let mut statements = self.transform_for_of_statement(for_of, ctx);
            let last_statement = statements.pop().unwrap();
            *stmt = last_statement;
            self.ctx.statement_injector.insert_many_before(&stmt.address(), statements);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let function = match stmt {
            Statement::FunctionDeclaration(func) => Some(func),
            Statement::ExportDefaultDeclaration(decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                    &mut decl.declaration
                {
                    Some(func)
                } else {
                    None
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &mut decl.declaration {
                    Some(func)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(function) = function {
            if function.r#async && function.generator && !function.is_typescript_syntax() {
                let new_statement = self.executor.transform_function_declaration(function, ctx);
                self.ctx.statement_injector.insert_after(stmt, new_statement);
            }
        }
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if func.r#async
            && func.generator
            && !func.is_typescript_syntax()
            && matches!(
                ctx.parent(),
                // `class A { async foo() {} }` | `({ async foo() {} })`
                Ancestor::MethodDefinitionValue(_) | Ancestor::ObjectPropertyValue(_)
            )
        {
            self.executor.transform_function_for_method_definition(func, ctx);
        }
    }
}

impl<'a, 'ctx> AsyncGeneratorFunctions<'a, 'ctx> {
    /// Check whether the current node is inside an async generator function.
    fn is_inside_async_generator_function(ctx: &mut TraverseCtx<'a>) -> bool {
        // Early return if current scope is top because we don't need to transform top-level await expression.
        if ctx.current_scope_flags().is_top() {
            return false;
        }

        for ancestor in ctx.ancestors() {
            match ancestor {
                Ancestor::FunctionBody(func) => return *func.r#async() && *func.generator(),
                Ancestor::ArrowFunctionExpressionBody(_) => {
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    /// Transform `yield * argument` expression to `yield asyncGeneratorDelegate(asyncIterator(argument))`.
    #[allow(clippy::unused_self)]
    fn transform_yield_expression(
        &self,
        expr: &mut YieldExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !expr.delegate {
            return None;
        }

        expr.argument.as_mut().map(|argument| {
            let argument = Argument::from(ctx.ast.move_expression(argument));
            let arguments = ctx.ast.vec1(argument);
            let mut argument = self.ctx.helper_call_expr(Helper::AsyncIterator, arguments, ctx);
            let arguments = ctx.ast.vec1(Argument::from(argument));
            argument = self.ctx.helper_call_expr(Helper::AsyncGeneratorDelegate, arguments, ctx);
            ctx.ast.expression_yield(SPAN, expr.delegate, Some(argument))
        })
    }

    /// Transforms `await expr` expression to `yield awaitAsyncGenerator(expr)`.
    /// Ignores top-level await expression.
    #[allow(clippy::unused_self)]
    fn transform_await_expression(
        &self,
        expr: &mut AwaitExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !Self::is_inside_async_generator_function(ctx) {
            return None;
        }

        let mut argument = ctx.ast.move_expression(&mut expr.argument);
        let arguments = ctx.ast.vec1(Argument::from(argument));
        argument = self.ctx.helper_call_expr(Helper::AwaitAsyncGenerator, arguments, ctx);

        Some(ctx.ast.expression_yield(SPAN, false, Some(argument)))
    }
}
