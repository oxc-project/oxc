//! Arrow Functions Converter
//!
//! This converter transforms arrow functions (`() => {}`) to function expressions (`function () {}`).
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
//!   name: "Bob",
//!   friends: ["Sally", "Tom"],
//!   printFriends() {
//!     this.friends.forEach(f => console.log(this.name + " knows " + f));
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! Output:
//! ```js
//! var a = function() {};
//! var a = function(b) { return b; };
//!
//! const double = [1, 2, 3].map(function(num) {
//!   return num * 2;
//! });
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   name: "Bob",
//!   friends: ["Sally", "Tom"],
//!   printFriends() {
//!     var _this = this;
//!     this.friends.forEach(function(f) {
//!       return console.log(_this.name + " knows " + f);
//!     });
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! #### Example
//!
//! Using spec mode with the above example produces:
//!
//! ```js
//! var _this = this;
//!
//! var a = function a() {
//!   babelHelpers.newArrowCheck(this, _this);
//! }.bind(this);
//! var a = function a(b) {
//!   babelHelpers.newArrowCheck(this, _this);
//!   return b;
//! }.bind(this);
//!
//! const double = [1, 2, 3].map(
//!   function(num) {
//!     babelHelpers.newArrowCheck(this, _this);
//!     return num * 2;
//!   }.bind(this)
//! );
//! console.log(double); // [2,4,6]
//!
//! var bob = {
//!   name: "Bob",
//!   friends: ["Sally", "Tom"],
//!   printFriends() {
//!     var _this2 = this;
//!     this.friends.forEach(
//!       function(f) {
//!         babelHelpers.newArrowCheck(this, _this2);
//!         return console.log(this.name + " knows " + f);
//!       }.bind(this)
//!     );
//!   },
//! };
//! console.log(bob.printFriends());
//! ```
//!
//! The Implementation based on
//! <https://github.com/babel/babel/blob/d20b314c14533ab86351ecf6ca6b7296b66a57b3/packages/babel-traverse/src/path/conversion.ts#L170-L247>

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_data_structures::stack::SparseStack;
use oxc_span::SPAN;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::TransformOptions;

/// Mode for arrow function conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowFunctionConverterMode {
    /// Disable arrow function conversion
    Disabled,

    /// Convert all arrow functions to regular functions
    Enabled,

    /// Only convert async arrow functions
    AsyncOnly,
}

pub struct ArrowFunctionConverter<'a> {
    mode: ArrowFunctionConverterMode,
    this_var_stack: SparseStack<BoundIdentifier<'a>>,
}

impl<'a> ArrowFunctionConverter<'a> {
    pub fn new(options: &TransformOptions) -> Self {
        let mode = if options.env.es2015.arrow_function.is_some() {
            ArrowFunctionConverterMode::Enabled
        } else if options.env.es2017.async_to_generator
            || options.env.es2018.async_generator_functions
        {
            ArrowFunctionConverterMode::AsyncOnly
        } else {
            ArrowFunctionConverterMode::Disabled
        };
        // `SparseStack` is created with 1 empty entry, for `Program`
        Self { mode, this_var_stack: SparseStack::new() }
    }
}

impl<'a> Traverse<'a> for ArrowFunctionConverter<'a> {
    // Note: No visitors for `TSModuleBlock` because `this` is not legal in TS module blocks.
    // <https://www.typescriptlang.org/play/?#code/HYQwtgpgzgDiDGEAEAxA9mpBvAsAKCSXjWCgBckANJAXiQAoBKWgPiTIAsBLKAbnwC++fGDQATAK4AbZACEQAJ2z5CxUhWp0mrdtz6D8QA>

    /// Insert `var _this = this;` for the global scope.
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Some(this_var) = self.this_var_stack.take_last() {
            let target_scope_id = program.scope_id();
            Self::insert_this_var_statement_at_the_top_of_statements(
                &mut program.body,
                target_scope_id,
                &this_var,
                ctx,
            );
        }
        debug_assert!(self.this_var_stack.len() == 1);
        debug_assert!(self.this_var_stack.last().is_none());
    }

    fn enter_function(&mut self, _func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        self.this_var_stack.push(None);
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

        if let Some(this_var) = self.this_var_stack.pop() {
            let target_scope_id = func.scope_id();
            let Some(body) = &mut func.body else { unreachable!() };

            Self::insert_this_var_statement_at_the_top_of_statements(
                &mut body.statements,
                target_scope_id,
                &this_var,
                ctx,
            );
        }
    }

    fn enter_static_block(&mut self, _block: &mut StaticBlock<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        self.this_var_stack.push(None);
    }

    fn exit_static_block(&mut self, block: &mut StaticBlock<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Some(this_var) = self.this_var_stack.pop() {
            let target_scope_id = block.scope_id();
            Self::insert_this_var_statement_at_the_top_of_statements(
                &mut block.body,
                target_scope_id,
                &this_var,
                ctx,
            );
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
                *element_name = JSXElementName::IdentifierReference(ident);
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
                *object = JSXMemberExpressionObject::IdentifierReference(ident);
            }
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Expression::ThisExpression(this) = expr {
            if let Some(ident) = self.get_this_identifier(this.span, ctx) {
                *expr = Expression::Identifier(ident);
            }
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        if let Expression::ArrowFunctionExpression(arrow_function_expr) = expr {
            if self.is_async_only() && !arrow_function_expr.r#async {
                return;
            }

            let Expression::ArrowFunctionExpression(arrow_function_expr) =
                ctx.ast.move_expression(expr)
            else {
                unreachable!()
            };

            *expr = Self::transform_arrow_function_expression(arrow_function_expr, ctx);
        }
    }
}

impl<'a> ArrowFunctionConverter<'a> {
    /// Check if arrow function conversion is disabled
    fn is_disabled(&self) -> bool {
        self.mode == ArrowFunctionConverterMode::Disabled
    }

    /// Check if arrow function conversion has enabled for transform async arrow functions
    fn is_async_only(&self) -> bool {
        self.mode == ArrowFunctionConverterMode::AsyncOnly
    }

    fn get_this_identifier(
        &mut self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<ArenaBox<'a, IdentifierReference<'a>>> {
        // Find arrow function we are currently in (if we are)
        let arrow_scope_id = self.get_arrow_function_scope(ctx)?;

        // TODO(improve-on-babel): We create a new UID for every scope. This is pointless, as only one
        // `this` can be in scope at a time. We could create a single `_this` UID and reuse it in each
        // scope. But this does not match output for some of Babel's test cases.
        // <https://github.com/oxc-project/oxc/pull/5840>
        let this_var = self.this_var_stack.last_or_init(|| {
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
            ctx.generate_uid("this", target_scope_id, SymbolFlags::FunctionScopedVariable)
        });
        Some(ctx.ast.alloc(this_var.create_spanned_read_reference(span, ctx)))
    }

    /// Find arrow function we are currently in, if it's between current node, and where `this` is bound.
    /// Return its `ScopeId`.
    fn get_arrow_function_scope(&self, ctx: &mut TraverseCtx<'a>) -> Option<ScopeId> {
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
        let mut ancestors = ctx.ancestors();
        while let Some(ancestor) = ancestors.next() {
            match ancestor {
                // Top level
                Ancestor::ProgramBody(_)
                // Function params
                | Ancestor::FunctionParams(_)
                // Class property body
                | Ancestor::PropertyDefinitionValue(_)
                // Class accessor property body
                | Ancestor::AccessorPropertyValue(_)
                // Class static block
                | Ancestor::StaticBlockBody(_) => return None,
                // Arrow function
                Ancestor::ArrowFunctionExpressionParams(func) => {
                    return if self.is_async_only() && !*func.r#async() {
                        None
                    } else {
                        Some(func.scope_id().get().unwrap())
                    };
                }
                Ancestor::ArrowFunctionExpressionBody(func) => {
                    return if self.is_async_only() && !*func.r#async() {
                        None
                    } else {
                        Some(func.scope_id().get().unwrap())
                    };
                }
                // Function body
                Ancestor::FunctionBody(func) => {
                    return if self.is_async_only()
                    && *func.r#async()
                    && Self::is_class_method_like_ancestor(
                        ancestors.next().unwrap()
                    ) {
                        Some(func.scope_id().get().unwrap())
                    } else {
                        None
                    };
                }
                _ => {}
            }
        }
        unreachable!();
    }

    fn transform_arrow_function_expression(
        arrow_function_expr: ArenaBox<'a, ArrowFunctionExpression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let arrow_function_expr = arrow_function_expr.unbox();
        let scope_id = arrow_function_expr.scope_id();
        let flags = ctx.scopes_mut().get_flags_mut(scope_id);
        *flags &= !ScopeFlags::Arrow;

        let mut body = arrow_function_expr.body;

        if arrow_function_expr.expression {
            assert!(body.statements.len() == 1);
            let stmt = body.statements.pop().unwrap();
            let Statement::ExpressionStatement(stmt) = stmt else { unreachable!() };
            let stmt = stmt.unbox();
            let return_statement = ctx.ast.statement_return(stmt.span, Some(stmt.expression));
            body.statements.push(return_statement);
        }

        Expression::FunctionExpression(ctx.ast.alloc_function_with_scope_id(
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
        ))
    }

    /// Check whether the given [`Ancestor`] is a class method-like node.
    fn is_class_method_like_ancestor(ancestor: Ancestor) -> bool {
        match ancestor {
            // `class A { async foo() {} }`
            Ancestor::MethodDefinitionValue(_) => true,
            // Only `({ async foo() {} })` does not include non-method like `({ async foo: function() {} })`,
            // because it's just a property with a function value
            Ancestor::ObjectPropertyValue(property) => *property.method(),
            _ => false,
        }
    }

    /// Insert `var _this = this;` at the top of the statements.
    fn insert_this_var_statement_at_the_top_of_statements(
        statements: &mut ArenaVec<'a, Statement<'a>>,
        target_scope_id: ScopeId,
        this_var: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let symbol_id = this_var.symbol_id;
        let scope_id = ctx.symbols().get_scope_id(symbol_id);
        // Because scope can be moved or deleted, we need to check the scope id again,
        // if it's different, we need to move the binding to the target scope.
        if target_scope_id != scope_id {
            ctx.scopes_mut().move_binding(scope_id, target_scope_id, &this_var.name);
            ctx.symbols_mut().set_scope_id(symbol_id, target_scope_id);
        }

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
