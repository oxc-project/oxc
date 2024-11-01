//! Arrow function to expression transformation.

use std::cell::RefCell;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_data_structures::stack::NonEmptyStack;
use oxc_span::SPAN;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::context::TransformCtx;

/// Used to store scope_id which used to create the bound identifier,
/// or the bound identifier itself which is created in the scope.
enum ThisVar<'a> {
    ScopeId(ScopeId),
    BoundIdentifier(BoundIdentifier<'a>),
}

impl<'a> ThisVar<'a> {
    pub fn to_bound_identifier(&self) -> Option<&BoundIdentifier<'a>> {
        if let ThisVar::BoundIdentifier(ident) = self {
            Some(ident)
        } else {
            None
        }
    }
}

pub struct ArrowFunctionToExpression<'a, 'ctx> {
    this_var_stack: NonEmptyStack<ThisVar<'a>>,
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ArrowFunctionToExpression<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        // `NonEmptyStack` is created with PROGRAM_SCOPE_ID as the first scope.
        const PROGRAM_SCOPE_ID: ScopeId = ScopeId::new(0);
        Self { this_var_stack: NonEmptyStack::new(ThisVar::ScopeId(PROGRAM_SCOPE_ID)), ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for ArrowFunctionToExpression<'a, 'ctx> {
    // Note: No visitors for `TSModuleBlock` because `this` is not legal in TS module blocks.
    // <https://www.typescriptlang.org/play/?#code/HYQwtgpgzgDiDGEAEAxA9mpBvAsAKCSXjWCgBckANJAXiQAoBKWgPiTIAsBLKAbnwC++fGDQATAK4AbZACEQAJ2z5CxUhWp0mrdtz6D8QA>

    /// Insert `var _this = this;` for the global scope.
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Some(this_var) = self.this_var_stack.last().to_bound_identifier() {
            self.insert_this_var_statement_at_the_top_of_statements(
                &mut program.body,
                this_var,
                ctx,
            );
        }
        debug_assert!(self.this_var_stack.len() == 1);
    }

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        self.this_var_stack.push(ThisVar::ScopeId(func.scope_id.get().unwrap()));
    }

    /// ```ts
    /// function a(){
    ///   return () => console.log(this);
    /// }
    /// // to
    /// function a(){
    ///   var _this = this;
    ///   return function() { return console.log(_this); };
    /// }
    /// ```
    /// Insert the var _this = this; statement outside the arrow function
    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Some(this_var) = self.this_var_stack.pop().to_bound_identifier() {
            let Some(body) = &mut func.body else { unreachable!() };

            self.insert_this_var_statement_at_the_top_of_statements(
                &mut body.statements,
                this_var,
                ctx,
            );
        }
    }

    fn enter_static_block(&mut self, block: &mut StaticBlock<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        self.this_var_stack.push(ThisVar::ScopeId(block.scope_id.get().unwrap()));
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Some(this_var) = self.this_var_stack.pop().to_bound_identifier() {
            self.insert_this_var_statement_at_the_top_of_statements(&mut block.body, this_var, ctx);
        }
    }

    fn enter_jsx_element_name(
        &mut self,
        element_name: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.is_disabled() {
            return;
        }

        if let JSXElementName::ThisExpression(this) = element_name {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *element_name = ctx.ast.jsx_element_name_from_identifier_reference(ident);
            }
        };
    }

    fn enter_jsx_member_expression_object(
        &mut self,
        object: &mut JSXMemberExpressionObject<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.is_disabled() {
            return;
        }

        if let JSXMemberExpressionObject::ThisExpression(this) = object {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *object = ctx.ast.jsx_member_expression_object_from_identifier_reference(ident);
            }
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Expression::ThisExpression(this) = expr {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *expr = ctx.ast.expression_from_identifier_reference(ident);
            }
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Expression::ArrowFunctionExpression(arrow_function_expr) = expr {
            // Only transform async arrow functions if we are in async arrow function mode.
            if self.is_async_arrow_function() && !arrow_function_expr.r#async {
                return;
            }

            let Expression::ArrowFunctionExpression(arrow_function_expr) =
                ctx.ast.move_expression(expr)
            else {
                unreachable!()
            };

            *expr = self.transform_arrow_function_expression(arrow_function_expr.unbox(), ctx);
        }
    }
}

impl<'a, 'ctx> ArrowFunctionToExpression<'a, 'ctx> {
    fn is_disabled(&self) -> bool {
        *self.ctx.arrow_function_to_expression.mode.borrow()
            == ArrowFunctionsToExpressionMode::Disabled
    }

    fn is_async_arrow_function(&self) -> bool {
        *self.ctx.arrow_function_to_expression.mode.borrow()
            == ArrowFunctionsToExpressionMode::AsyncArrowFunction
    }

    fn get_this_identifier(
        &mut self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<IdentifierReference<'a>> {
        // If the `this` is not used in an arrow function, we don't need to transform it.
        if !self.is_in_arrow_function_scope(ctx) {
            return None;
        }

        // TODO(improve-on-babel): We create a new UID for every scope. This is pointless, as only one
        // `this` can be in scope at a time. We could create a single `_this` UID and reuse it in each
        // scope. But this does not match output for some of Babel's test cases.
        // <https://github.com/oxc-project/oxc/pull/5840>
        let this_var = self.this_var_stack.last_mut();
        let this_var = match this_var {
            // If it's a scope_id, create a new identifier in the scope
            ThisVar::ScopeId(scope_id) => {
                *this_var = ThisVar::BoundIdentifier(ctx.generate_uid(
                    "this",
                    *scope_id,
                    SymbolFlags::FunctionScopedVariable,
                ));
                let ThisVar::BoundIdentifier(ident) = this_var else { unreachable!() };
                ident
            }
            ThisVar::BoundIdentifier(ident) => ident,
        };
        Some(this_var.create_spanned_read_reference(span, ctx))
    }

    /// Check if we are in an arrow function.
    fn is_in_arrow_function_scope(&self, ctx: &mut TraverseCtx<'a>) -> bool {
        // Early exit if we are in an arrow function
        if ctx.current_scope_flags().contains(ScopeFlags::Arrow) {
            return true;
        }

        // `this` inside a class resolves to `this` *outside* the class in:
        // * `extends` clause
        // * Computed method key
        // * Computed property key
        // * Computed accessor property key (but `this` in this position is not legal TS)
        //
        // ```js
        // // All these `this` refer to global `this`
        // class C extends this {
        //     [this] = 123;
        //     static [this] = 123;
        //     [this]() {}
        //     static [this]() {}
        //     accessor [this] = 123;
        //     static accessor [this] = 123;
        // }
        // ```
        //
        // `this` resolves to the class / class instance (i.e. `this` defined *within* the class) in:
        // * Method body
        // * Method param
        // * Property value
        // * Static block
        //
        // ```js
        // // All these `this` refer to `this` defined within the class
        // class C {
        //     a = this;
        //     static b = this;
        //     #c = this;
        //     d() { this }
        //     static e() { this }
        //     #f() { this }
        //     g(x = this) {}
        //     accessor h = this;
        //     static accessor i = this;
        //     static { this }
        // }
        // ```
        //
        // So in this loop, we only exit when we encounter one of the above.
        for ancestor in ctx.ancestors() {
            match ancestor {
                // Top level
                Ancestor::ProgramBody(_)
                // Class property body
                | Ancestor::PropertyDefinitionValue(_)
                // Function (includes class method body)
                | Ancestor::FunctionParams(_)
                // Class static block
                | Ancestor::StaticBlockBody(_) => return false,
                // Arrow function
                Ancestor::ArrowFunctionExpressionParams(_) | Ancestor::ArrowFunctionExpressionBody(_) => {
                    return true
                }
                Ancestor::FunctionBody(a) => {
                    if self.is_async_arrow_function() && *a.r#async() {
                        continue;
                    }
                    return false
                }
                Ancestor::ObjectPropertyValue(_) |
                 Ancestor::MethodDefinitionValue(_) => {
                    return self.is_async_arrow_function();
                }
                _ => {
                }
            }
        }
        unreachable!();
    }

    #[expect(clippy::unused_self)]
    fn transform_arrow_function_expression(
        &mut self,
        arrow_function_expr: ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut body = arrow_function_expr.body;

        if arrow_function_expr.expression {
            assert!(body.statements.len() == 1);
            let stmt = body.statements.pop().unwrap();
            let Statement::ExpressionStatement(stmt) = stmt else { unreachable!() };
            let stmt = stmt.unbox();
            let return_statement = ctx.ast.statement_return(stmt.span, Some(stmt.expression));
            body.statements.push(return_statement);
        }

        let scope_id = arrow_function_expr.scope_id.get().unwrap();
        let flags = ctx.scopes_mut().get_flags_mut(scope_id);
        *flags &= !ScopeFlags::Arrow;

        let new_function = ctx.ast.alloc_function_with_scope_id(
            FunctionType::FunctionExpression,
            arrow_function_expr.span,
            None,
            false,
            arrow_function_expr.r#async,
            false,
            arrow_function_expr.type_parameters,
            None::<TSThisParameter<'a>>,
            arrow_function_expr.params,
            arrow_function_expr.return_type,
            Some(body),
            scope_id,
        );
        Expression::FunctionExpression(new_function)
    }

    /// Insert `var _this = this;` at the top of the statements.
    #[expect(clippy::unused_self)]
    fn insert_this_var_statement_at_the_top_of_statements(
        &self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        this_var: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let variable_declarator = ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            this_var.create_binding_pattern(ctx),
            Some(ctx.ast.expression_this(SPAN)),
            false,
        );

        let stmt = ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(variable_declarator),
            false,
        );

        let stmt = Statement::VariableDeclaration(stmt);

        statements.insert(0, stmt);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArrowFunctionsToExpressionMode {
    Disabled,
    ArrowFunction,
    AsyncArrowFunction,
}

pub struct ArrowFunctionToExpressionStore {
    mode: RefCell<ArrowFunctionsToExpressionMode>,
}

impl ArrowFunctionToExpressionStore {
    pub fn new() -> Self {
        Self { mode: RefCell::new(ArrowFunctionsToExpressionMode::Disabled) }
    }

    pub fn enable_arrow_function(&self) {
        self.mode.replace(ArrowFunctionsToExpressionMode::ArrowFunction);
    }

    pub fn enable_async_arrow_function(&self) {
        if *self.mode.borrow() != ArrowFunctionsToExpressionMode::ArrowFunction {
            self.mode.replace(ArrowFunctionsToExpressionMode::AsyncArrowFunction);
        }
    }
}
