//! ES2022: Class Properties
//! Insertion of instance property initializers into constructor.
//!
//! When a class has instance properties / instance private properties, we need to either:
//! 1. Move initialization of these properties into existing constructor, or
//! 2. Add a constructor to the class containing property initializers.
//!
//! Oxc's output uses Babel's helpers (`_defineProperty`, `_classPrivateFieldInitSpec` etc).
//!
//! ## Output vs Babel and ESBuild
//!
//! Oxc's output follows Babel where:
//! 1. the class has no super class, or
//! 2. the class has no constructor, or
//! 3. constructor only contains a single `super()` call at top level of the function.
//!
//! Where a class with superclass has an existing constructor containing 1 or more `super()` calls
//! nested within the constructor, we do more like ESBuild does. We insert a single arrow function
//! `_super` at top of the function and replace all `super()` calls with `_super()`.
//!
//! Input:
//! ```js
//! class C extends S {
//!   prop = 1;
//!   constructor(yes) {
//!     if (yes) {
//!       super(2);
//!     } else {
//!       super(3);
//!     }
//!   }
//! }
//! ```
//!
//! Babel output:
//! ```js
//! class C extends S {
//!   constructor(yes) {
//!     if (yes) {
//!       super(2);
//!       this.prop = foo();
//!     } else {
//!       super(3);
//!       this.prop = foo();
//!     }
//!   }
//! }
//! ```
//! [Babel REPL](https://babeljs.io/repl#?code_lz=MYGwhgzhAEDC0FMAeAXBA7AJjAytA3gFDTQAOATgPanQC80AZpZQBQCUA3MdMJehCnIBXYCkrkWATwQQ2BbiQCWDaFJlyiJLdAhDSCCQCZOC6AF9EICAnnaSu_RIDMJ7We7uzQA&presets=&externalPlugins=%40babel%2Fplugin-transform-class-properties%407.25.9&assumptions=%7B%22setPublicClassFields%22%3Atrue%7D)
//!
//! Oxc output:
//! ```js
//! class C extends S {
//!   constructor(yes) {
//!     var _super = (..._args) => (
//!       super(..._args),
//!       this.prop = foo(),
//!       this
//!     );
//!     if (yes) {
//!       _super(2);
//!     } else {
//!       _super(3);
//!     }
//!   }
//! }
//! ```
//! ESBuild's output: [ESBuild REPL](https://esbuild.github.io/try/#dAAwLjI0LjAALS10YXJnZXQ9ZXMyMDIwAGNsYXNzIEMgZXh0ZW5kcyBTIHsKICBwcm9wID0gZm9vKCk7CiAgY29uc3RydWN0b3IoeWVzKSB7CiAgICBpZiAoeWVzKSB7CiAgICAgIHN1cGVyKDIpOwogICAgfSBlbHNlIHsKICAgICAgc3VwZXIoMyk7CiAgICB9CiAgfQp9)
//!
//! ## `super()` in constructor params
//!
//! Babel handles this case correctly for standard properties, but Babel's approach is problematic for us
//! because Babel outputs the property initializers twice if there are 2 x `super()` calls.
//! We would need to use `CloneIn` and then duplicate all the `ReferenceId`s etc.
//!
//! Instead, we create a `_super` function containing property initializers *outside* the class
//! and convert `super()` calls to `_super(super())`.
//!
//! Input:
//! ```js
//! class C extends S {
//!   prop = foo();
//!   constructor(x = super(), y = super()) {}
//! }
//! ```
//!
//! Oxc output:
//! ```js
//! let _super = function() {
//!   "use strict";
//!   this.prop = foo();
//!   return this;
//! };
//! class C extends S {
//!   constructor(x = _super.call(super()), y = _super.call(super())) {}
//! }
//! ```
//!
//! ESBuild does not handle `super()` in constructor params correctly:
//! [ESBuild REPL](https://esbuild.github.io/try/#dAAwLjI0LjAALS10YXJnZXQ9ZXMyMDIwAGNsYXNzIEMgZXh0ZW5kcyBTIHsKICBwcm9wID0gZm9vKCk7CiAgY29uc3RydWN0b3IoeCA9IHN1cGVyKCksIHkgPSBzdXBlcigpKSB7fQp9Cg)

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{ast::*, visit::walk_mut, VisitMut, NONE};
use oxc_span::SPAN;
use oxc_syntax::{
    node::NodeId,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use super::{
    utils::{create_assignment, exprs_into_stmts},
    ClassProperties,
};

/// Location to insert instance property initializers
pub(super) enum InstanceInitsInsertLocation<'a> {
    /// Create new constructor, containing initializers
    NewConstructor,
    /// Insert initializers into existing constructor at this statement index
    ExistingConstructor(usize),
    /// Create a `_super` function inside class constructor, containing initializers
    SuperFnInsideConstructor(BoundIdentifier<'a>),
    /// Create a `_super` function outside class, containing initializers
    SuperFnOutsideClass(BoundIdentifier<'a>),
}

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Replace `super()` call(s) in constructor, if required.
    ///
    /// Returns `InstanceInitsInsertLocation` detailing where instance property initializers
    /// should be inserted.
    ///
    /// * `super()` first appears as a top level statement in constructor body (common case):
    ///   * Do not alter constructor.
    ///   * No `_super` function is required.
    ///   * Returns `InstanceInitsInsertLocation::Statements`, specifying statement index
    ///     where inits should be inserted.
    /// * `super()` is used in function params:
    ///   * Replace `super()` calls with `_super.call(super())`.
    ///   * `_super` function will need to be inserted outside class.
    ///   * Returns `InstanceInitsInsertLocation::SuperFnOutsideClass`.
    /// * `super()` is found elsewhere in constructor:
    ///   * Replace `super()` calls with `_super()`.
    ///   * `_super` function will need to be inserted at top of class constructor.
    ///   * Returns `InstanceInitsInsertLocation::SuperFnInsideConstructor`.
    /// * `super()` in constructor params or body:
    ///   * `_super` function will need to be inserted at top of class constructor.
    ///   * Returns `InstanceInitsInsertLocation::SuperFnInsideConstructor`.
    ///
    /// See doc comment at top of this file for more details of last 3 cases.
    ///
    /// If a `_super` function is required, binding for `_super`, and `ScopeId` of `_super` function
    /// are recorded in the returned `InstanceInitsInsertLocation`.
    ///
    /// This function does not create the `_super` function or insert it. That happens later.
    pub(super) fn replace_super_in_constructor(
        constructor: &mut Function<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (ScopeId, InstanceInitsInsertLocation<'a>) {
        // Find any `super()`s in constructor params and replace with `_super.call(super())`
        let replacer = ConstructorParamsSuperReplacer::new(ctx);
        if let Some(result) = replacer.replace(constructor) {
            return result;
        }

        // No `super()` in constructor params.
        // Insert property initializers after `super()` statement, or in a `_super` function.
        let replacer = ConstructorBodySuperReplacer::new(constructor.scope_id(), ctx);
        let body_stmts = &mut constructor.body.as_mut().unwrap().statements;
        replacer.replace(body_stmts)
    }

    /// Insert property initializers into existing class constructor.
    ///
    /// `scope_id` has different meaning depending on type of `insertion_location`.
    pub(super) fn insert_instance_inits(
        &mut self,
        class: &mut Class<'a>,
        inits: Vec<Expression<'a>>,
        insertion_location: &InstanceInitsInsertLocation<'a>,
        scope_id: ScopeId,
        constructor_index: usize,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Handle where vars used in property init clash with vars in top scope of constructor.
        // (or maybe do that earlier?)
        // TODO: Handle private props in constructor params `class C { #x; constructor(x = this.#x) {} }`.

        match insertion_location {
            InstanceInitsInsertLocation::NewConstructor => {
                Self::insert_constructor(class, scope_id, inits, ctx);
            }
            InstanceInitsInsertLocation::ExistingConstructor(stmt_index) => {
                Self::insert_inits_into_constructor_as_statements(
                    class,
                    inits,
                    constructor_index,
                    *stmt_index,
                    ctx,
                );
            }
            InstanceInitsInsertLocation::SuperFnInsideConstructor(super_binding) => {
                Self::create_super_function_inside_constructor(
                    class,
                    inits,
                    super_binding,
                    scope_id,
                    constructor_index,
                    ctx,
                );
            }
            InstanceInitsInsertLocation::SuperFnOutsideClass(super_binding) => {
                self.create_super_function_outside_constructor(inits, super_binding, scope_id, ctx);
            }
        }
    }

    /// Add a constructor to class containing property initializers.
    fn insert_constructor(
        class: &mut Class<'a>,
        constructor_scope_id: ScopeId,
        inits: Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Create statements to go in function body.
        let has_super_class = class.super_class.is_some();
        let mut stmts = ctx.ast.vec_with_capacity(inits.len() + usize::from(has_super_class));

        // Add `super(...args);` statement and `...args` param if class has a super class.
        // `constructor(...args) { super(...args); /* prop initialization */ }`
        // TODO: One of initializers could access a var called `args` from outer scope.
        // Use a UID `_args` instead of `args` here.
        let mut params_rest = None;
        if has_super_class {
            let args_binding = ctx.generate_binding(
                Atom::from("args"),
                constructor_scope_id,
                SymbolFlags::FunctionScopedVariable,
            );
            params_rest = Some(
                ctx.ast.alloc_binding_rest_element(SPAN, args_binding.create_binding_pattern(ctx)),
            );
            stmts.push(ctx.ast.statement_expression(SPAN, create_super_call(&args_binding, ctx)));
        }
        // TODO: Should these have the span of the original `PropertyDefinition`s?
        stmts.extend(exprs_into_stmts(inits, ctx));

        let ctor = ClassElement::MethodDefinition(ctx.ast.alloc_method_definition(
            SPAN,
            MethodDefinitionType::MethodDefinition,
            ctx.ast.vec(),
            PropertyKey::StaticIdentifier(
                ctx.ast.alloc_identifier_name(SPAN, Atom::from("constructor")),
            ),
            ctx.ast.alloc_function_with_scope_id(
                SPAN,
                FunctionType::FunctionExpression,
                None,
                false,
                false,
                false,
                NONE,
                NONE,
                ctx.ast.alloc_formal_parameters(
                    SPAN,
                    FormalParameterKind::FormalParameter,
                    ctx.ast.vec(),
                    params_rest,
                ),
                NONE,
                Some(ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), stmts)),
                constructor_scope_id,
            ),
            MethodDefinitionKind::Constructor,
            false,
            false,
            false,
            false,
            None,
        ));

        // TODO(improve-on-babel): Could push constructor onto end of elements, instead of inserting as first
        class.body.body.insert(0, ctor);
    }

    /// Insert instance property initializers into constructor body at `insertion_index`.
    fn insert_inits_into_constructor_as_statements(
        class: &mut Class<'a>,
        inits: Vec<Expression<'a>>,
        constructor_index: usize,
        insertion_index: usize,
        ctx: &TraverseCtx<'a>,
    ) {
        let body_stmts = Self::get_constructor_body_stmts(class, constructor_index);
        body_stmts.splice(insertion_index..insertion_index, exprs_into_stmts(inits, ctx));
    }

    /// Create `_super` function containing instance property initializers,
    /// and insert at top of constructor body.
    /// `var _super = (..._args) => (super(..._args), <inits>, this);`
    fn create_super_function_inside_constructor(
        class: &mut Class<'a>,
        inits: Vec<Expression<'a>>,
        super_binding: &BoundIdentifier<'a>,
        super_func_scope_id: ScopeId,
        constructor_index: usize,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `(super(..._args), <inits>, this)`
        //
        // TODO(improve-on-babel): When not in loose mode, inits are `_defineProperty(this, propName, value)`.
        // `_defineProperty` returns `this`, so last statement could be `return _defineProperty(this, propName, value)`,
        // rather than an additional `return this` statement.
        // Actually this wouldn't work at present, as `_classPrivateFieldInitSpec(this, _prop, value)`
        // does not return `this`. We could alter it so it does when we have our own helper package.
        let args_binding =
            ctx.generate_uid("args", super_func_scope_id, SymbolFlags::FunctionScopedVariable);
        let super_call = create_super_call(&args_binding, ctx);
        let this_expr = ctx.ast.expression_this(SPAN);
        let body_exprs = ctx.ast.expression_sequence(
            SPAN,
            ctx.ast.vec_from_iter([super_call].into_iter().chain(inits).chain([this_expr])),
        );
        let body = ctx.ast.vec1(ctx.ast.statement_expression(SPAN, body_exprs));

        // `(..._args) => (super(..._args), <inits>, this)`
        let super_func = Expression::ArrowFunctionExpression(
            ctx.ast.alloc_arrow_function_expression_with_scope_id(
                SPAN,
                true,
                false,
                NONE,
                ctx.ast.alloc_formal_parameters(
                    SPAN,
                    FormalParameterKind::ArrowFormalParameters,
                    ctx.ast.vec(),
                    Some(ctx.ast.alloc_binding_rest_element(
                        SPAN,
                        args_binding.create_binding_pattern(ctx),
                    )),
                ),
                NONE,
                ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), body),
                super_func_scope_id,
            ),
        );

        // `var _super = (..._args) => ( ... );`
        let super_func_decl = Statement::from(ctx.ast.declaration_variable(
            SPAN,
            VariableDeclarationKind::Var,
            ctx.ast.vec1(ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                super_binding.create_binding_pattern(ctx),
                Some(super_func),
                false,
            )),
            false,
        ));

        // Insert at top of function
        let body_stmts = Self::get_constructor_body_stmts(class, constructor_index);
        body_stmts.insert(0, super_func_decl);
    }

    /// Create `_super` function containing instance property initializers,
    /// and insert it outside class.
    /// `let _super = function() { <inits>; return this; }`
    fn create_super_function_outside_constructor(
        &mut self,
        inits: Vec<Expression<'a>>,
        super_binding: &BoundIdentifier<'a>,
        super_func_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Add `"use strict"` directive if outer scope is not strict mode
        let directives = if ctx.current_scope_flags().is_strict_mode() {
            ctx.ast.vec()
        } else {
            ctx.ast.vec1(ctx.ast.use_strict_directive())
        };

        // `return this;`
        let return_stmt = ctx.ast.statement_return(SPAN, Some(ctx.ast.expression_this(SPAN)));
        // `<inits>; return this;`
        let body_stmts = ctx.ast.vec_from_iter(exprs_into_stmts(inits, ctx).chain([return_stmt]));
        // `function() { <inits>; return this; }`
        let super_func = Expression::FunctionExpression(ctx.ast.alloc_function_with_scope_id(
            SPAN,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            false,
            NONE,
            NONE,
            ctx.ast.alloc_formal_parameters(
                SPAN,
                FormalParameterKind::FormalParameter,
                ctx.ast.vec(),
                NONE,
            ),
            NONE,
            Some(ctx.ast.alloc_function_body(SPAN, directives, body_stmts)),
            super_func_scope_id,
        ));

        // Insert `_super` function after class.
        // Note: Inserting it after class not before, so that other transforms run on it.
        // TODO: That doesn't work - other transforms do not run on it.
        // TODO: If static block transform is not enabled, it's possible to construct the class
        // within the static block `class C { static { new C() } }` and that'd run before `_super`
        // is defined. So it needs to go before the class, not after, in that case.
        let init = if self.is_declaration {
            Some(super_func)
        } else {
            let assignment = create_assignment(super_binding, super_func, ctx);
            // TODO: Why does this end up before class, not after?
            self.insert_after_exprs.push(assignment);
            None
        };
        self.ctx.var_declarations.insert_let(super_binding, init, ctx);
    }

    /// Get body statements of constructor, given constructor's index within class elements.
    fn get_constructor_body_stmts<'b>(
        class: &'b mut Class<'a>,
        constructor_index: usize,
    ) -> &'b mut ArenaVec<'a, Statement<'a>> {
        let constructor = match class.body.body.get_mut(constructor_index) {
            Some(ClassElement::MethodDefinition(constructor)) => constructor.as_mut(),
            _ => unreachable!(),
        };
        debug_assert!(constructor.kind == MethodDefinitionKind::Constructor);
        &mut constructor.value.body.as_mut().unwrap().statements
    }
}

/// Visitor for transforming `super()` in class constructor params.
struct ConstructorParamsSuperReplacer<'a, 'c> {
    /// Binding for `_super` function.
    /// Initially `None`. Binding is created if `super()` is found.
    super_binding: Option<BoundIdentifier<'a>>,
    ctx: &'c mut TraverseCtx<'a>,
}

impl<'a, 'c> ConstructorParamsSuperReplacer<'a, 'c> {
    fn new(ctx: &'c mut TraverseCtx<'a>) -> Self {
        Self { super_binding: None, ctx }
    }

    fn replace(
        mut self,
        constructor: &mut Function<'a>,
    ) -> Option<(ScopeId, InstanceInitsInsertLocation<'a>)> {
        self.visit_formal_parameters(&mut constructor.params);

        #[expect(clippy::question_mark)]
        if self.super_binding.is_none() {
            // No `super()` in constructor params
            return None;
        }

        // `super()` was found in constructor params.
        // Replace any `super()`s in constructor body with `_super.call(super())`.
        // TODO: Is this correct if super class constructor returns another object?
        // ```js
        // class S { constructor() { return {}; } }
        // class C extends S { prop = 1; constructor(x = super()) {} }
        // ```
        let body_stmts = &mut constructor.body.as_mut().unwrap().statements;
        self.visit_statements(body_stmts);

        let super_binding = self.super_binding.unwrap();
        let insert_location = InstanceInitsInsertLocation::SuperFnOutsideClass(super_binding);

        // Create scope for `_super` function
        let outer_scope_id = self.ctx.current_scope_id();
        let super_func_scope_id = self.ctx.scopes_mut().add_scope(
            Some(outer_scope_id),
            NodeId::DUMMY,
            ScopeFlags::Function | ScopeFlags::StrictMode,
        );

        Some((super_func_scope_id, insert_location))
    }
}

impl<'a, 'c> VisitMut<'a> for ConstructorParamsSuperReplacer<'a, 'c> {
    /// Replace `super()` with `_super.call(super())`.
    // `#[inline]` to make hot path for all other expressions as cheap as possible.
    #[inline]
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if let Expression::CallExpression(call_expr) = expr {
            if call_expr.callee.is_super() {
                // Walk `CallExpression`'s arguments here rather than falling through to `walk_expression`
                // below to avoid infinite loop as `super()` gets visited over and over
                self.visit_arguments(&mut call_expr.arguments);

                let span = call_expr.span;
                self.wrap_super(expr, span);
                return;
            }
        }

        walk_mut::walk_expression(self, expr);
    }

    // Stop traversing where scope of current `super` ends
    #[inline]
    fn visit_function(&mut self, _func: &mut Function<'a>, _flags: ScopeFlags) {}

    #[inline]
    fn visit_static_block(&mut self, _block: &mut StaticBlock) {}

    #[inline]
    fn visit_ts_module_block(&mut self, _block: &mut TSModuleBlock<'a>) {}

    #[inline]
    fn visit_property_definition(&mut self, prop: &mut PropertyDefinition<'a>) {
        // `super()` in computed key of property or method refers to super binding of parent class.
        // So visit computed `key`, but not `value`.
        // ```js
        // class Outer extends OuterSuper {
        //   constructor(
        //     x = class Inner extends InnerSuper {
        //       [super().foo] = 1; // `super()` refers to `Outer`'s super class
        //       [super().bar]() {} // `super()` refers to `Outer`'s super class
        //       x = super(); // `super()` refers to `Inner`'s super class, but illegal syntax
        //     }
        //   ) {}
        // }
        // ```
        // Don't visit `type_annotation` field because can't contain `super()`.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }
    }

    #[inline]
    fn visit_accessor_property(&mut self, prop: &mut AccessorProperty<'a>) {
        // Visit computed `key` but not `value`, for same reasons as `visit_property_definition` above.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }
    }
}

impl<'a, 'c> ConstructorParamsSuperReplacer<'a, 'c> {
    /// Wrap `super()` -> `_super.call(super())`
    fn wrap_super(&mut self, expr: &mut Expression<'a>, span: Span) {
        let super_binding = self.super_binding.get_or_insert_with(|| {
            self.ctx.generate_uid(
                "super",
                self.ctx.current_scope_id(),
                SymbolFlags::BlockScopedVariable,
            )
        });

        let ctx = &mut *self.ctx;
        let super_call = ctx.ast.move_expression(expr);
        *expr = ctx.ast.expression_call(
            span,
            Expression::from(ctx.ast.member_expression_static(
                SPAN,
                super_binding.create_read_expression(ctx),
                ctx.ast.identifier_name(SPAN, Atom::from("call")),
                false,
            )),
            NONE,
            ctx.ast.vec1(Argument::from(super_call)),
            false,
        );
    }
}

/// Visitor for transforming `super()` in class constructor body.
struct ConstructorBodySuperReplacer<'a, 'c> {
    /// Scope of class constructor
    constructor_scope_id: ScopeId,
    /// Binding for `_super` function.
    /// Initially `None`. Binding is created if `super()` is found in position other than top-level,
    /// that requires a `_super` function.
    super_binding: Option<BoundIdentifier<'a>>,
    ctx: &'c mut TraverseCtx<'a>,
}

impl<'a, 'c> ConstructorBodySuperReplacer<'a, 'c> {
    fn new(constructor_scope_id: ScopeId, ctx: &'c mut TraverseCtx<'a>) -> Self {
        Self { constructor_scope_id, super_binding: None, ctx }
    }

    fn replace(
        mut self,
        body_stmts: &mut ArenaVec<'a, Statement<'a>>,
    ) -> (ScopeId, InstanceInitsInsertLocation<'a>) {
        let mut body_stmts_iter = body_stmts.iter_mut();

        loop {
            let mut body_stmts_iter_enumerated = body_stmts_iter.by_ref().enumerate();
            if let Some((index, stmt)) = body_stmts_iter_enumerated.next() {
                // If statement is standalone `super()`, insert inits after `super()`.
                // We can avoid a nested `_super` function for this common case.
                if let Statement::ExpressionStatement(expr_stmt) = &*stmt {
                    if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                        if call_expr.callee.is_super() {
                            let insert_location =
                                InstanceInitsInsertLocation::ExistingConstructor(index + 1);
                            return (self.constructor_scope_id, insert_location);
                        }
                    }
                }

                // Traverse statement looking for `super()` deeper in the statement
                self.visit_statement(stmt);
                if self.super_binding.is_some() {
                    break;
                }
            } else {
                // No `super()` anywhere in constructor.
                // This is weird, but legal code. It would be a runtime error if the class is constructed
                // (unless the constructor returns early).
                // In reasonable code, we should never get here.
                // Handle this weird case of no `super()` by inserting initializers in a `_super` function
                // which is never called. That is pointless, but not inserting the initializers anywhere
                // would leave `Semantic` in an inconsistent state.
                // What we get is completely legal output and correct `Semantic`, just longer than it
                // could be. But this should very rarely happen in practice, and minifier will delete
                // the `_super` function as dead code.
                // TODO: Delete the initializers instead.
                let super_binding = self.create_super_binding();
                let insert_location =
                    InstanceInitsInsertLocation::SuperFnInsideConstructor(super_binding);
                return (self.create_super_func_scope(), insert_location);
            }
        }

        // `super()` found in nested position. There may be more `super()`s in constructor.
        // Convert them all to `_super()`.
        for stmt in body_stmts_iter {
            self.visit_statement(stmt);
        }

        let super_func_scope_id = self.create_super_func_scope();
        let super_binding = self.super_binding.unwrap();
        let insert_location = InstanceInitsInsertLocation::SuperFnInsideConstructor(super_binding);
        (super_func_scope_id, insert_location)
    }

    /// Create scope for `_super` function inside constructor body
    fn create_super_func_scope(&mut self) -> ScopeId {
        self.ctx.scopes_mut().add_scope(
            Some(self.constructor_scope_id),
            NodeId::DUMMY,
            ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::StrictMode,
        )
    }
}

impl<'a, 'c> VisitMut<'a> for ConstructorBodySuperReplacer<'a, 'c> {
    /// Replace `super()` with `_super()`.
    // `#[inline]` to make hot path for all other function calls as cheap as possible.
    #[inline]
    fn visit_call_expression(&mut self, call_expr: &mut CallExpression<'a>) {
        if let Expression::Super(super_) = &call_expr.callee {
            let span = super_.span;
            self.replace_super(call_expr, span);
        }

        walk_mut::walk_call_expression(self, call_expr);
    }

    // Stop traversing where scope of current `super` ends
    #[inline]
    fn visit_function(&mut self, _func: &mut Function<'a>, _flags: ScopeFlags) {}

    #[inline]
    fn visit_static_block(&mut self, _block: &mut StaticBlock) {}

    #[inline]
    fn visit_ts_module_block(&mut self, _block: &mut TSModuleBlock<'a>) {}

    #[inline]
    fn visit_property_definition(&mut self, prop: &mut PropertyDefinition<'a>) {
        // `super()` in computed key of property or method refers to super binding of parent class.
        // So visit computed `key`, but not `value`.
        // ```js
        // class Outer extends OuterSuper {
        //   constructor() {
        //     class Inner extends InnerSuper {
        //       [super().foo] = 1; // `super()` refers to `Outer`'s super class
        //       [super().bar]() {} // `super()` refers to `Outer`'s super class
        //       x = super(); // `super()` refers to `Inner`'s super class, but illegal syntax
        //     }
        //   }
        // }
        // ```
        // Don't visit `type_annotation` field because can't contain `super()`.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }
    }

    #[inline]
    fn visit_accessor_property(&mut self, prop: &mut AccessorProperty<'a>) {
        // Visit computed `key` but not `value`, for same reasons as `visit_property_definition` above.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }
    }
}

impl<'a, 'c> ConstructorBodySuperReplacer<'a, 'c> {
    /// Replace `super(arg1, arg2)` with `_super(arg1, arg2)`
    fn replace_super(&mut self, call_expr: &mut CallExpression<'a>, span: Span) {
        if self.super_binding.is_none() {
            self.super_binding = Some(self.create_super_binding());
        }
        let super_binding = self.super_binding.as_ref().unwrap();

        call_expr.callee = super_binding.create_spanned_read_expression(span, self.ctx);
    }

    /// Create binding for `_super` function
    fn create_super_binding(&mut self) -> BoundIdentifier<'a> {
        self.ctx.generate_uid(
            "super",
            self.constructor_scope_id,
            SymbolFlags::FunctionScopedVariable,
        )
    }
}

/// `super(...args);`
fn create_super_call<'a>(
    args_binding: &BoundIdentifier<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Expression<'a> {
    ctx.ast.expression_call(
        SPAN,
        ctx.ast.expression_super(SPAN),
        NONE,
        ctx.ast
            .vec1(ctx.ast.argument_spread_element(SPAN, args_binding.create_read_expression(ctx))),
        false,
    )
}
