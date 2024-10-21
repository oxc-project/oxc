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

use std::mem;

use oxc_allocator::{Box, GetAddress};
use oxc_ast::ast::{AssignmentOperator, AssignmentTarget, IdentifierReference};
use oxc_ast::{
    ast::{
        ArrowFunctionExpression, BindingIdentifier, Expression, FormalParameterKind,
        FormalParameters, Function, FunctionBody, FunctionType, Statement, VariableDeclarationKind,
    },
    NONE,
};
use oxc_semantic::{NodeId, ReferenceFlags, ScopeFlags, ScopeId, SymbolFlags};
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
            if let Some(new_expr) = self.transform_function_expression(func, ctx) {
                *expr = new_expr;
            }
        } else if let Expression::ArrowFunctionExpression(arrow) = expr {
            if !arrow.r#async {
                return;
            }
            *expr = self.transform_arrow_function(arrow, ctx);
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::FunctionDeclaration(func) = stmt {
            if let Some(new_statement) = self.transform_function_declaration(func, ctx) {
                self.ctx.statement_injector.insert_after(stmt.address(), new_statement);
            }
        }
    }
}

impl<'a, 'ctx> AsyncToGenerator<'a, 'ctx> {
    fn transform_function_expression(
        &self,
        func: &mut Box<'a, Function<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !func.r#async || func.generator || func.is_typescript_syntax() {
            return None;
        }

        func.r#async = false;
        let id = func.id.take();
        let body = ctx.ast.move_function_body(func.body.as_mut().unwrap());
        let params = ctx.ast.move_formal_parameters(&mut func.params);
        let scope_id = func.scope_id.get().unwrap();

        // Handle `ignoreFunctionLength`
        // <https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-helper-wrap-function/src/index.ts#L164>
        if id.is_none() && !params.has_parameter() {
            return Some(self.wrap_helper_function(scope_id, params, body, ctx));
        }

        // _ref
        let ref_ident = ctx.generate_uid_in_current_scope(
            id.as_ref().map_or_else(|| "ref", |id| id.name.as_str()),
            SymbolFlags::FunctionScopedVariable,
        );

        let id = id.as_ref().map(|id| {
            let name = id.name.clone().into_compact_str();
            // Add binding to scope
            let symbol_id = ctx.symbols_mut().create_symbol(
                SPAN,
                name,
                SymbolFlags::FunctionScopedVariable | SymbolFlags::Function,
                scope_id,
                NodeId::DUMMY,
            );
            BindingIdentifier::new_with_symbol_id(SPAN, ctx.ast.atom(&id.name), symbol_id)
        });

        let apply_function = Self::get_apply_function(&ref_ident, id, &params, scope_id, ctx);
        let expression = self.wrap_helper_function(scope_id, params, body, ctx);
        let helper_call_assignment = Self::get_helper_call_declaration(&ref_ident, expression, ctx);

        let body = func.body.as_mut().unwrap();
        body.statements.push(helper_call_assignment);

        if let Some(id) = apply_function.id.as_ref() {
            let reference = ctx.create_bound_reference_id(
                SPAN,
                id.name.clone(),
                id.symbol_id.get().unwrap(),
                ReferenceFlags::Read,
            );
            let statement = Statement::from(ctx.ast.declaration_from_function(apply_function));
            body.statements.push(statement);
            let statement_return = ctx.ast.statement_return(
                SPAN,
                Some(ctx.ast.expression_from_identifier_reference(reference)),
            );
            body.statements.push(statement_return);
        } else {
            let statement_return = ctx
                .ast
                .statement_return(SPAN, Some(ctx.ast.expression_from_function(apply_function)));
            body.statements.push(statement_return);
        }

        let callee = ctx.ast.expression_from_function(ctx.ast.move_function(func));
        Some(ctx.ast.expression_call(SPAN, callee, NONE, ctx.ast.vec(), false))
    }

    /// ```js
    /// function NAME(PARAMS) { return REF.apply(this, arguments); }
    /// function REF() {
    ///     REF = FUNCTION;
    ///     return REF.apply(this, arguments);
    /// }
    /// ```
    fn transform_function_declaration(
        &self,
        func: &mut Box<'a, Function<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if !func.r#async || func.generator || func.is_typescript_syntax() {
            return None;
        }

        func.r#async = false;
        let body = ctx.ast.move_function_body(func.body.as_mut().unwrap());
        let params = ctx.ast.move_formal_parameters(&mut func.params);
        let scope_id = func.scope_id.get().unwrap();

        // _ref
        let ref_ident = ctx.generate_uid_in_current_scope(
            func.id.as_ref().map_or_else(|| "ref", |id| id.name.as_str()),
            SymbolFlags::FunctionScopedVariable,
        );

        let func_body = func.body.as_mut().unwrap();
        func_body.statements.push(Self::get_apply_call_assignment(&ref_ident, ctx));

        let mut apply_function = Self::get_apply_function(
            &ref_ident,
            Some(ref_ident.create_binding_identifier()),
            &params,
            scope_id,
            ctx,
        );
        // Swap the parameters between the original function and the apply function.
        mem::swap(&mut func.params, &mut apply_function.params);
        let expression = self.wrap_helper_function(scope_id, params, body, ctx);
        let helper_call_assignment = Self::get_helper_call_assignment(&ref_ident, expression, ctx);

        let body = apply_function.body.as_mut().unwrap();
        body.statements.insert(0, helper_call_assignment);

        Some(Statement::from(ctx.ast.declaration_from_function(apply_function)))
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

        let scope_id = arrow.scope_id.get().unwrap();
        ctx.scopes_mut().get_flags_mut(scope_id).remove(ScopeFlags::Arrow);

        self.wrap_helper_function(
            scope_id,
            ctx.ast.move_formal_parameters(&mut arrow.params),
            body,
            ctx,
        )
    }

    /// apply
    fn get_apply_function(
        ref_ident: &BoundIdentifier<'a>,
        id: Option<BindingIdentifier<'a>>,
        params: &FormalParameters<'a>,
        parent_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> Function<'a> {
        let function_scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::Function);
        let parameters = Self::generate_placeholder_parameters(ctx, params, function_scope_id);
        let statements = ctx.ast.vec1(Self::get_apply_call_assignment(ref_ident, ctx));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), statements);
        let function = ctx.ast.function(
            FunctionType::FunctionExpression,
            SPAN,
            id,
            false,
            false,
            false,
            NONE,
            NONE,
            parameters,
            NONE,
            Some(body),
        );
        function.scope_id.set(Some(function_scope_id));
        function
    }

    fn get_apply_call_assignment(
        ref_ident: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), "arguments");
        let arguments_ident =
            ctx.create_reference_id(SPAN, Atom::from("arguments"), symbol_id, ReferenceFlags::Read);
        let arguments_ident = ctx.ast.expression_from_identifier_reference(arguments_ident);

        let mut arguments = ctx.ast.vec_with_capacity(2);
        arguments.push(ctx.ast.argument_expression(ctx.ast.expression_this(SPAN)));
        arguments.push(ctx.ast.argument_expression(arguments_ident));

        ctx.ast.statement_return(
            SPAN,
            Some(ctx.ast.expression_call(
                SPAN,
                ctx.ast.expression_member(ctx.ast.member_expression_static(
                    SPAN,
                    ref_ident.create_read_expression(ctx),
                    ctx.ast.identifier_name(SPAN, "apply"),
                    false,
                )),
                NONE,
                arguments,
                false,
            )),
        )
    }

    fn wrap_helper_function(
        &self,
        scope_id: ScopeId,
        parameters: FormalParameters<'a>,
        body: FunctionBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let r#type = FunctionType::FunctionExpression;
        let function = ctx.ast.function(
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
        function.scope_id.set(Some(scope_id));
        let arguments =
            ctx.ast.vec1(ctx.ast.argument_expression(ctx.ast.expression_from_function(function)));
        self.ctx.helper_call_expr(Helper::AsyncToGenerator, arguments, ctx)
    }

    fn get_helper_call_declaration(
        ref_ident: &BoundIdentifier<'a>,
        expression: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let ref_assignment = ctx.ast.statement_declaration(ctx.ast.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                ref_ident.create_binding_pattern(ctx),
                Some(expression),
                false,
            )),
            false,
        ));
        ref_assignment
    }

    fn get_helper_call_assignment(
        ref_ident: &BoundIdentifier<'a>,
        expression: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let expression = ctx.ast.expression_assignment(
            SPAN,
            AssignmentOperator::Assign,
            AssignmentTarget::from(ctx.ast.simple_assignment_target_from_identifier_reference(
                ref_ident.create_read_reference(ctx),
            )),
            expression,
        );
        ctx.ast.statement_expression(SPAN, expression)
    }

    fn generate_placeholder_parameters(
        ctx: &mut TraverseCtx<'a>,
        params: &FormalParameters<'a>,
        function_scope_id: ScopeId,
    ) -> FormalParameters<'a> {
        let mut parameters = ctx.ast.vec_with_capacity(params.items.len());
        for param in &params.items {
            let binding =
                ctx.generate_uid("x", function_scope_id, SymbolFlags::FunctionScopedVariable);
            parameters.push(
                ctx.ast.plain_formal_parameter(param.span(), binding.create_binding_pattern(ctx)),
            );
        }
        let parameters =
            ctx.ast.formal_parameters(SPAN, FormalParameterKind::FormalParameter, parameters, NONE);
        parameters
    }
}
