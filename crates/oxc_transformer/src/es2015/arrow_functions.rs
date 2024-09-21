//! ES2015 Arrow Functions
//!
//! This plugin transforms arrow functions (`() => {}`) to function expressions (`function () {}`).
//!
//! > This plugin is included in `preset-env`, in ES2015
//!
//! ## Missing features
//!
//! Implementation is incomplete at present. Still TODO:
//!
//! * Handle `arguments` in arrow functions.
//! * Handle `new.target` in arrow functions.
//! * Error on arrow functions in class properties.
//!   <https://babeljs.io/repl#?code_lz=MYGwhgzhAEDC0G8BQ1oDMD2HoF5oAoBKXAPmgBcALASwgG4kBfJIA&presets=&externalPlugins=%40babel%2Fplugin-transform-arrow-functions%407.24.7>
//! * Error on `super` in arrow functions.
//!   <https://babeljs.io/repl#?code_lz=MYGwhgzhAEBiD29oG8C-AoUkYCEwCdoBTADwBciA7AExgSWXWmgFsiyALeagCgEoUTZtHzsArvkrR-0ALwA-aBDEAHIvgB0AM0QBuIRgxA&presets=&externalPlugins=%40babel%2Fplugin-transform-arrow-functions%407.24.7>
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
//! Implementation based on [@babel/plugin-transform-arrow-functions](https://babel.dev/docs/babel-plugin-transform-arrow-functions).
//!
//! ## References:
//!
//! * Babel plugin implementation: <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-arrow-functions>
//! * Arrow function specification: <https://tc39.es/ecma262/#sec-arrow-function-definitions>

use oxc_allocator::Vec;
use oxc_ast::{ast::*, NONE};
use oxc_span::SPAN;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};
use serde::Deserialize;

use crate::{context::Ctx, helpers::bindings::BoundIdentifier};

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
    this_var_stack: std::vec::Vec<Option<BoundIdentifier<'a>>>,
}

impl<'a> ArrowFunctions<'a> {
    pub fn new(options: ArrowFunctionsOptions, ctx: Ctx<'a>) -> Self {
        Self {
            ctx,
            _options: options,
            // Initial entry for `Program` scope
            this_var_stack: vec![None],
        }
    }
}

impl<'a> Traverse<'a> for ArrowFunctions<'a> {
    // Note: No visitors for `TSModuleBlock` because `this` is not legal in TS module blocks.
    // <https://www.typescriptlang.org/play/?#code/HYQwtgpgzgDiDGEAEAxA9mpBvAsAKCSXjWCgBckANJAXiQAoBKWgPiTIAsBLKAbnwC++fGDQATAK4AbZACEQAJ2z5CxUhWp0mrdtz6D8QA>

    /// Insert `var _this = this;` for the global scope.
    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        assert!(self.this_var_stack.len() == 1);
        let this_var = self.this_var_stack.pop().unwrap();
        if let Some(this_var) = this_var {
            self.insert_this_var_statement_at_the_top_of_statements(&mut program.body, &this_var);
        }
    }

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        if func.body.is_some() {
            self.this_var_stack.push(None);
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

        let this_var = self.this_var_stack.pop().unwrap();
        if let Some(this_var) = this_var {
            self.insert_this_var_statement_at_the_top_of_statements(
                &mut body.statements,
                &this_var,
            );
        }
    }

    fn enter_static_block(&mut self, _block: &mut StaticBlock<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.this_var_stack.push(None);
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, _ctx: &mut TraverseCtx<'a>) {
        let this_var = self.this_var_stack.pop().unwrap();
        if let Some(this_var) = this_var {
            self.insert_this_var_statement_at_the_top_of_statements(&mut block.body, &this_var);
        }
    }

    fn enter_jsx_element_name(
        &mut self,
        element_name: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let JSXElementName::ThisExpression(this) = element_name {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *element_name = self.ctx.ast.jsx_element_name_from_identifier_reference(ident);
            }
        };
    }

    fn enter_jsx_member_expression_object(
        &mut self,
        object: &mut JSXMemberExpressionObject<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let JSXMemberExpressionObject::ThisExpression(this) = object {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *object =
                    self.ctx.ast.jsx_member_expression_object_from_identifier_reference(ident);
            }
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Expression::ThisExpression(this) = expr {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *expr = self.ctx.ast.expression_from_identifier_reference(ident);
            }
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
}

impl<'a> ArrowFunctions<'a> {
    fn get_this_identifier(
        &mut self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<IdentifierReference<'a>> {
        // Find arrow function we are currently in (if we are)
        let arrow_scope_id = Self::get_arrow_function_scope(ctx)?;

        // TODO(improve-on-babel): We create a new UID for every scope. This is pointless, as only one
        // `this` can be in scope at a time. We could create a single `_this` UID and reuse it in each
        // scope. But this does not match output for some of Babel's test cases.
        // <https://github.com/oxc-project/oxc/pull/5840>
        let this_var = self.this_var_stack.last_mut().unwrap();
        if this_var.is_none() {
            let target_scope_id = ctx
                .scopes()
                .ancestors(arrow_scope_id)
                // Skip arrow function scope
                .skip(1)
                .find(|&scope_id| {
                    let scope_flags = ctx.scopes().get_flags(scope_id);
                    scope_flags.intersects(
                        ScopeFlags::Function | ScopeFlags::Top | ScopeFlags::ClassStaticBlock,
                    ) && !scope_flags.contains(ScopeFlags::Arrow)
                })
                .unwrap();

            this_var.replace(BoundIdentifier::new_uid(
                "this",
                target_scope_id,
                SymbolFlags::FunctionScopedVariable,
                ctx,
            ));
        }
        let this_var = this_var.as_ref().unwrap();
        Some(this_var.create_spanned_read_reference(span, ctx))
    }

    /// Find arrow function we are currently in, if it's between current node, and where `this` is bound.
    /// Return its `ScopeId`.
    fn get_arrow_function_scope(ctx: &mut TraverseCtx<'a>) -> Option<ScopeId> {
        // `this` inside a class resolves to `this` *outside* the class in:
        // * `extends` clause
        // * Computed method key
        // * Computed property key
        //
        // ```js
        // // All these `this` refer to global `this`
        // class C extends this {
        //     [this] = 123;
        //     static [this] = 123;
        //     [this]() {}
        //     static [this]() {}
        // }
        // ```
        //
        // `this` resolves to the class / class instance (i.e. `this` defined *within* the class) in:
        // * Class method bodies
        // * Class property bodies
        // * Class static blocks
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
        //     static { this }
        // }
        // ```
        //
        // So in this loop, we only exit when we encounter one of the above.
        for ancestor in ctx.ancestors() {
            match ancestor {
                // Top level
                Ancestor::ProgramBody(_)
                // Function (includes class method body)
                | Ancestor::FunctionTypeParameters(_)
                | Ancestor::FunctionThisParam(_)
                | Ancestor::FunctionParams(_)
                | Ancestor::FunctionReturnType(_)
                | Ancestor::FunctionBody(_)
                // Class property body 
                | Ancestor::PropertyDefinitionValue(_)
                // Class static block
                | Ancestor::StaticBlockBody(_) => return None,
                Ancestor::ArrowFunctionExpressionTypeParameters(func) => {
                    return Some(func.scope_id().get().unwrap())
                }
                Ancestor::ArrowFunctionExpressionParams(func) => {
                    return Some(func.scope_id().get().unwrap())
                }
                Ancestor::ArrowFunctionExpressionReturnType(func) => {
                    return Some(func.scope_id().get().unwrap())
                }
                Ancestor::ArrowFunctionExpressionBody(func) => {
                    return Some(func.scope_id().get().unwrap())
                }
                _ => {}
            }
        }
        unreachable!();
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
        this_var: &BoundIdentifier<'a>,
    ) {
        let binding_pattern = self.ctx.ast.binding_pattern(
            self.ctx
                .ast
                .binding_pattern_kind_from_binding_identifier(this_var.create_binding_identifier()),
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
