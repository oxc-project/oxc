//! ES2015 Arrow Functions
//!
//! This plugin transforms arrow functions (`() => {}`) to function expressions (`function () {}`).
//!
//! > This plugin is included in `preset-env`, in ES2015
//!
//! ## Example
//!
//! Input:
//! ```js
//! var a = () => {};
//! var a = b => b;
//!
//! const double = [1, 2, 3].map(num => num * 2);
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   _name: "Bob",
//!   _friends: ["Sally", "Tom"],
//!   printFriends() {
//!     this._friends.forEach(f => console.log(this._name + " knows " + f));
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! Output:
//! ```js
//! var a = function() {};
//! var a = function(b) {
//!   return b;
//! };
//!
//! const double = [1, 2, 3].map(function(num) {
//!   return num * 2;
//! });
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   _name: "Bob",
//!   _friends: ["Sally", "Tom"],
//!   printFriends() {
//!     var _this = this;
//!
//!     this._friends.forEach(function(f) {
//!       return console.log(_this._name + " knows " + f);
//!     });
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-exponentiation-operator](https://babel.dev/docs/babel-plugin-transform-arrow-functions).
//!
//! ## References:
//!
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-arrow-functions>
//! * Arrow function specification: <https://tc39.es/ecma262/#sec-arrow-function-definitions>

use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
use oxc_span::{CompactStr, SPAN};
use oxc_syntax::{
    node::NodeId,
    reference::ReferenceFlags,
    scope::ScopeFlags,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::{Traverse, TraverseCtx};
use serde::Deserialize;

use crate::context::Ctx;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ArrowFunctionsOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    #[serde(default)]
    pub spec: bool,
}

pub struct ArrowFunctions<'a> {
    ctx: Ctx<'a>,
    _options: ArrowFunctionsOptions,
    this_var_name: Option<Atom<'a>>,
    this_var_symbol_ids_stack: std::vec::Vec<Option<SymbolId>>,
    /// Stack to keep track of whether we are inside an arrow function or not.
    inside_arrow_function_stack: std::vec::Vec<bool>,
}

impl<'a> ArrowFunctions<'a> {
    pub fn new(options: ArrowFunctionsOptions, ctx: Ctx<'a>) -> Self {
        Self {
            ctx,
            _options: options,
            this_var_name: None,
            // Initial entries for `Program` scope
            this_var_symbol_ids_stack: vec![None],
            inside_arrow_function_stack: vec![false],
        }
    }
}

impl<'a> Traverse<'a> for ArrowFunctions<'a> {
    /// Insert `var _this = this;` for the global scope.
    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        debug_assert!(self.inside_arrow_function_stack.len() == 1);

        assert!(self.this_var_symbol_ids_stack.len() == 1);
        let symbol_id = self.this_var_symbol_ids_stack.pop().unwrap();
        if let Some(symbol_id) = symbol_id {
            self.insert_this_var_statement_at_the_top_of_statements(&mut program.body, symbol_id);
        }
    }

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        if func.body.is_some() {
            self.this_var_symbol_ids_stack.push(None);
            self.inside_arrow_function_stack.push(false);
        }
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
    fn exit_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        let Some(body) = func.body.as_mut() else {
            return;
        };

        let symbol_id = self.this_var_symbol_ids_stack.pop().unwrap();
        if let Some(symbol_id) = symbol_id {
            self.insert_this_var_statement_at_the_top_of_statements(
                &mut body.statements,
                symbol_id,
            );
        }

        self.inside_arrow_function_stack.pop().unwrap();
    }

    fn enter_arrow_function_expression(
        &mut self,
        _arrow: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.inside_arrow_function_stack.push(true);
    }

    fn exit_arrow_function_expression(
        &mut self,
        _arrow: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.inside_arrow_function_stack.pop().unwrap();
    }

    fn enter_class(&mut self, _class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.inside_arrow_function_stack.push(false);
    }

    fn exit_class(&mut self, _class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.inside_arrow_function_stack.pop().unwrap();
    }

    fn enter_jsx_element_name(
        &mut self,
        element_name: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let JSXElementName::ThisExpression(this) = element_name {
            if !self.is_inside_arrow_function() {
                return;
            }

            let ident = self.get_this_identifier(this.span, ctx);
            *element_name = self.ctx.ast.jsx_element_name_from_identifier_reference(ident);
        };
    }

    fn enter_jsx_member_expression_object(
        &mut self,
        object: &mut JSXMemberExpressionObject<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let JSXMemberExpressionObject::ThisExpression(this) = object {
            if !self.is_inside_arrow_function() {
                return;
            }

            let ident = self.get_this_identifier(this.span, ctx);
            *object = self.ctx.ast.jsx_member_expression_object_from_identifier_reference(ident);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::ThisExpression(this_expr) = expr {
            if !self.is_inside_arrow_function() {
                return;
            }

            let ident = self.get_this_identifier(this_expr.span, ctx);
            *expr = self.ctx.ast.expression_from_identifier_reference(ident);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::ArrowFunctionExpression(_) = expr {
            let Expression::ArrowFunctionExpression(arrow_function_expr) =
                ctx.ast.move_expression(expr)
            else {
                unreachable!()
            };

            *expr = self.transform_arrow_function_expression(arrow_function_expr.unbox(), ctx);
        }
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !matches!(node.init, Some(Expression::ArrowFunctionExpression(_))) {
            return;
        }

        let Some(id) = node.id.get_binding_identifier() else { return };
        *ctx.symbols_mut().get_flags_mut(id.symbol_id.get().unwrap()) &=
            !SymbolFlags::ArrowFunction;
    }
}

impl<'a> ArrowFunctions<'a> {
    fn is_inside_arrow_function(&self) -> bool {
        *self.inside_arrow_function_stack.last().unwrap()
    }

    fn get_this_identifier(
        &mut self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> IdentifierReference<'a> {
        let symbol_id_ref = self.this_var_symbol_ids_stack.last_mut().unwrap();
        let (this_atom, symbol_id) = if let Some(symbol_id) = symbol_id_ref {
            // Symbol for `_this` in this scope has already been created
            let this_atom = self.this_var_name.as_ref().unwrap().clone();
            (this_atom, *symbol_id)
        } else {
            // No symbol for `_this` in this scope.
            // Create UID for `_this` if not already created.
            let (this_atom, this_compact_str) = if let Some(this_var_name) = &self.this_var_name {
                let this_atom = this_var_name.clone();
                let this_compact_str = CompactStr::from(this_atom.as_str());
                (this_atom, this_compact_str)
            } else {
                let this_compact_str = ctx.generate_uid_name("this");
                let this_atom = ctx.ast.atom(&this_compact_str);
                self.this_var_name = Some(this_atom.clone());
                (this_atom, this_compact_str)
            };

            // Create symbol and record it for later
            let scope_id = ctx
                .scopes()
                .ancestors(ctx.current_scope_id())
                .skip(1)
                .find(|&scope_id| {
                    let scope_flags = ctx.scopes().get_flags(scope_id);
                    scope_flags.intersects(ScopeFlags::Function | ScopeFlags::Top)
                        && !scope_flags.contains(ScopeFlags::Arrow)
                })
                .unwrap();

            let symbol_id = ctx.symbols_mut().create_symbol(
                SPAN,
                this_compact_str.clone(),
                SymbolFlags::FunctionScopedVariable,
                scope_id,
                NodeId::DUMMY,
            );
            ctx.scopes_mut().add_binding(scope_id, this_compact_str, symbol_id);

            *symbol_id_ref = Some(symbol_id);

            (this_atom, symbol_id)
        };

        ctx.create_bound_reference_id(span, this_atom, symbol_id, ReferenceFlags::Read)
    }

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
            let return_statement = self.ctx.ast.statement_return(stmt.span, Some(stmt.expression));
            body.statements.push(return_statement);
        }

        let scope_id = arrow_function_expr.scope_id.get().unwrap();
        let flags = ctx.scopes_mut().get_flags_mut(scope_id);
        *flags &= !ScopeFlags::Arrow;

        let new_function = ctx.ast.function(
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
        );
        new_function.scope_id.set(Some(scope_id));

        Expression::FunctionExpression(self.ctx.ast.alloc(new_function))
    }

    /// Insert `var _this = this;` at the top of the statements.
    fn insert_this_var_statement_at_the_top_of_statements(
        &mut self,
        statements: &mut Vec<'a, Statement<'a>>,
        symbol_id: SymbolId,
    ) {
        let binding_id = BindingIdentifier::new_with_symbol_id(
            SPAN,
            self.this_var_name.as_ref().unwrap().clone(),
            symbol_id,
        );

        let binding_pattern = self.ctx.ast.binding_pattern(
            self.ctx.ast.binding_pattern_kind_from_binding_identifier(binding_id),
            NONE,
            false,
        );

        let variable_declarator = self.ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            binding_pattern,
            Some(self.ctx.ast.expression_this(SPAN)),
            false,
        );

        let stmt = self.ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            self.ctx.ast.vec1(variable_declarator),
            false,
        );

        let stmt = Statement::VariableDeclaration(stmt);

        statements.insert(0, stmt);
    }
}
