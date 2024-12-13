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

use compact_str::CompactString;
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashSet};

use oxc_allocator::{Box as ArenaBox, Vec as ArenaVec};
use oxc_ast::{ast::*, NONE};
use oxc_data_structures::stack::{NonEmptyStack, SparseStack};
use oxc_semantic::{ReferenceFlags, SymbolId};
use oxc_span::{CompactStr, SPAN};
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::EnvOptions;

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

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

#[derive(PartialEq, Eq, Hash)]
struct SuperMethodKey<'a> {
    /// If it is true, the method should accept a value parameter.
    is_assignment: bool,
    /// Name of property getter/setter is for.
    /// Empty string for computed properties.
    property: &'a str,
}

struct SuperMethodInfo<'a> {
    binding: BoundIdentifier<'a>,
    super_expr: Expression<'a>,
    /// If it is true, the method should accept a prop parameter.
    is_computed: bool,
}

pub struct ArrowFunctionConverter<'a> {
    mode: ArrowFunctionConverterMode,
    this_var_stack: SparseStack<BoundIdentifier<'a>>,
    arguments_var_stack: SparseStack<BoundIdentifier<'a>>,
    arguments_needs_transform_stack: NonEmptyStack<bool>,
    renamed_arguments_symbol_ids: FxHashSet<SymbolId>,
    // TODO(improve-on-babel): `FxHashMap` would suffice here. Iteration order is not important.
    // Only using `FxIndexMap` for predictable iteration order to match Babel's output.
    super_methods: Option<FxIndexMap<SuperMethodKey<'a>, SuperMethodInfo<'a>>>,
}

impl<'a> ArrowFunctionConverter<'a> {
    pub fn new(env: &EnvOptions) -> Self {
        let mode = if env.es2015.arrow_function.is_some() {
            ArrowFunctionConverterMode::Enabled
        } else if env.es2017.async_to_generator || env.es2018.async_generator_functions {
            ArrowFunctionConverterMode::AsyncOnly
        } else {
            ArrowFunctionConverterMode::Disabled
        };
        // `SparseStack`s are created with 1 empty entry, for `Program`
        Self {
            mode,
            this_var_stack: SparseStack::new(),
            arguments_var_stack: SparseStack::new(),
            arguments_needs_transform_stack: NonEmptyStack::new(false),
            renamed_arguments_symbol_ids: FxHashSet::default(),
            super_methods: None,
        }
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

        let this_var = self.this_var_stack.take_last();
        let arguments_var = self.arguments_var_stack.take_last();
        self.insert_variable_statement_at_the_top_of_statements(
            program.scope_id(),
            &mut program.body,
            this_var,
            arguments_var,
            ctx,
        );

        debug_assert!(self.this_var_stack.len() == 1);
        debug_assert!(self.this_var_stack.last().is_none());
        debug_assert!(self.arguments_var_stack.len() == 1);
        debug_assert!(self.arguments_var_stack.last().is_none());
    }

    fn enter_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() || func.body.is_none() {
            return;
        }

        self.this_var_stack.push(None);
        self.arguments_var_stack.push(None);
        if self.is_async_only() && func.r#async && Self::is_class_method_like_ancestor(ctx.parent())
        {
            self.super_methods = Some(FxIndexMap::default());
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
    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_disabled() {
            return;
        }

        let scope_id = func.scope_id();
        let Some(body) = &mut func.body else {
            return;
        };
        let this_var = self.this_var_stack.pop();
        let arguments_var = self.arguments_var_stack.pop();
        self.insert_variable_statement_at_the_top_of_statements(
            scope_id,
            &mut body.statements,
            this_var,
            arguments_var,
            ctx,
        );
    }

    fn enter_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if self.is_async_only() {
            let previous = *self.arguments_needs_transform_stack.last();
            self.arguments_needs_transform_stack.push(previous || arrow.r#async);
        }
    }

    fn enter_function_body(&mut self, _body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_async_only() {
            // Ignore arrow functions
            if let Ancestor::FunctionBody(func) = ctx.parent() {
                let is_async_method =
                    *func.r#async() && Self::is_class_method_like_ancestor(ctx.ancestor(1));
                self.arguments_needs_transform_stack.push(is_async_method);
            }
        }
    }

    fn exit_function_body(&mut self, _body: &mut FunctionBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        // This covers exiting either a `Function` or an `ArrowFunctionExpression`
        if self.is_async_only() {
            self.arguments_needs_transform_stack.pop();
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

        let this_var = self.this_var_stack.pop();
        self.insert_variable_statement_at_the_top_of_statements(
            block.scope_id(),
            &mut block.body,
            this_var,
            // `arguments` is not allowed to be used in static blocks
            None,
            ctx,
        );
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

        let new_expr = match expr {
            Expression::ThisExpression(this) => {
                self.get_this_identifier(this.span, ctx).map(Expression::Identifier)
            }
            Expression::CallExpression(call) => self.transform_call_expression_for_super(call, ctx),
            Expression::AssignmentExpression(assignment) => {
                self.transform_assignment_expression_for_super(assignment, ctx)
            }
            match_member_expression!(Expression) => {
                self.transform_member_expression_for_super(expr, None, ctx)
            }
            _ => return,
        };

        if let Some(new_expr) = new_expr {
            *expr = new_expr;
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

    // `#[inline]` because this is a hot path
    #[inline]
    fn enter_identifier_reference(
        &mut self,
        ident: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Do this check here rather than in `transform_identifier_reference_for_arguments`
        // so that the fast path for "no transform required" doesn't require a function call
        let arguments_needs_transform = *self.arguments_needs_transform_stack.last();
        if arguments_needs_transform {
            self.transform_identifier_reference_for_arguments(ident, ctx);
        }
    }

    // `#[inline]` because this is a hot path
    #[inline]
    fn enter_binding_identifier(
        &mut self,
        ident: &mut BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Do this check here rather than in `transform_binding_identifier_for_arguments`
        // so that the fast path for "no transform required" doesn't require a function call
        let arguments_needs_transform = *self.arguments_needs_transform_stack.last();
        if arguments_needs_transform {
            self.transform_binding_identifier_for_arguments(ident, ctx);
        }
    }
}

impl<'a> ArrowFunctionConverter<'a> {
    /// Check if arrow function conversion is disabled
    fn is_disabled(&self) -> bool {
        self.mode == ArrowFunctionConverterMode::Disabled
    }

    /// Check if arrow function conversion has enabled for transform async arrow functions
    #[inline]
    fn is_async_only(&self) -> bool {
        self.mode == ArrowFunctionConverterMode::AsyncOnly
    }

    fn get_this_identifier(
        &mut self,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<ArenaBox<'a, IdentifierReference<'a>>> {
        // Find arrow function we are currently in (if we are)
        let arrow_scope_id = self.get_scope_id_from_this_affected_block(ctx)?;

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

    /// Traverses upward through ancestor nodes to find the `ScopeId` of the block
    /// that potential affects the `this` expression.
    fn get_scope_id_from_this_affected_block(&self, ctx: &mut TraverseCtx<'a>) -> Option<ScopeId> {
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
                        // Continue checking the parent to see if it's inside an async function.
                        continue;
                    } else {
                        Some(func.scope_id().get().unwrap())
                    };
                }
                Ancestor::ArrowFunctionExpressionBody(func) => {
                    return if self.is_async_only() && !*func.r#async() {
                        // Continue checking the parent to see if it's inside an async function.
                        continue;
                    } else {
                        Some(func.scope_id().get().unwrap())
                    };
                }
                // Function body (includes class method or object method)
                Ancestor::FunctionBody(func) => {
                    // If we're inside a class async method or an object async method, and `is_async_only` is true,
                    // the `AsyncToGenerator` or `AsyncGeneratorFunctions` plugin will move the body
                    // of the method into a new generator function. This transformation can cause `this`
                    // to point to the wrong context.
                    // To prevent this issue, we replace `this` with `_this`, treating it similarly
                    // to how we handle arrow functions. Therefore, we return the `ScopeId` of the function.
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
            arrow_function_expr.span,
            FunctionType::FunctionExpression,
            None,
            false,
            arrow_function_expr.r#async,
            false,
            arrow_function_expr.type_parameters,
            NONE,
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
            // Only `({ async foo() {} })` does not include non-method like `({ foo: async function() {} })`,
            // because it's just a property with a function value
            Ancestor::ObjectPropertyValue(property) => *property.method(),
            _ => false,
        }
    }

    /// Transforms a `MemberExpression` whose object is a `super` expression.
    ///
    /// In the [`AsyncToGenerator`](crate::es2017::async_to_generator::AsyncToGenerator) and
    /// [`AsyncGeneratorFunctions`](crate::es2018::async_generator_functions::AsyncGeneratorFunctions) plugins,
    /// we move the body of an async method to a new generator function. This can cause
    /// `super` expressions to appear in unexpected places, leading to syntax errors.
    ///
    /// ## How it works
    ///
    /// To correctly handle `super` expressions, we need to ensure that they remain
    /// within the async method's body.
    ///
    /// This function modifies the `super` expression to call a new arrow function
    /// whose body includes the original `super` expression. The arrow function's name
    /// is generated based on the property name, such as `_superprop_getProperty`.
    ///
    /// The `super` expressions are temporarily stored in [`Self::super_methods`]
    /// and eventually inserted by [`Self::insert_variable_statement_at_the_top_of_statements`].`
    ///
    /// ## Example
    ///
    /// Before:
    /// ```js
    /// super.property;
    /// super['property']
    /// ```
    ///
    /// After:
    /// ```js
    /// var _superprop_getProperty = () => super.property, _superprop_get = (_prop) => super[_prop];
    /// _superprop_getProperty();
    /// _superprop_get('property')
    /// ```
    fn transform_member_expression_for_super(
        &mut self,
        expr: &mut Expression<'a>,
        assign_value: Option<&mut Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let super_methods = self.super_methods.as_mut()?;

        let mut argument = None;
        let mut property = "";
        let init = match expr.to_member_expression_mut() {
            MemberExpression::ComputedMemberExpression(computed_member) => {
                if !matches!(computed_member.object, Expression::Super(_)) {
                    return None;
                }
                // The property will as a parameter to pass to the new arrow function.
                // `super[property]` to `_superprop_get(property)`
                argument = Some(ctx.ast.move_expression(&mut computed_member.expression));
                ctx.ast.move_expression(&mut computed_member.object)
            }
            MemberExpression::StaticMemberExpression(static_member) => {
                if !matches!(static_member.object, Expression::Super(_)) {
                    return None;
                }

                // Used to generate the name of the arrow function.
                property = static_member.property.name.as_str();
                ctx.ast.move_expression(expr)
            }
            MemberExpression::PrivateFieldExpression(_) => {
                // Private fields can't be accessed by `super`.
                return None;
            }
        };

        let is_assignment = assign_value.is_some();
        let key = SuperMethodKey { is_assignment, property };
        let super_info = super_methods.entry(key).or_insert_with(|| {
            let binding_name = Self::generate_super_binding_name(is_assignment, property);
            let binding = ctx
                .generate_uid_in_current_scope(&binding_name, SymbolFlags::FunctionScopedVariable);
            SuperMethodInfo { binding, super_expr: init, is_computed: argument.is_some() }
        });

        let callee = super_info.binding.create_read_expression(ctx);
        let mut arguments = ctx.ast.vec_with_capacity(
            usize::from(assign_value.is_some()) + usize::from(argument.is_some()),
        );
        // _prop
        if let Some(argument) = argument {
            arguments.push(Argument::from(argument));
        }
        // _value
        if let Some(assign_value) = assign_value {
            arguments.push(Argument::from(ctx.ast.move_expression(assign_value)));
        }
        let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
        Some(call)
    }

    /// Transform a `CallExpression` whose callee is a `super` member expression.
    ///
    /// This function modifies calls to `super` methods within arrow functions
    /// to ensure the correct `this` context is maintained after transformation.
    ///
    /// ## Example
    ///
    /// Before:
    /// ```js
    /// super.method(a, b);
    /// ```
    ///
    /// After:
    /// ```js
    /// var _superprop_getMethod = () => super.method;
    /// _superprop_getMethod.call(this, a, b);
    /// ```
    #[inline]
    fn transform_call_expression_for_super(
        &mut self,
        call: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if self.super_methods.is_none() || !call.callee.is_member_expression() {
            return None;
        }

        let object = self.transform_member_expression_for_super(&mut call.callee, None, ctx)?;
        // Add `this` as the first argument and original arguments as the rest.
        let mut arguments = ctx.ast.vec_with_capacity(call.arguments.len() + 1);
        arguments.push(Argument::from(ctx.ast.expression_this(SPAN)));
        arguments.extend(ctx.ast.move_vec(&mut call.arguments));

        let property = ctx.ast.identifier_name(SPAN, "call");
        let callee = ctx.ast.member_expression_static(SPAN, object, property, false);
        let callee = Expression::from(callee);
        Some(ctx.ast.expression_call(SPAN, callee, NONE, arguments, false))
    }

    /// Transform an `AssignmentExpression` whose assignment target is a `super` member expression.
    ///
    /// In this function, we replace assignments to call a new arrow function whose body includes
    /// [AssignmentExpression::left], and use [AssignmentExpression::right] as arguments for the call expression.
    ///
    /// ## Example
    ///
    /// Before:
    /// ```js
    /// super.value = true;
    /// ```
    ///
    /// After:
    /// ```js
    /// var _superprop_setValue = (_value) => super.value = _value;
    /// _superprop_setValue(true);
    /// ```
    fn transform_assignment_expression_for_super(
        &mut self,
        assignment: &mut AssignmentExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        // Check if the left of the assignment is a `super` member expression.
        if self.super_methods.is_none()
            || !assignment
                .left
                .as_member_expression()
                .is_some_and(|m| matches!(m.object(), Expression::Super(_)))
        {
            return None;
        }

        let assignment_target = ctx.ast.move_assignment_target(&mut assignment.left);
        let mut assignment_expr = Expression::from(assignment_target.into_member_expression());
        self.transform_member_expression_for_super(
            &mut assignment_expr,
            Some(&mut assignment.right),
            ctx,
        )
    }

    /// Adjust the scope of the binding.
    ///
    /// Since scope can be moved or deleted, we need to ensure the scope of the binding
    /// same as the target scope, if it's mismatch, we need to move the binding to the target scope.
    fn adjust_binding_scope(
        target_scope_id: ScopeId,
        binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let original_scope_id = ctx.symbols().get_scope_id(binding.symbol_id);
        if target_scope_id != original_scope_id {
            ctx.symbols_mut().set_scope_id(binding.symbol_id, target_scope_id);
            ctx.scopes_mut().move_binding(original_scope_id, target_scope_id, &binding.name);
        }
    }

    /// Generate a variable declarator for the super method by the given [`SuperMethodInfo`].
    fn generate_super_method(
        target_scope_id: ScopeId,
        super_method: SuperMethodInfo<'a>,
        is_assignment: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> VariableDeclarator<'a> {
        let SuperMethodInfo { binding, super_expr: mut init, is_computed } = super_method;

        Self::adjust_binding_scope(target_scope_id, &binding, ctx);
        let scope_id =
            ctx.create_child_scope(target_scope_id, ScopeFlags::Arrow | ScopeFlags::Function);

        let mut items =
            ctx.ast.vec_with_capacity(usize::from(is_computed) + usize::from(is_assignment));

        // Create a parameter for the prop if it's a computed member expression.
        if is_computed {
            // TODO(improve-on-babel): No need for UID here. Just `prop` would be fine as there's nothing
            // in `prop => super[prop]` or `(prop, value) => super[prop] = value` which can clash.
            let param_binding =
                ctx.generate_uid("prop", scope_id, SymbolFlags::FunctionScopedVariable);
            let param = ctx.ast.formal_parameter(
                SPAN,
                ctx.ast.vec(),
                param_binding.create_binding_pattern(ctx),
                None,
                false,
                false,
            );
            items.push(param);

            // `super` -> `super[prop]`
            init = Expression::from(ctx.ast.member_expression_computed(
                SPAN,
                init,
                param_binding.create_read_expression(ctx),
                false,
            ));
        };

        // Create a parameter for the value if it's an assignment.
        if is_assignment {
            // TODO(improve-on-babel): No need for UID here. Just `value` would be fine as there's nothing
            // in `value => super.prop = value` or `(prop, value) => super[prop] = value` which can clash.
            let param_binding =
                ctx.generate_uid("value", scope_id, SymbolFlags::FunctionScopedVariable);
            let param = ctx.ast.formal_parameter(
                SPAN,
                ctx.ast.vec(),
                param_binding.create_binding_pattern(ctx),
                None,
                false,
                false,
            );
            items.push(param);

            // `super[prop]` -> `super[prop] = value`
            let left = SimpleAssignmentTarget::from(init.into_member_expression());
            let left = AssignmentTarget::from(left);
            let right = param_binding.create_read_expression(ctx);
            init = ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, right);
        }

        let params = ctx.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            items,
            NONE,
        );
        let statements = ctx.ast.vec1(ctx.ast.statement_expression(SPAN, init));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), statements);
        let init = ctx.ast.alloc_arrow_function_expression_with_scope_id(
            SPAN, true, false, NONE, params, NONE, body, scope_id,
        );
        ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            binding.create_binding_pattern(ctx),
            Some(Expression::ArrowFunctionExpression(init)),
            false,
        )
    }

    /// Generate a binding name for the super method, like `superprop_getXXX`.
    fn generate_super_binding_name(is_assignment: bool, property: &str) -> CompactString {
        let mut name = if is_assignment {
            CompactString::const_new("superprop_set")
        } else {
            CompactString::const_new("superprop_get")
        };

        let Some(&first_byte) = property.as_bytes().first() else {
            return name;
        };

        // Capitalize the first letter of the property name.
        // Fast path for ASCII (very common case).
        // TODO(improve-on-babel): We could just use format `superprop_get_prop` and avoid capitalizing.
        if first_byte.is_ascii() {
            // We know `IdentifierName`s begin with `a-z`, `A-Z`, `_` or `$` if ASCII,
            // so can use a slightly cheaper conversion than `u8::to_ascii_uppercase`.
            // Adapted from `u8::to_ascii_uppercase`'s implementation.
            // https://godbolt.org/z/5Txa6Pv9z
            #[inline]
            fn ascii_ident_first_char_uppercase(b: u8) -> u8 {
                const ASCII_CASE_MASK: u8 = 0b0010_0000;
                let is_lower_case = b >= b'a';
                b ^ (u8::from(is_lower_case) * ASCII_CASE_MASK)
            }

            name.push(ascii_ident_first_char_uppercase(first_byte) as char);
            if property.len() > 1 {
                name.push_str(&property[1..]);
            }
        } else {
            #[cold]
            #[inline(never)]
            fn push_unicode(property: &str, name: &mut CompactString) {
                let mut chars = property.chars();
                let first_char = chars.next().unwrap();
                name.extend(first_char.to_uppercase());
                name.push_str(chars.as_str());
            }
            push_unicode(property, &mut name);
        }

        name
    }

    /// Rename the `arguments` symbol to a new name.
    fn rename_arguments_symbol(symbol_id: SymbolId, name: CompactStr, ctx: &mut TraverseCtx<'a>) {
        let scope_id = ctx.symbols().get_scope_id(symbol_id);
        ctx.symbols_mut().rename(symbol_id, name.clone());
        ctx.scopes_mut().rename_binding(scope_id, "arguments", name);
    }

    /// Transform the identifier reference for `arguments` if it's affected after transformation.
    ///
    /// See [`Self::transform_member_expression_for_super`] for the reason.
    fn transform_identifier_reference_for_arguments(
        &mut self,
        ident: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if &ident.name != "arguments" {
            return;
        }

        let reference_id = ident.reference_id();
        let symbol_id = ctx.symbols().get_reference(reference_id).symbol_id();

        let binding = self.arguments_var_stack.last_or_init(|| {
            if let Some(symbol_id) = symbol_id {
                let arguments_name = ctx.generate_uid_name("arguments");
                let arguments_name_atom = ctx.ast.atom(&arguments_name);
                Self::rename_arguments_symbol(symbol_id, arguments_name, ctx);
                // Record the symbol ID as a renamed `arguments` variable.
                self.renamed_arguments_symbol_ids.insert(symbol_id);
                BoundIdentifier::new(arguments_name_atom, symbol_id)
            } else {
                // We cannot determine the final scope ID of the `arguments` variable insertion,
                // because the `arguments` variable will be inserted to a new scope which haven't been created yet,
                // so we temporary use root scope id as the fake target scope ID.
                let target_scope_id = ctx.scopes().root_scope_id();
                ctx.generate_uid("arguments", target_scope_id, SymbolFlags::FunctionScopedVariable)
            }
        });

        // If no symbol ID, it means there is no variable named `arguments` in the scope.
        // The following code is just to sync semantics.
        if symbol_id.is_none() {
            let reference = ctx.symbols_mut().get_reference_mut(reference_id);
            reference.set_symbol_id(binding.symbol_id);
            ctx.scopes_mut().delete_root_unresolved_reference(&ident.name, reference_id);
            ctx.symbols_mut().resolved_references[binding.symbol_id].push(reference_id);
        }

        ident.name = binding.name.clone();
    }

    /// Transform the binding identifier for `arguments` if it's affected after transformation.
    ///
    /// The main work is to rename the `arguments` binding identifier to a new name.
    fn transform_binding_identifier_for_arguments(
        &mut self,
        ident: &mut BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `arguments` is not allowed to be defined in strict mode.
        // Check if strict mode first to avoid the more expensive string comparison check if possible.
        if ctx.current_scope_flags().is_strict_mode() || &ident.name != "arguments" {
            return;
        }

        self.arguments_var_stack.last_or_init(|| {
            let arguments_name = ctx.generate_uid_name("arguments");
            ident.name = ctx.ast.atom(&arguments_name);
            let symbol_id = ident.symbol_id();
            Self::rename_arguments_symbol(symbol_id, arguments_name, ctx);
            // Record the symbol ID as a renamed `arguments` variable.
            self.renamed_arguments_symbol_ids.insert(symbol_id);
            BoundIdentifier::new(ident.name.clone(), symbol_id)
        });
    }

    /// Create a variable declarator looks like `_arguments = arguments;`.
    fn create_arguments_var_declarator(
        &self,
        target_scope_id: ScopeId,
        arguments_var: Option<BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<VariableDeclarator<'a>> {
        let arguments_var = arguments_var?;

        // Just a renamed `arguments` variable, we don't need to create a new variable declaration.
        if self.renamed_arguments_symbol_ids.contains(&arguments_var.symbol_id) {
            return None;
        }

        Self::adjust_binding_scope(target_scope_id, &arguments_var, ctx);
        let reference =
            ctx.create_unbound_ident_reference(SPAN, Atom::from("arguments"), ReferenceFlags::Read);
        let mut init = Expression::Identifier(ctx.ast.alloc(reference.clone()));

        // Top level may doesn't have `arguments`, so we need to check it.
        // `typeof arguments === "undefined" ? void 0 : arguments;`
        if ctx.scopes().root_scope_id() == target_scope_id {
            let argument = Expression::Identifier(ctx.ast.alloc(reference));
            let typeof_arguments = ctx.ast.expression_unary(SPAN, UnaryOperator::Typeof, argument);
            let undefined_literal = ctx.ast.expression_string_literal(SPAN, "undefined", None);
            let test = ctx.ast.expression_binary(
                SPAN,
                typeof_arguments,
                BinaryOperator::StrictEquality,
                undefined_literal,
            );
            init = ctx.ast.expression_conditional(SPAN, test, ctx.ast.void_0(SPAN), init);
        }

        Some(ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Var,
            arguments_var.create_binding_pattern(ctx),
            Some(init),
            false,
        ))
    }

    /// Insert variable statement at the top of the statements.
    fn insert_variable_statement_at_the_top_of_statements(
        &mut self,
        target_scope_id: ScopeId,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        this_var: Option<BoundIdentifier<'a>>,
        arguments_var: Option<BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `_arguments = arguments;`
        let arguments = self.create_arguments_var_declarator(target_scope_id, arguments_var, ctx);

        let is_class_method_like = Self::is_class_method_like_ancestor(ctx.parent());
        let declarations_count = usize::from(arguments.is_some())
            + if is_class_method_like {
                self.super_methods.as_ref().map_or(0, FxIndexMap::len)
            } else {
                0
            }
            + usize::from(this_var.is_some());

        // Exit if no declarations to be inserted
        if declarations_count == 0 {
            return;
        }

        let mut declarations = ctx.ast.vec_with_capacity(declarations_count);

        if let Some(arguments) = arguments {
            declarations.push(arguments);
        }

        // `_superprop_getSomething = () => super.something;`
        // `_superprop_setSomething = _value => super.something = _value;`
        // `_superprop_set = (_prop, _value) => super[_prop] = _value;`
        if is_class_method_like {
            if let Some(super_methods) = self.super_methods.as_mut() {
                declarations.extend(super_methods.drain(..).map(|(key, super_method)| {
                    Self::generate_super_method(
                        target_scope_id,
                        super_method,
                        key.is_assignment,
                        ctx,
                    )
                }));
            }
        }

        // `_this = this;`
        if let Some(this_var) = this_var {
            Self::adjust_binding_scope(target_scope_id, &this_var, ctx);
            let variable_declarator = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                this_var.create_binding_pattern(ctx),
                Some(ctx.ast.expression_this(SPAN)),
                false,
            );
            declarations.push(variable_declarator);
        }

        debug_assert_eq!(declarations_count, declarations.len());

        let stmt = ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            declarations,
            false,
        );

        let stmt = Statement::VariableDeclaration(stmt);

        statements.insert(0, stmt);
    }
}
