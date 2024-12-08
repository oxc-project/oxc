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
//!     var _super = (..._args) => {
//!       super(..._args);
//!       this.prop = foo();
//!       return this;
//!     };
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

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Add a constructor to class containing property initializers.
    pub(super) fn insert_constructor(
        class: &mut Class<'a>,
        inits: Vec<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Create scope for constructor
        let scope_id = ctx.scopes_mut().add_scope(
            Some(class.scope_id()),
            NodeId::DUMMY,
            ScopeFlags::Function | ScopeFlags::Constructor | ScopeFlags::StrictMode,
        );

        // Create statements to go in function body.
        let has_super_class = class.super_class.is_some();
        let mut stmts = ctx.ast.vec_with_capacity(inits.len() + usize::from(has_super_class));

        // Add `super(...args);` statement and `...args` param if class has a super class.
        // `constructor(...args) { super(...args); /* prop initialization */ }`
        // TODO: One of initializers could access a var called `args` from outer scope.
        // Use a UID `_args` instead of `args` here.
        let mut params_rest = None;
        if has_super_class {
            let binding = ctx.generate_binding(
                Atom::from("args"),
                scope_id,
                SymbolFlags::FunctionScopedVariable,
            );
            params_rest =
                Some(ctx.ast.alloc_binding_rest_element(SPAN, binding.create_binding_pattern(ctx)));
            stmts.push(create_super_call_stmt(&binding, ctx));
        }
        // TODO: Should these have the span of the original `PropertyDefinition`s?
        stmts.extend(exprs_into_stmts(inits, ctx));

        let ctor = ClassElement::MethodDefinition(ctx.ast.alloc_method_definition(
            MethodDefinitionType::MethodDefinition,
            SPAN,
            ctx.ast.vec(),
            PropertyKey::StaticIdentifier(
                ctx.ast.alloc_identifier_name(SPAN, Atom::from("constructor")),
            ),
            ctx.ast.alloc_function_with_scope_id(
                FunctionType::FunctionExpression,
                SPAN,
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
                scope_id,
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

    /// Insert property initializers into existing class constructor.
    pub(super) fn insert_inits_into_constructor(
        &mut self,
        class: &mut Class<'a>,
        inits: Vec<Expression<'a>>,
        constructor_index: usize,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Handle where vars used in property init clash with vars in top scope of constructor.
        // (or maybe do that earlier?)
        // TODO: Handle private props in constructor params `class C { #x; constructor(x = this.#x) {} }`.
        let constructor = match class.body.body.get_mut(constructor_index).unwrap() {
            ClassElement::MethodDefinition(constructor) => constructor.as_mut(),
            _ => unreachable!(),
        };
        debug_assert!(constructor.kind == MethodDefinitionKind::Constructor);

        let constructor_scope_id = constructor.value.scope_id();
        let func = constructor.value.as_mut();
        let body_stmts = &mut func.body.as_mut().unwrap().statements;

        if class.super_class.is_some() {
            // Class has super class. Insert inits after `super()`.
            self.insert_inits_into_constructor_of_class_with_super_class(
                &mut func.params,
                body_stmts,
                inits,
                constructor_scope_id,
                ctx,
            );
        } else {
            // No super class. Insert inits at top of constructor.
            body_stmts.splice(0..0, exprs_into_stmts(inits, ctx));
        }
    }

    /// Insert property initializers into existing class constructor for class which has super class.
    ///
    /// * If `super()` appears only as a top-level statement in constructor
    ///   * Insert initializers as statements after it.
    ///   * Return `None`.
    /// * If `super()` appears in constructor params
    ///   * Creare a `_super` function *outside* class, which contains initializers.
    ///   * Replace `super()` in both constructor params and body with `_super.call(super())`.
    ///   * Return binding for `_super` and `_super` function as `Expression::FunctionExpression`.
    /// * If `super()` appears in constructor, but not as a top level statement:
    ///   * Insert a `_super` function inside constructor, which contains initializers.
    ///   * Replace `super()` with `_super()`.
    ///   * Return `None`.
    ///
    /// See doc comment at top of this file for more details of last 2 cases.
    fn insert_inits_into_constructor_of_class_with_super_class(
        &mut self,
        params: &mut FormalParameters<'a>,
        body_stmts: &mut ArenaVec<'a, Statement<'a>>,
        inits: Vec<Expression<'a>>,
        constructor_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Find any `super()`s in constructor params and replace with `_super.call(super())`
        // TODO: Check if any references to class name and swap them for reference to local binding.
        // TODO: Add tests for `super()` in constructor params.
        let mut replacer = ConstructorParamsSuperReplacer::new(ctx);
        replacer.visit_formal_parameters(params);

        if replacer.super_binding.is_some() {
            // `super()` was found in constructor params.
            // Replace any `super()`s in constructor body with `_super.call(super())`.
            // TODO: Is this correct if super class constructor returns another object?
            // ```js
            // class S { constructor() { return {}; } }
            // class C extends S { prop = 1; constructor(x = super()) {} }
            // ```
            replacer.visit_statements(body_stmts);

            let super_binding = replacer.super_binding.as_ref().unwrap();

            // Create `_super` function
            let super_func = ConstructorParamsSuperReplacer::create_super_func(inits, ctx);

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
        } else {
            // No `super()` in constructor params.
            // Insert property initializers after `super()` statement, or in a `_super` function.
            let mut inserter = ConstructorBodyInitsInserter::new(constructor_scope_id, ctx);
            inserter.insert(body_stmts, inits);
        }
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
}

impl<'a, 'c> VisitMut<'a> for ConstructorParamsSuperReplacer<'a, 'c> {
    /// Replace `super()` with `_super.call(super())`.
    // `#[inline]` to make hot path for all other expressions as cheap as possible.
    #[inline]
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if let Expression::CallExpression(call_expr) = expr {
            if let Expression::Super(_) = &call_expr.callee {
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
                SymbolFlags::FunctionScopedVariable,
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

    /// Create `_super` function to go outside class.
    /// `function() { <inits>; return this; }`
    //
    // TODO(improve-on-babel): When not in loose mode, inits are `_defineProperty(this, propName, value)`.
    // `_defineProperty` returns `this`, so last statement could be `return _defineProperty(this, propName, value)`,
    // rather than an additional `return this` statement.
    fn create_super_func(inits: Vec<Expression<'a>>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let outer_scope_id = ctx.current_scope_id();
        let super_func_scope_id = ctx.scopes_mut().add_scope(
            Some(outer_scope_id),
            NodeId::DUMMY,
            ScopeFlags::Function | ScopeFlags::StrictMode,
        );

        // Add `"use strict"` directive if outer scope is not strict mode
        let directives = if ctx.scopes().get_flags(outer_scope_id).is_strict_mode() {
            ctx.ast.vec()
        } else {
            ctx.ast.vec1(ctx.ast.directive(
                SPAN,
                ctx.ast.string_literal(SPAN, Atom::from("use strict"), None),
                Atom::from("use strict"),
            ))
        };

        // `return this;`
        let return_stmt = ctx.ast.statement_return(SPAN, Some(ctx.ast.expression_this(SPAN)));
        // `<inits>; return this;`
        let body_stmts = ctx.ast.vec_from_iter(exprs_into_stmts(inits, ctx).chain([return_stmt]));
        // `function() { <inits>; return this; }`
        Expression::FunctionExpression(ctx.ast.alloc_function_with_scope_id(
            FunctionType::FunctionExpression,
            SPAN,
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
        ))
    }
}

/// Visitor for transforming `super()` in class constructor body.
struct ConstructorBodyInitsInserter<'a, 'c> {
    /// Scope of class constructor
    constructor_scope_id: ScopeId,
    /// Binding for `_super` function.
    /// Initially `None`. Binding is created if `super()` is found in position other than top-level,
    /// that requires a `_super` function.
    super_binding: Option<BoundIdentifier<'a>>,
    ctx: &'c mut TraverseCtx<'a>,
}

impl<'a, 'c> ConstructorBodyInitsInserter<'a, 'c> {
    fn new(constructor_scope_id: ScopeId, ctx: &'c mut TraverseCtx<'a>) -> Self {
        Self { constructor_scope_id, super_binding: None, ctx }
    }

    fn insert(&mut self, body_stmts: &mut ArenaVec<'a, Statement<'a>>, inits: Vec<Expression<'a>>) {
        // TODO: Re-parent child scopes of `init`s
        let mut body_stmts_iter = body_stmts.iter_mut();
        let mut insert_index = 1; // 1 because want to insert after `super()`, not before

        // It's a runtime error (not a syntax error) for constructor of a class with a super class
        // not to contain `super()`.
        // So it's legal code and won't cause an error, as long as the class is never constructed!
        // In reasonable code, we should never get to end of this loop without finding `super()`,
        // but handle this weird case of no `super()` by allowing loop to exit.
        // Inits will be inserted in a `_super` function which is never called. That is pointless,
        // but exiting this function entirely would leave `Semantic` in an inconsistent state.
        // What we get is completely legal output and correct `Semantic`, just longer than it could be.
        // But this should never happen in practice, so no point writing special logic to handle it.
        for stmt in body_stmts_iter.by_ref() {
            // If statement is standalone `super()`, insert inits after `super()`.
            // We can avoid a nested `_super` function for this common case.
            if let Statement::ExpressionStatement(expr_stmt) = &*stmt {
                if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                    if let Expression::Super(_) = &call_expr.callee {
                        body_stmts
                            .splice(insert_index..insert_index, exprs_into_stmts(inits, self.ctx));
                        return;
                    }
                }
            }

            // Traverse statement looking for `super()` deeper in the statement
            self.visit_statement(stmt);
            if self.super_binding.is_some() {
                break;
            }

            insert_index += 1;
        }

        // `super()` found in nested position. There may be more `super()`s in constructor.
        // Convert them all to `_super()`.
        for stmt in body_stmts_iter {
            self.visit_statement(stmt);
        }

        // Insert `_super` function at top of constructor
        self.insert_super_func(body_stmts, inits);
    }

    /// Insert `_super` function at top of constructor.
    /// ```js
    /// var _super = (..._args) => {
    ///   super(..._args);
    ///   <inits>
    ///   return this;
    /// };
    /// ```
    fn insert_super_func(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        inits: Vec<Expression<'a>>,
    ) {
        let ctx = &mut *self.ctx;

        let super_func_scope_id = ctx.scopes_mut().add_scope(
            Some(self.constructor_scope_id),
            NodeId::DUMMY,
            ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::StrictMode,
        );
        let args_binding =
            ctx.generate_uid("args", super_func_scope_id, SymbolFlags::FunctionScopedVariable);

        // `super(..._args); <inits>; return this;`
        //
        // TODO(improve-on-babel): When not in loose mode, inits are `_defineProperty(this, propName, value)`.
        // `_defineProperty` returns `this`, so last statement could be `return _defineProperty(this, propName, value)`,
        // rather than an additional `return this` statement.
        let super_call = create_super_call_stmt(&args_binding, ctx);
        let return_stmt = ctx.ast.statement_return(SPAN, Some(ctx.ast.expression_this(SPAN)));
        let body_stmts = ctx.ast.vec_from_iter(
            [super_call].into_iter().chain(exprs_into_stmts(inits, ctx)).chain([return_stmt]),
        );

        // `(...args) => { super(..._args); <inits>; return this; }`
        let super_func = Expression::ArrowFunctionExpression(
            ctx.ast.alloc_arrow_function_expression_with_scope_id(
                SPAN,
                false,
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
                ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), body_stmts),
                super_func_scope_id,
            ),
        );

        // `var _super = (...args) => { ... }`
        // Note: `super_binding` can be `None` at this point if no `super()` found in constructor
        // (see comment above in `insert`).
        let super_binding = self
            .super_binding
            .get_or_insert_with(|| Self::generate_super_binding(self.constructor_scope_id, ctx));
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

        stmts.insert(0, super_func_decl);
    }
}

impl<'a, 'c> VisitMut<'a> for ConstructorBodyInitsInserter<'a, 'c> {
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

impl<'a, 'c> ConstructorBodyInitsInserter<'a, 'c> {
    fn replace_super(&mut self, call_expr: &mut CallExpression<'a>, span: Span) {
        let super_binding = self.super_binding.get_or_insert_with(|| {
            Self::generate_super_binding(self.constructor_scope_id, self.ctx)
        });
        call_expr.callee = super_binding.create_spanned_read_expression(span, self.ctx);
    }

    fn generate_super_binding(
        constructor_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        ctx.generate_uid("super", constructor_scope_id, SymbolFlags::FunctionScopedVariable)
    }
}

/// `super(...args);`
fn create_super_call_stmt<'a>(
    args_binding: &BoundIdentifier<'a>,
    ctx: &mut TraverseCtx<'a>,
) -> Statement<'a> {
    ctx.ast.statement_expression(
        SPAN,
        ctx.ast.expression_call(
            SPAN,
            ctx.ast.expression_super(SPAN),
            NONE,
            ctx.ast.vec1(
                ctx.ast.argument_spread_element(SPAN, args_binding.create_read_expression(ctx)),
            ),
            false,
        ),
    )
}
