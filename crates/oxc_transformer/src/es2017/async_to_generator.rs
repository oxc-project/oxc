//! ES2017: Async / Await
//!
//! This plugin transforms async functions to generator functions
//! and wraps them with `asyncToGenerator` helper function.
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
//! Output:
//! ```js
//! function foo() {
//!   return _foo.apply(this, arguments);
//! }
//! function _foo() {
//!   _foo = babelHelpers.asyncToGenerator(function* () {
//!           yield bar();
//!   });
//!   return _foo.apply(this, arguments);
//! }
//! const foo2 = function() {
//!   var _ref = babelHelpers.asyncToGenerator(function* () {
//!           yield bar();
//!   });
//!   return function foo2() {
//!      return _ref.apply(this, arguments);
//!   };
//! }();
//! babelHelpers.asyncToGenerator(function* () {
//!   yield bar();
//! });
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-async-to-generator](https://babel.dev/docs/babel-plugin-transform-async-to-generator).
//!
//! Reference:
//! * Babel docs: <https://babeljs.io/docs/en/babel-plugin-transform-async-to-generator>
//! * Babel implementation: <https://github.com/babel/babel/blob/v7.26.2/packages/babel-plugin-transform-async-to-generator>
//! * Async / Await TC39 proposal: <https://github.com/tc39/proposal-async-await>

use std::{borrow::Cow, mem};

use oxc_allocator::{ArenaBox, ArenaStringBuilder, ArenaVec, GetAllocator, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_ast_visit::Visit;
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{GetSpan, SPAN};
use oxc_str::{Ident, static_ident};
use oxc_syntax::{
    identifier::{is_identifier_name, is_identifier_part, is_identifier_start},
    keyword::is_reserved_keyword,
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse};

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    state::TransformState,
    utils::sync_function_symbol_flags,
};

pub struct AsyncToGenerator<'a> {
    executor: AsyncGeneratorExecutor<'a>,
}

impl AsyncToGenerator<'_> {
    pub fn new() -> Self {
        Self { executor: AsyncGeneratorExecutor::new(Helper::AsyncToGenerator) }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for AsyncToGenerator<'a> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let new_expr = match expr {
            Expression::AwaitExpression(await_expr) => {
                Self::transform_await_expression(await_expr, ctx)
            }
            Expression::FunctionExpression(func) => {
                if func.r#async && !func.generator && !func.is_typescript_syntax() {
                    Some(self.executor.transform_function_expression(func, ctx))
                } else {
                    None
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                if arrow.r#async {
                    Some(self.executor.transform_arrow_function(arrow, ctx))
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

        if let Some(function) = function
            && function.r#async
            && !function.generator
            && !function.is_typescript_syntax()
        {
            let new_statement = self.executor.transform_function_declaration(function, ctx);
            ctx.state.statement_injector.insert_after(stmt, new_statement);
        }
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if func.r#async
            && !func.is_typescript_syntax()
            && AsyncGeneratorExecutor::is_class_method_like_ancestor(ctx.parent())
        {
            self.executor.transform_function_for_method_definition(func, ctx);
        }
    }
}

impl<'a> AsyncToGenerator<'a> {
    /// Check whether the current node is inside an async function.
    fn is_inside_async_function(ctx: &TraverseCtx<'a>) -> bool {
        // Early return if current scope is top because we don't need to transform top-level await expression.
        if ctx.current_scope_flags().is_top() {
            return false;
        }

        for ancestor in ctx.ancestors() {
            match ancestor {
                Ancestor::FunctionBody(func) => return *func.r#async(),
                Ancestor::ArrowFunctionExpressionBody(func) => {
                    return *func.r#async();
                }
                _ => {}
            }
        }
        false
    }

    /// Transforms `await` expressions to `yield` expressions.
    /// Ignores top-level await expressions.
    fn transform_await_expression(
        expr: &mut AwaitExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        // We don't need to handle top-level await.
        if Self::is_inside_async_function(ctx) {
            Some(Expression::new_yield_expression(
                expr.span,
                false,
                Some(expr.argument.take_in(ctx)),
                ctx,
            ))
        } else {
            None
        }
    }
}

pub struct AsyncGeneratorExecutor<'a> {
    helper: Helper,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> AsyncGeneratorExecutor<'a> {
    pub fn new(helper: Helper) -> Self {
        Self { helper, _marker: std::marker::PhantomData }
    }

    /// Transforms async method definitions to generator functions wrapped in asyncToGenerator.
    ///
    /// ## Example
    ///
    /// Input:
    /// ```js
    /// class A { async foo() { await bar(); } }
    /// ```
    ///
    /// Output:
    /// ```js
    /// class A {
    /// foo() {
    ///     return babelHelpers.asyncToGenerator(function* () {
    ///         yield bar();
    ///     })();
    /// }
    /// ```
    pub fn transform_function_for_method_definition(
        &self,
        func: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(body) = func.body.take() else {
            return;
        };

        // If parameters could throw errors, we need to move them to the inner function,
        // because it is an async function, which should return a rejecting promise if
        // there is an error.
        let needs_move_parameters_to_inner_function =
            Self::could_throw_errors_parameters(&func.params);

        let (generator_scope_id, wrapper_scope_id) = {
            let new_scope_id = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            let scope_id = func.scope_id.replace(Some(new_scope_id)).unwrap();
            // We need to change the parent id to new scope id because we need to this function's body inside the wrapper function,
            // and then the new scope id will be wrapper function's scope id.
            ctx.scoping_mut().change_scope_parent_id(scope_id, Some(new_scope_id));
            if !needs_move_parameters_to_inner_function {
                // We need to change formal parameters's scope back to the original scope,
                // because we only move out the function body.
                Self::move_formal_parameters_to_target_scope(new_scope_id, &func.params, ctx);
            }

            (scope_id, new_scope_id)
        };

        let params = if needs_move_parameters_to_inner_function {
            // Make sure to not change the value of the "length" property. This is
            // done by generating dummy arguments for the outer function equal to
            // the expected length of the function:
            //
            //   async function foo(a, b, c = d, ...e) {
            //   }
            //
            // This turns into:
            //
            //   function foo(_x, _x1) {
            //     return _asyncToGenerator(function* (a, b, c = d, ...e) {
            //     }).call(this, arguments);
            //   }
            //
            // The "_x" and "_x1" are dummy variables to ensure "foo.length" is 2.
            let new_params = Self::create_placeholder_params(&func.params, wrapper_scope_id, ctx);
            mem::replace(&mut func.params, new_params)
        } else {
            Self::create_empty_params(ctx)
        };

        let callee = self.create_async_to_generator_call(params, body, generator_scope_id, ctx);
        let (callee, arguments) = if needs_move_parameters_to_inner_function {
            // callee.apply(this, arguments)
            let property = IdentifierName::new(SPAN, "apply", ctx);
            let callee =
                Expression::new_static_member_expression(SPAN, callee, property, false, ctx);

            // this, arguments
            let this_argument = Argument::new_this_expression(SPAN, ctx);
            let arguments_argument = Argument::from(ctx.create_unbound_ident_expr(
                SPAN,
                static_ident!("arguments"),
                ReferenceFlags::Read,
            ));
            (callee, ArenaVec::from_array_in([this_argument, arguments_argument], ctx))
        } else {
            // callee()
            (callee, ArenaVec::new_in(ctx))
        };

        let expression = Expression::new_call_expression(SPAN, callee, NONE, arguments, false, ctx);
        let statement = Statement::new_return_statement(SPAN, Some(expression), ctx);

        // Modify the wrapper function
        func.r#async = false;
        func.generator = false;
        func.body = Some(FunctionBody::boxed(
            SPAN,
            ArenaVec::new_in(ctx),
            ArenaVec::from_value_in(statement, ctx),
            ctx,
        ));
        func.scope_id.set(Some(wrapper_scope_id));
        sync_function_symbol_flags(func, ctx);
    }

    /// Transforms [`Function`] whose type is [`FunctionType::FunctionExpression`] to a generator function
    /// and wraps it in asyncToGenerator helper function.
    pub fn transform_function_expression(
        &self,
        wrapper_function: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let span = wrapper_function.span;
        let body = wrapper_function.body.take().unwrap();
        let params = wrapper_function.params.take_in_box(ctx);
        let id = wrapper_function.id.take();
        let has_function_id = id.is_some();

        if !has_function_id && !Self::is_function_length_affected(&params) {
            return self.create_async_to_generator_call(
                params,
                body,
                wrapper_function.scope_id.take().unwrap(),
                ctx,
            );
        }

        let (generator_scope_id, wrapper_scope_id) = {
            let wrapper_scope_id =
                ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            let scope_id = wrapper_function.scope_id.replace(Some(wrapper_scope_id)).unwrap();
            // Change the parent scope of the function scope with the current scope.
            ctx.scoping_mut().change_scope_parent_id(scope_id, Some(wrapper_scope_id));
            // If there is an id, then we will use it as the name of caller_function,
            // and the caller_function is inside the wrapper function.
            // so we need to move the id to the new scope.
            if let Some(id) = id.as_ref() {
                Self::move_binding_identifier_to_target_scope(wrapper_scope_id, id, ctx);
                let symbol_id = id.symbol_id();
                *ctx.scoping_mut().symbol_flags_mut(symbol_id) = SymbolFlags::Function;
            }
            (scope_id, wrapper_scope_id)
        };

        let bound_ident = Self::create_bound_identifier(
            id.as_ref(),
            wrapper_scope_id,
            SymbolFlags::FunctionScopedVariable,
            ctx,
        );

        let caller_function = {
            let scope_id = ctx.create_child_scope(wrapper_scope_id, ScopeFlags::Function);
            let params = Self::create_placeholder_params(&params, scope_id, ctx);
            let statements =
                ArenaVec::from_value_in(Self::create_apply_call_statement(&bound_ident, ctx), ctx);
            let body = FunctionBody::boxed(SPAN, ArenaVec::new_in(ctx), statements, ctx);
            let (r#type, id) = if id.is_some() {
                // Caller is emitted as a function declaration inside the wrapper; its binding
                // was already moved to `wrapper_scope_id` above.
                (FunctionType::FunctionDeclaration, id)
            } else {
                // Caller is emitted as a named function expression; per the JS spec, its name
                // binds only inside the function itself — place the inferred id's binding in
                // the caller's own scope, not the wrapper scope.
                (
                    FunctionType::FunctionExpression,
                    Self::infer_function_id_from_parent_node(scope_id, ctx),
                )
            };
            Self::create_function(r#type, id, params, body, scope_id, ctx)
        };

        {
            // Modify the wrapper function to add new body, params, and scope_id.
            let async_to_gen_decl = self.create_async_to_generator_declaration(
                &bound_ident,
                params,
                body,
                generator_scope_id,
                ctx,
            );
            let statements = if has_function_id {
                let id = caller_function.id.as_ref().unwrap();
                // If the function has an id, then we need to return the id.
                // `function foo() { ... }` -> `function foo() {} return foo;`
                let reference = ctx.create_bound_ident_expr(
                    SPAN,
                    id.name,
                    id.symbol_id(),
                    ReferenceFlags::Read,
                );
                let func_decl = Statement::FunctionDeclaration(caller_function);
                let statement_return = Statement::new_return_statement(SPAN, Some(reference), ctx);
                ArenaVec::from_array_in([async_to_gen_decl, func_decl, statement_return], ctx)
            } else {
                // If the function doesn't have an id, then we need to return the function itself.
                // `function() { ... }` -> `return function() { ... };`
                let statement_return = Statement::new_return_statement(
                    SPAN,
                    Some(Expression::FunctionExpression(caller_function)),
                    ctx,
                );
                ArenaVec::from_array_in([async_to_gen_decl, statement_return], ctx)
            };
            debug_assert!(wrapper_function.body.is_none());
            wrapper_function.r#async = false;
            wrapper_function.generator = false;
            wrapper_function.body.replace(FunctionBody::boxed(
                SPAN,
                ArenaVec::new_in(ctx),
                statements,
                ctx,
            ));
        }

        // Construct the IIFE
        let callee = Expression::FunctionExpression(wrapper_function.take_in_box(ctx));
        Expression::new_call_expression_with_pure(
            span,
            callee,
            NONE,
            ArenaVec::new_in(ctx),
            false,
            true,
            ctx,
        )
    }

    /// Transforms async function declarations into generator functions wrapped in the asyncToGenerator helper.
    pub fn transform_function_declaration(
        &self,
        wrapper_function: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let (generator_scope_id, wrapper_scope_id) = {
            let wrapper_scope_id =
                ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            let scope_id = wrapper_function.scope_id.replace(Some(wrapper_scope_id)).unwrap();
            // Change the parent scope of the function scope with the current scope.
            ctx.scoping_mut().change_scope_parent_id(scope_id, Some(wrapper_scope_id));
            (scope_id, wrapper_scope_id)
        };
        let body = wrapper_function.body.take().unwrap();
        let params =
            Self::create_placeholder_params(&wrapper_function.params, wrapper_scope_id, ctx);
        let params = mem::replace(&mut wrapper_function.params, params);

        let function_binding_scope_id =
            Self::function_declaration_binding_scope_id(wrapper_function.id.as_ref(), ctx);
        if function_binding_scope_id != ctx.current_scope_id()
            && let Some(id) = &wrapper_function.id
        {
            Self::move_binding_identifier_to_target_scope(function_binding_scope_id, id, ctx);
        }

        let bound_ident = Self::create_bound_identifier(
            wrapper_function.id.as_ref(),
            Self::function_declaration_binding_scope_id(None, ctx),
            SymbolFlags::Function,
            ctx,
        );

        // Modify the wrapper function
        {
            wrapper_function.r#async = false;
            wrapper_function.generator = false;
            sync_function_symbol_flags(wrapper_function, ctx);
            let statements =
                ArenaVec::from_value_in(Self::create_apply_call_statement(&bound_ident, ctx), ctx);
            debug_assert!(wrapper_function.body.is_none());
            wrapper_function.body.replace(FunctionBody::boxed(
                SPAN,
                ArenaVec::new_in(ctx),
                statements,
                ctx,
            ));
        }

        // function _name() { _ref.apply(this, arguments); }
        {
            let statements = ArenaVec::from_array_in(
                [
                    self.create_async_to_generator_assignment(
                        &bound_ident,
                        params,
                        body,
                        generator_scope_id,
                        ctx,
                    ),
                    Self::create_apply_call_statement(&bound_ident, ctx),
                ],
                ctx,
            );
            let body = FunctionBody::boxed(SPAN, ArenaVec::new_in(ctx), statements, ctx);

            let scope_id = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            // The generator function will move to this function, so we need
            // to change the parent scope of the generator function to the scope of this function.
            ctx.scoping_mut().change_scope_parent_id(generator_scope_id, Some(scope_id));

            let params = Self::create_empty_params(ctx);
            let id = Some(bound_ident.create_binding_identifier(ctx));
            let caller_function = Self::create_function(
                FunctionType::FunctionDeclaration,
                id,
                params,
                body,
                scope_id,
                ctx,
            );
            Statement::FunctionDeclaration(caller_function)
        }
    }

    /// Returns the binding scope a rebuilt semantic tree would use for a plain function
    /// declaration emitted at the current position.
    fn function_declaration_binding_scope_id(
        id: Option<&BindingIdentifier<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> ScopeId {
        if ctx.state.source_type.is_typescript() {
            return ctx.current_scope_id();
        }

        let scope_flags = ctx.current_scope_flags();
        if scope_flags.is_var() || scope_flags.is_strict_mode() {
            return ctx.current_scope_id();
        }

        let hoist_scope_id = ctx.current_hoist_scope_id();
        if let Some(id) = id
            && ctx.scoping().scope_has_binding(hoist_scope_id, id.name)
        {
            ctx.current_scope_id()
        } else {
            hoist_scope_id
        }
    }

    /// Transforms async arrow functions into generator functions wrapped in the asyncToGenerator helper.
    pub(self) fn transform_arrow_function(
        &self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let arrow_span = arrow.span;
        let mut body = arrow.body.take_in_box(ctx);

        // If the arrow's expression is true, we need to wrap the only one expression with return statement.
        if arrow.expression {
            let statement = body.statements.first_mut().unwrap();
            let expression = match statement {
                Statement::ExpressionStatement(es) => es.expression.take_in(ctx),
                _ => unreachable!(),
            };
            *statement = Statement::new_return_statement(expression.span(), Some(expression), ctx);
        }

        let params = arrow.params.take_in_box(ctx);
        let generator_function_id = arrow.scope_id();
        ctx.scoping_mut().scope_flags_mut(generator_function_id).remove(ScopeFlags::Arrow);
        let function_name = Self::infer_function_name_from_parent_node(ctx);

        if function_name.is_none() && !Self::is_function_length_affected(&params) {
            return self.create_async_to_generator_call(params, body, generator_function_id, ctx);
        }

        let wrapper_scope_id = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);

        // The generator function will move to inside wrapper, so we need
        // to change the parent scope of the generator function to the wrapper function.
        ctx.scoping_mut().change_scope_parent_id(generator_function_id, Some(wrapper_scope_id));

        let bound_ident = Self::create_bound_identifier(
            None,
            wrapper_scope_id,
            SymbolFlags::FunctionScopedVariable,
            ctx,
        );

        let caller_function = {
            let scope_id = ctx.create_child_scope(wrapper_scope_id, ScopeFlags::Function);
            let params = Self::create_placeholder_params(&params, scope_id, ctx);
            let statements =
                ArenaVec::from_value_in(Self::create_apply_call_statement(&bound_ident, ctx), ctx);
            let body = FunctionBody::boxed(SPAN, ArenaVec::new_in(ctx), statements, ctx);
            let id = function_name.map(|name| {
                ctx.generate_binding(name, scope_id, SymbolFlags::Function)
                    .create_binding_identifier(ctx)
            });
            let function = Self::create_function(
                FunctionType::FunctionExpression,
                id,
                params,
                body,
                scope_id,
                ctx,
            );
            let argument = Some(Expression::FunctionExpression(function));
            Statement::new_return_statement(SPAN, argument, ctx)
        };

        // Wrapper function
        {
            let statement = self.create_async_to_generator_declaration(
                &bound_ident,
                params,
                body,
                generator_function_id,
                ctx,
            );
            let statements = ArenaVec::from_array_in([statement, caller_function], ctx);
            let body = FunctionBody::boxed(SPAN, ArenaVec::new_in(ctx), statements, ctx);
            let params = Self::create_empty_params(ctx);
            let wrapper_function = Self::create_function(
                FunctionType::FunctionExpression,
                None,
                params,
                body,
                wrapper_scope_id,
                ctx,
            );
            // Construct the IIFE
            let callee = Expression::FunctionExpression(wrapper_function);
            Expression::new_call_expression(
                arrow_span,
                callee,
                NONE,
                ArenaVec::new_in(ctx),
                false,
                ctx,
            )
        }
    }

    /// Infers the function id from [`TraverseCtx::parent`].
    fn infer_function_id_from_parent_node(
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<BindingIdentifier<'a>> {
        let name = Self::infer_function_name_from_parent_node(ctx)?;
        Some(
            ctx.generate_binding(name, scope_id, SymbolFlags::Function)
                .create_binding_identifier(ctx),
        )
    }

    /// Infers the function name from the [`TraverseCtx::parent`].
    fn infer_function_name_from_parent_node(ctx: &TraverseCtx<'a>) -> Option<Ident<'a>> {
        match ctx.parent() {
            // infer `foo` from `const foo = async function() {}`
            Ancestor::VariableDeclaratorInit(declarator) => {
                declarator.id().get_binding_identifier().map(|id| id.name)
            }
            // infer `foo` from `({ foo: async function() {} })`
            Ancestor::ObjectPropertyValue(property) if !*property.method() => {
                property.key().static_name().map(|key| Self::normalize_function_name(&key, ctx))
            }
            _ => None,
        }
    }

    /// Normalizes the function name.
    ///
    /// Examples:
    ///
    /// // Valid
    /// * `foo` -> `foo`
    ///   // Contains space
    /// * `foo bar` -> `foo_bar`
    ///   // Reserved keyword
    /// * `this` -> `_this`
    /// * `arguments` -> `_arguments`
    fn normalize_function_name(input: &Cow<'a, str>, ctx: &TraverseCtx<'a>) -> Ident<'a> {
        let input_str = input.as_ref();
        if !is_reserved_keyword(input_str) && is_identifier_name(input_str) {
            return Ident::from_cow_in(input, ctx);
        }

        let mut name = ArenaStringBuilder::with_capacity_in(input_str.len() + 1, ctx.allocator());
        let mut capitalize_next = false;

        let mut chars = input_str.chars();
        if let Some(first) = chars.next()
            && is_identifier_start(first)
        {
            name.push(first);
        }

        for c in chars {
            if c == ' ' {
                name.push('_');
            } else if !is_identifier_part(c) {
                capitalize_next = true;
            } else if capitalize_next {
                name.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                name.push(c);
            }
        }

        if name.is_empty() {
            return static_ident!("_");
        }

        if is_reserved_keyword(name.as_str()) {
            name.push_ascii_byte_start(b'_');
        }

        Ident::from(name)
    }

    /// Creates a [`Function`] with the specified params, body and scope_id.
    #[inline]
    fn create_function(
        r#type: FunctionType,
        id: Option<BindingIdentifier<'a>>,
        params: ArenaBox<'a, FormalParameters<'a>>,
        body: ArenaBox<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaBox<'a, Function<'a>> {
        let function = Function::boxed_with_scope_id(
            SPAN,
            r#type,
            id,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
            scope_id,
            ctx,
        );
        sync_function_symbol_flags(&function, ctx);
        function
    }

    /// Creates a [`Statement`] that calls the `apply` method on the bound identifier.
    ///
    /// The generated code structure is:
    /// ```js
    /// bound_ident.apply(this, arguments);
    /// ```
    fn create_apply_call_statement(
        bound_ident: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let arguments = static_ident!("arguments");
        let symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), arguments);
        let arguments_ident =
            Argument::from(ctx.create_ident_expr(SPAN, arguments, symbol_id, ReferenceFlags::Read));

        // (this, arguments)
        let this = Argument::new_this_expression(SPAN, ctx);
        let arguments = ArenaVec::from_array_in([this, arguments_ident], ctx);
        // _ref.apply
        let callee = Expression::new_static_member_expression(
            SPAN,
            bound_ident.create_read_expression(ctx),
            IdentifierName::new(SPAN, "apply", ctx),
            false,
            ctx,
        );
        let argument = Expression::new_call_expression(SPAN, callee, NONE, arguments, false, ctx);
        Statement::new_return_statement(SPAN, Some(argument), ctx)
    }

    /// Creates an [`Expression`] that calls the [`AsyncGeneratorExecutor::helper`] helper function.
    ///
    /// This function constructs the helper call with arguments derived from the provided
    /// parameters, body, and scope_id.
    ///
    /// The generated code structure is:
    /// ```js
    /// asyncToGenerator(function* (PARAMS) {
    ///    BODY
    /// });
    /// ```
    fn create_async_to_generator_call(
        &self,
        params: ArenaBox<'a, FormalParameters<'a>>,
        body: ArenaBox<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut function = Self::create_function(
            FunctionType::FunctionExpression,
            None,
            params,
            body,
            scope_id,
            ctx,
        );
        function.generator = true;
        let arguments = ArenaVec::from_value_in(Argument::FunctionExpression(function), ctx);
        helper_call_expr(self.helper, arguments, ctx)
    }

    /// Creates a helper declaration statement for async-to-generator transformation.
    ///
    /// This function generates code that looks like:
    /// ```js
    /// var _ref = asyncToGenerator(function* (PARAMS) {
    ///   BODY
    /// });
    /// ```
    fn create_async_to_generator_declaration(
        &self,
        bound_ident: &BoundIdentifier<'a>,
        params: ArenaBox<'a, FormalParameters<'a>>,
        body: ArenaBox<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let init = self.create_async_to_generator_call(params, body, scope_id, ctx);
        let declarations = ArenaVec::from_value_in(
            VariableDeclarator::new(
                SPAN,
                VariableDeclarationKind::Var,
                bound_ident.create_binding_pattern(ctx),
                NONE,
                Some(init),
                false,
                ctx,
            ),
            ctx,
        );
        Statement::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            declarations,
            false,
            ctx,
        )
    }

    /// Creates a helper assignment statement for async-to-generator transformation.
    ///
    /// This function generates code that looks like:
    /// ```js
    /// _ref = asyncToGenerator(function* (PARAMS) {
    ///   BODY
    /// });
    /// ```
    fn create_async_to_generator_assignment(
        &self,
        bound: &BoundIdentifier<'a>,
        params: ArenaBox<'a, FormalParameters<'a>>,
        body: ArenaBox<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let right = self.create_async_to_generator_call(params, body, scope_id, ctx);
        let expression = Expression::new_assignment_expression(
            SPAN,
            AssignmentOperator::Assign,
            bound.create_write_target(ctx),
            right,
            ctx,
        );
        Statement::new_expression_statement(SPAN, expression, ctx)
    }

    /// Creates placeholder [`FormalParameters`] which named `_x` based on the passed-in parameters.
    /// `function p(x, y, z, d = 0, ...rest) {}` -> `function* (_x, _x1, _x2) {}`
    fn create_placeholder_params(
        params: &FormalParameters<'a>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaBox<'a, FormalParameters<'a>> {
        let mut parameters = ArenaVec::with_capacity_in(params.items.len(), ctx);
        for param in &params.items {
            if param.initializer.is_some() {
                break;
            }
            let binding = ctx.generate_uid("x", scope_id, SymbolFlags::FunctionScopedVariable);
            parameters.push(FormalParameter::new_plain(
                param.span(),
                binding.create_binding_pattern(ctx),
                ctx,
            ));
        }

        FormalParameters::boxed(SPAN, FormalParameterKind::FormalParameter, parameters, NONE, ctx)
    }

    /// Creates an empty [FormalParameters] with [FormalParameterKind::FormalParameter].
    #[inline]
    fn create_empty_params(ctx: &TraverseCtx<'a>) -> ArenaBox<'a, FormalParameters<'a>> {
        FormalParameters::boxed(
            SPAN,
            FormalParameterKind::FormalParameter,
            ArenaVec::new_in(ctx),
            NONE,
            ctx,
        )
    }

    /// Creates a [`BoundIdentifier`] for the id of the function.
    #[inline]
    fn create_bound_identifier(
        id: Option<&BindingIdentifier<'a>>,
        scope_id: ScopeId,
        flags: SymbolFlags,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        ctx.generate_uid(id.as_ref().map_or_else(|| "ref", |id| id.name.as_str()), scope_id, flags)
    }

    /// Check whether the given [`Ancestor`] is a class method-like node.
    pub(crate) fn is_class_method_like_ancestor(ancestor: Ancestor) -> bool {
        match ancestor {
            // `class A { async foo() {} }`
            Ancestor::MethodDefinitionValue(_) => true,
            // Only `({ async foo() {} })` does not include non-method like `({ foo: async function() {} })`,
            // because it's just a property with a function value
            Ancestor::ObjectPropertyValue(property) => *property.method(),
            _ => false,
        }
    }

    /// Checks if the function length is affected by the parameters.
    ///
    /// TODO: Needs to handle `ignoreFunctionLength` assumption.
    // <https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-helper-wrap-function/src/index.ts#L164>
    #[inline]
    fn is_function_length_affected(params: &FormalParameters<'_>) -> bool {
        params.items.first().is_some_and(|param| param.initializer.is_none())
    }

    /// Check whether the function parameters could throw errors.
    #[inline]
    fn could_throw_errors_parameters(params: &FormalParameters<'a>) -> bool {
        params.items.iter().any(|param| {
            param
                .initializer
                .as_ref()
                .is_some_and(|init| Self::could_potentially_throw_error_expression(init))
        })
    }

    /// Check whether the expression could potentially throw an error.
    #[inline]
    fn could_potentially_throw_error_expression(expr: &Expression<'a>) -> bool {
        !(matches!(
            expr,
            Expression::NullLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::NumericLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::BigIntLiteral(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::FunctionExpression(_)
        ) || expr.is_undefined())
    }

    #[inline]
    fn move_formal_parameters_to_target_scope(
        target_scope_id: ScopeId,
        params: &FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        BindingMover::new(target_scope_id, ctx).visit_formal_parameters(params);
    }

    #[inline]
    fn move_binding_identifier_to_target_scope(
        target_scope_id: ScopeId,
        ident: &BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        BindingMover::new(target_scope_id, ctx).visit_binding_identifier(ident);
    }
}

/// Moves the bindings from original scope to target scope.
struct BindingMover<'a, 'ctx> {
    ctx: &'ctx mut TraverseCtx<'a>,
    target_scope_id: ScopeId,
}

impl<'a, 'ctx> BindingMover<'a, 'ctx> {
    fn new(target_scope_id: ScopeId, ctx: &'ctx mut TraverseCtx<'a>) -> Self {
        Self { ctx, target_scope_id }
    }

    fn move_scope_to_target(&mut self, scope_id: ScopeId) {
        self.ctx.scoping_mut().change_scope_parent_id(scope_id, Some(self.target_scope_id));
    }
}

impl<'a> Visit<'a> for BindingMover<'a, '_> {
    fn visit_formal_parameter(&mut self, param: &FormalParameter<'a>) {
        // Move the parameter binding itself, then only reparent direct scopes from the initializer.
        // Initializer expressions can contain function/class bindings that must stay in their own
        // scopes, so they are not binding-moved.
        self.visit_binding_pattern(&param.pattern);
        if let Some(initializer) = &param.initializer {
            self.visit_expression(initializer);
        }
    }

    fn visit_formal_parameter_rest(&mut self, param: &FormalParameterRest<'a>) {
        // Rest parameters have no initializer, so the binding rest element is the only binding
        // position that needs to be moved.
        self.visit_binding_rest_element(&param.rest);
    }

    fn visit_assignment_pattern(&mut self, pattern: &AssignmentPattern<'a>) {
        // Move only the left-hand binding. The right-hand default is an expression, so only direct
        // scopes inside it are reparented; bindings declared inside those scopes stay there.
        self.visit_binding_pattern(&pattern.left);
        self.visit_expression(&pattern.right);
    }

    fn visit_binding_property(&mut self, property: &BindingProperty<'a>) {
        // Computed keys are expressions, not binding positions. They can still contain direct
        // scopes that moved with the parameter list.
        if property.computed {
            self.visit_property_key(&property.key);
        }
        self.visit_binding_pattern(&property.value);
    }

    #[inline]
    fn visit_function(&mut self, func: &Function<'a>, _flags: ScopeFlags) {
        self.move_scope_to_target(func.scope_id());
    }

    #[inline]
    fn visit_arrow_function_expression(&mut self, func: &ArrowFunctionExpression<'a>) {
        self.move_scope_to_target(func.scope_id());
    }

    #[inline]
    fn visit_class(&mut self, class: &Class<'a>) {
        // Decorators are evaluated outside the class scope and may contain direct scopes of their
        // own, so visit them before stopping at the class scope boundary.
        self.visit_decorators(&class.decorators);
        self.move_scope_to_target(class.scope_id());
    }

    /// Visits a binding identifier and moves it to the target scope.
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        let symbol_id = ident.symbol_id();
        let current_scope_id = self.ctx.scoping().symbol_scope_id(symbol_id);
        self.ctx.scoping_mut().move_binding_by_symbol_id(
            current_scope_id,
            self.target_scope_id,
            symbol_id,
        );
    }
}
