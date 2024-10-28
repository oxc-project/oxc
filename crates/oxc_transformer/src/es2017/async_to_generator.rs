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
//! * Babel implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-async-to-generator>
//! * Async / Await TC39 proposal: <https://github.com/tc39/proposal-async-await>

use std::mem;

use oxc_allocator::Box;
use oxc_ast::{ast::*, Visit, NONE};
use oxc_semantic::{ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
use oxc_span::{Atom, GetSpan, SPAN};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

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
        let new_expr = match expr {
            Expression::AwaitExpression(await_expr) => {
                self.transform_await_expression(await_expr, ctx)
            }
            Expression::FunctionExpression(func) => self.transform_function_expression(func, ctx),
            Expression::ArrowFunctionExpression(arrow) => self.transform_arrow_function(arrow, ctx),
            _ => None,
        };

        if let Some(new_expr) = new_expr {
            *expr = new_expr;
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let new_statement = match stmt {
            Statement::FunctionDeclaration(func) => self.transform_function_declaration(func, ctx),
            Statement::ExportDefaultDeclaration(decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func) =
                    &mut decl.declaration
                {
                    self.transform_function_declaration(func, ctx)
                } else {
                    None
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(Declaration::FunctionDeclaration(func)) = &mut decl.declaration {
                    self.transform_function_declaration(func, ctx)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(new_statement) = new_statement {
            self.ctx.statement_injector.insert_after(stmt, new_statement);
        }
    }

    fn exit_method_definition(
        &mut self,
        node: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.transform_function_for_method_definition(&mut node.value, ctx);
    }
}

impl<'a, 'ctx> AsyncToGenerator<'a, 'ctx> {
    /// Transforms `await` expressions to `yield` expressions.
    /// Ignores top-level await expressions.
    #[allow(clippy::unused_self)]
    fn transform_await_expression(
        &self,
        expr: &mut AwaitExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        // We don't need to handle top-level await.
        if ctx.parent().is_program() {
            None
        } else {
            Some(ctx.ast.expression_yield(
                SPAN,
                false,
                Some(ctx.ast.move_expression(&mut expr.argument)),
            ))
        }
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
    fn transform_function_for_method_definition(
        &self,
        func: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !func.r#async {
            return;
        }

        let Some(body) = func.body.take() else {
            return;
        };

        let (generator_scope_id, wrapper_scope_id) = {
            let new_scope_id = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            let scope_id = func.scope_id.replace(Some(new_scope_id)).unwrap();
            // We need to change the parent id to new scope id because we need to this function's body inside the wrapper function,
            // and then the new scope id will be wrapper function's scope id.
            ctx.scopes_mut().change_parent_id(scope_id, Some(new_scope_id));
            // We need to transform formal parameters change back to the original scope,
            // because we only move out the function body.
            BindingMover::new(new_scope_id, ctx).visit_formal_parameters(&func.params);

            (scope_id, new_scope_id)
        };

        let params = Self::create_empty_params(ctx);
        let expression = self.create_async_to_generator_call(params, body, generator_scope_id, ctx);
        // Construct the IIFE
        let expression = ctx.ast.expression_call(SPAN, expression, NONE, ctx.ast.vec(), false);
        let statement = ctx.ast.statement_return(SPAN, Some(expression));

        // Modify the wrapper function
        func.r#async = false;
        func.body = Some(ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), ctx.ast.vec1(statement)));
        func.scope_id.set(Some(wrapper_scope_id));
    }

    /// Transforms [`Function`] whose type is [`FunctionType::FunctionExpression`] to a generator function
    /// and wraps it in asyncToGenerator helper function.
    fn transform_function_expression(
        &self,
        wrapper_function: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !wrapper_function.r#async
            || wrapper_function.generator
            || wrapper_function.is_typescript_syntax()
        {
            return None;
        }

        let body = wrapper_function.body.take().unwrap();
        let params = ctx.alloc(ctx.ast.move_formal_parameters(&mut wrapper_function.params));
        let id = wrapper_function.id.take();
        let has_function_id = id.is_some();

        if !has_function_id && !Self::is_function_length_affected(&params) {
            return Some(self.create_async_to_generator_call(
                params,
                body,
                wrapper_function.scope_id.take().unwrap(),
                ctx,
            ));
        }

        let (generator_scope_id, wrapper_scope_id) = {
            let wrapper_scope_id =
                ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            let scope_id = wrapper_function.scope_id.replace(Some(wrapper_scope_id)).unwrap();
            // Change the parent scope of the function scope with the current scope.
            ctx.scopes_mut().change_parent_id(scope_id, Some(wrapper_scope_id));
            // If there is an id, then we will use it as the name of caller_function,
            // and the caller_function is inside the wrapper function.
            // so we need to move the id to the new scope.
            if let Some(id) = id.as_ref() {
                BindingMover::new(wrapper_scope_id, ctx).visit_binding_identifier(id);
                let symbol_id = id.symbol_id.get().unwrap();
                *ctx.symbols_mut().get_flags_mut(symbol_id) = SymbolFlags::FunctionScopedVariable;
            }
            (scope_id, wrapper_scope_id)
        };

        let bound_ident = Self::create_bound_identifier(id.as_ref(), wrapper_scope_id, ctx);

        let caller_function = {
            let scope_id = ctx.create_child_scope(wrapper_scope_id, ScopeFlags::Function);
            let params = Self::create_placeholder_params(&params, scope_id, ctx);
            let statements = ctx.ast.vec1(Self::create_apply_call_statement(&bound_ident, ctx));
            let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), statements);
            let id = id.or_else(|| {
                Self::infer_function_id_from_variable_declarator(wrapper_scope_id, ctx)
            });
            Self::create_function(id, params, body, scope_id, ctx)
        };

        {
            // Modify the wrapper function to add new body, params, and scope_id.
            let mut statements = ctx.ast.vec_with_capacity(3);
            let statement = self.create_async_to_generator_declaration(
                &bound_ident,
                params,
                body,
                generator_scope_id,
                ctx,
            );
            statements.push(statement);
            if has_function_id {
                let id = caller_function.id.as_ref().unwrap();
                // If the function has an id, then we need to return the id.
                // `function foo() { ... }` -> `function foo() {} return foo;`
                let reference = ctx.create_bound_reference_id(
                    SPAN,
                    id.name.clone(),
                    id.symbol_id.get().unwrap(),
                    ReferenceFlags::Read,
                );
                let statement = Statement::from(ctx.ast.declaration_from_function(caller_function));
                statements.push(statement);
                let argument = Some(ctx.ast.expression_from_identifier_reference(reference));
                statements.push(ctx.ast.statement_return(SPAN, argument));
            } else {
                // If the function doesn't have an id, then we need to return the function itself.
                // `function() { ... }` -> `return function() { ... };`
                let statement_return = ctx.ast.statement_return(
                    SPAN,
                    Some(ctx.ast.expression_from_function(caller_function)),
                );
                statements.push(statement_return);
            }
            debug_assert!(wrapper_function.body.is_none());
            wrapper_function.r#async = false;
            wrapper_function.body.replace(ctx.ast.alloc_function_body(
                SPAN,
                ctx.ast.vec(),
                statements,
            ));
        }

        // Construct the IIFE
        let callee = ctx.ast.expression_from_function(ctx.ast.move_function(wrapper_function));
        Some(ctx.ast.expression_call(SPAN, callee, NONE, ctx.ast.vec(), false))
    }

    /// Transforms async function declarations into generator functions wrapped in the asyncToGenerator helper.
    fn transform_function_declaration(
        &self,
        wrapper_function: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if !wrapper_function.r#async
            || wrapper_function.generator
            || wrapper_function.is_typescript_syntax()
        {
            return None;
        }

        let (generator_scope_id, wrapper_scope_id) = {
            let wrapper_scope_id =
                ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            let scope_id = wrapper_function.scope_id.replace(Some(wrapper_scope_id)).unwrap();
            // Change the parent scope of the function scope with the current scope.
            ctx.scopes_mut().change_parent_id(scope_id, Some(wrapper_scope_id));
            (scope_id, wrapper_scope_id)
        };
        let body = wrapper_function.body.take().unwrap();
        let params =
            Self::create_placeholder_params(&wrapper_function.params, wrapper_scope_id, ctx);
        let params = mem::replace(&mut wrapper_function.params, params);
        let bound_ident = Self::create_bound_identifier(
            wrapper_function.id.as_ref(),
            ctx.current_scope_id(),
            ctx,
        );

        // Modify the wrapper function
        {
            wrapper_function.r#async = false;
            let statements = ctx.ast.vec1(Self::create_apply_call_statement(&bound_ident, ctx));
            debug_assert!(wrapper_function.body.is_none());
            wrapper_function.body.replace(ctx.ast.alloc_function_body(
                SPAN,
                ctx.ast.vec(),
                statements,
            ));
        }

        // function _name() { _ref.apply(this, arguments); }
        {
            let mut statements = ctx.ast.vec_with_capacity(2);
            statements.push(self.create_async_to_generator_assignment(
                &bound_ident,
                params,
                body,
                generator_scope_id,
                ctx,
            ));
            statements.push(Self::create_apply_call_statement(&bound_ident, ctx));
            let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), statements);

            let scope_id = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
            // The generator function will move to this function, so we need
            // to change the parent scope of the generator function to the scope of this function.
            ctx.scopes_mut().change_parent_id(generator_scope_id, Some(scope_id));

            let params = Self::create_empty_params(ctx);
            let id = Some(bound_ident.create_binding_identifier(ctx));
            let caller_function = Self::create_function(id, params, body, scope_id, ctx);
            Some(Statement::from(ctx.ast.declaration_from_function(caller_function)))
        }
    }

    /// Transforms async arrow functions into generator functions wrapped in the asyncToGenerator helper.
    fn transform_arrow_function(
        &self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !arrow.r#async {
            return None;
        }

        let mut body = ctx.ast.move_function_body(&mut arrow.body);

        // If the arrow's expression is true, we need to wrap the only one expression with return statement.
        if arrow.expression {
            let statement = body.statements.first_mut().unwrap();
            let expression = match statement {
                Statement::ExpressionStatement(es) => ctx.ast.move_expression(&mut es.expression),
                _ => unreachable!(),
            };
            *statement = ctx.ast.statement_return(expression.span(), Some(expression));
        }

        let params = ctx.alloc(ctx.ast.move_formal_parameters(&mut arrow.params));
        let generator_function_id = arrow.scope_id.get().unwrap();
        ctx.scopes_mut().get_flags_mut(generator_function_id).remove(ScopeFlags::Arrow);

        if !Self::is_function_length_affected(&params) {
            return Some(self.create_async_to_generator_call(
                params,
                ctx.ast.alloc(body),
                generator_function_id,
                ctx,
            ));
        }

        let wrapper_scope_id = ctx.create_child_scope(ctx.current_scope_id(), ScopeFlags::Function);
        // The generator function will move to inside wrapper, so we need
        // to change the parent scope of the generator function to the wrapper function.
        ctx.scopes_mut().change_parent_id(generator_function_id, Some(wrapper_scope_id));

        let bound_ident = Self::create_bound_identifier(None, wrapper_scope_id, ctx);

        let caller_function = {
            let scope_id = ctx.create_child_scope(wrapper_scope_id, ScopeFlags::Function);
            let params = Self::create_placeholder_params(&params, scope_id, ctx);
            let statements = ctx.ast.vec1(Self::create_apply_call_statement(&bound_ident, ctx));
            let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), statements);
            let id = Self::infer_function_id_from_variable_declarator(wrapper_scope_id, ctx);
            let function = Self::create_function(id, params, body, scope_id, ctx);
            let argument = Some(ctx.ast.expression_from_function(function));
            ctx.ast.statement_return(SPAN, argument)
        };

        // Wrapper function
        {
            let statement = self.create_async_to_generator_declaration(
                &bound_ident,
                params,
                ctx.ast.alloc(body),
                generator_function_id,
                ctx,
            );
            let mut statements = ctx.ast.vec_with_capacity(2);
            statements.push(statement);
            statements.push(caller_function);
            let body = ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), statements);
            let params = Self::create_empty_params(ctx);
            let wrapper_function = Self::create_function(None, params, body, wrapper_scope_id, ctx);
            // Construct the IIFE
            let callee = ctx.ast.expression_from_function(wrapper_function);
            Some(ctx.ast.expression_call(SPAN, callee, NONE, ctx.ast.vec(), false))
        }
    }

    /// Infers the function id from [`Ancestor::VariableDeclaratorInit`].
    fn infer_function_id_from_variable_declarator(
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<BindingIdentifier<'a>> {
        let Ancestor::VariableDeclaratorInit(declarator) = ctx.parent() else {
            return None;
        };
        let Some(id) = declarator.id().get_binding_identifier() else { unreachable!() };
        Some(
            ctx.generate_binding(id.name.clone(), scope_id, SymbolFlags::FunctionScopedVariable)
                .create_binding_identifier(ctx),
        )
    }

    /// Creates a [`Function`] with the specified params, body and scope_id.
    #[inline]
    fn create_function(
        id: Option<BindingIdentifier<'a>>,
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Function<'a> {
        let r#type = if id.is_some() {
            FunctionType::FunctionDeclaration
        } else {
            FunctionType::FunctionExpression
        };
        ctx.ast.function_with_scope_id(
            r#type,
            SPAN,
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
        )
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
        let symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), "arguments");
        let arguments_ident =
            ctx.create_reference_id(SPAN, Atom::from("arguments"), symbol_id, ReferenceFlags::Read);
        let arguments_ident = ctx.ast.expression_from_identifier_reference(arguments_ident);

        // (this, arguments)
        let mut arguments = ctx.ast.vec_with_capacity(2);
        arguments.push(ctx.ast.argument_expression(ctx.ast.expression_this(SPAN)));
        arguments.push(ctx.ast.argument_expression(arguments_ident));
        // _ref.apply
        let callee = ctx.ast.expression_member(ctx.ast.member_expression_static(
            SPAN,
            bound_ident.create_read_expression(ctx),
            ctx.ast.identifier_name(SPAN, "apply"),
            false,
        ));
        let argument = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
        ctx.ast.statement_return(SPAN, Some(argument))
    }

    /// Creates an [`Expression`] that calls the [`Helper::AsyncToGenerator`] helper function.
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
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut function = Self::create_function(None, params, body, scope_id, ctx);
        function.generator = true;
        let function_expression = ctx.ast.expression_from_function(function);
        let argument = ctx.ast.argument_expression(function_expression);
        let arguments = ctx.ast.vec1(argument);
        self.ctx.helper_call_expr(Helper::AsyncToGenerator, arguments, ctx)
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
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let init = self.create_async_to_generator_call(params, body, scope_id, ctx);
        let declarations = ctx.ast.vec1(ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            bound_ident.create_binding_pattern(ctx),
            Some(init),
            false,
        ));
        ctx.ast.statement_declaration(ctx.ast.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            declarations,
            false,
        ))
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
        params: Box<'a, FormalParameters<'a>>,
        body: Box<'a, FunctionBody<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let right = self.create_async_to_generator_call(params, body, scope_id, ctx);
        let expression = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            bound.create_write_target(ctx),
            right,
        );
        ctx.ast.statement_expression(SPAN, expression)
    }

    /// Creates placeholder [`FormalParameters`] which named `_x` based on the passed-in parameters.
    /// `function p(x, y, z, d = 0, ...rest) {}` -> `function* (_x, _x1, _x2) {}`
    fn create_placeholder_params(
        params: &FormalParameters<'a>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Box<'a, FormalParameters<'a>> {
        let mut parameters = ctx.ast.vec_with_capacity(params.items.len());
        for param in &params.items {
            if param.pattern.kind.is_assignment_pattern() {
                break;
            }
            let binding = ctx.generate_uid("x", scope_id, SymbolFlags::FunctionScopedVariable);
            parameters.push(
                ctx.ast.plain_formal_parameter(param.span(), binding.create_binding_pattern(ctx)),
            );
        }
        let parameters = ctx.ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            parameters,
            NONE,
        );

        parameters
    }

    /// Creates an empty [FormalParameters] with [FormalParameterKind::FormalParameter].
    #[inline]
    fn create_empty_params(ctx: &mut TraverseCtx<'a>) -> Box<'a, FormalParameters<'a>> {
        ctx.ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::FormalParameter,
            ctx.ast.vec(),
            NONE,
        )
    }

    /// Creates a [`BoundIdentifier`] for the id of the function.
    #[inline]
    fn create_bound_identifier(
        id: Option<&BindingIdentifier<'a>>,
        scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        ctx.generate_uid(
            id.as_ref().map_or_else(|| "ref", |id| id.name.as_str()),
            scope_id,
            SymbolFlags::FunctionScopedVariable,
        )
    }

    /// Checks if the function length is affected by the parameters.
    ///
    /// TODO: Needs to handle `ignoreFunctionLength` assumption.
    // <https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-helper-wrap-function/src/index.ts#L164>
    #[inline]
    fn is_function_length_affected(params: &FormalParameters<'_>) -> bool {
        params.items.first().is_some_and(|param| !param.pattern.kind.is_assignment_pattern())
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
}

impl<'a, 'ctx> Visit<'a> for BindingMover<'a, 'ctx> {
    /// Visits a binding identifier and moves it to the target scope.
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        let symbols = self.ctx.symbols();
        let symbol_id = ident.symbol_id.get().unwrap();
        let current_scope_id = symbols.get_scope_id(symbol_id);
        let scopes = self.ctx.scopes_mut();
        scopes.move_binding(current_scope_id, self.target_scope_id, ident.name.as_str());
        let symbols = self.ctx.symbols_mut();
        symbols.set_scope_id(symbol_id, self.target_scope_id);
    }
}
