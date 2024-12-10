//! ES2022: Class Properties
//! Transform of static property initializers.

use std::cell::Cell;

use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform static property initializer.
    ///
    /// Replace `this`, and references to class name, with temp var for class. Transform private fields.
    /// See below for full details of transforms.
    pub(super) fn transform_static_initializer(
        &mut self,
        value: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.set_is_transforming_static_property_initializers(true);

        let mut replacer = StaticInitializerVisitor::new(self, ctx);
        replacer.visit_expression(value);

        self.set_is_transforming_static_property_initializers(false);
    }

    /// Set flag on `ClassBindings` that we are/are not currently transforming static prop initializers.
    ///
    /// The logic around which bindings are used for transforming private fields is complex,
    /// so we use this to make sure the logic is correct.
    ///
    /// In debug builds, `ClassBindings::get_or_init_temp_binding` will panic if we end up transforming
    /// a static private field, and there's no `temp` binding - which should be impossible.
    #[inline(always)] // `#[inline(always)]` because is no-op in release builds
    #[allow(clippy::inline_always)]
    #[cfg_attr(not(debug_assertions), expect(unused_variables, clippy::unused_self))]
    fn set_is_transforming_static_property_initializers(&mut self, is_it: bool) {
        #[cfg(debug_assertions)]
        {
            self.class_bindings.currently_transforming_static_property_initializers = is_it;
            if let Some(private_props) = self.private_props_stack.last_mut() {
                private_props.class_bindings.currently_transforming_static_property_initializers =
                    is_it;
            }
        }
    }
}

/// Visitor to transform:
///
/// 1. `this` to class temp var.
///    * Class declaration: `class C { static x = this.y; }`
///      -> `var _C; class C {}; _C = C; C.x = _C.y;`
///    * Class expression: `x = class C { static x = this.y; }`
///      -> `var _C; x = (_C = class C {}, _C.x = _C.y, _C)`
/// 2. Reference to class name to class temp var.
///    * Class declaration: `class C { static x = C.y; }`
///      -> `var _C; class C {}; _C = C; C.x = _C.y;`
///    * Class expression: `x = class C { static x = C.y; }`
///      -> `var _C; x = (_C = class C {}, _C.x = _C.y, _C)`
/// 3. Private fields which refer to private props of this class.
///    * Class declaration: `class C { static #x = 123; static y = this.#x; }`
///      -> `var _C; class C {}; _C = C; var _x = { _: 123 }; C.y = _assertClassBrand(_C, _C, _x)._;`
///    * Class expression: `x = class C { static #x = 123; static y = this.#x; }`
///      -> `var _C, _x; x = (_C = class C {}, _x = { _: 123 }, _C.y = _assertClassBrand(_C, _C, _x)._), _C)`
///
/// Also sets `ScopeFlags` of scopes to sloppy mode if code outside the class is sloppy mode.
///
/// Reason we need to transform `this` is because the initializer is being moved from inside the class
/// to outside. `this` outside the class refers to a different `this`, and private fields are only valid
/// within the class body. So we need to transform them.
///
/// Note that for class declarations, assignments are made to properties of original class name `C`,
/// but temp var `_C` is used in replacements for `this` or class name, and private fields.
/// This is because class binding `C` could be mutated, and the initializer may contain functions which
/// are not executed immediately, so the mutation occurs before that initializer code runs.
///
/// ```js
/// class C {
///   static getSelf = () => this;
///   static getSelf2 = () => C;
/// }
/// const C2 = C;
/// C = 123;
/// assert(C2.getSelf() === C); // Would fail if `this` was replaced with `C`, instead of temp var
/// assert(C2.getSelf2() === C); // Would fail if `C` in `getSelf2` was not replaced with temp var
/// ```
///
/// If this class defines no private properties, class has no name, and no `ScopeFlags` need updating,
/// then we only need to transform `this`. So can skip traversing into functions and other contexts
/// which have their own `this`.
///
/// Note: Those functions could contain private fields referring to a *parent* class's private props,
/// but we don't need to transform them here as they remain in same class scope.
//
// TODO(improve-on-babel): Unnecessary to create temp var for class declarations if either:
// 1. Class name binding is not mutated.
// 2. `this` / reference to class name / private field is not in a nested function, so we know the
//    code runs immediately, before any mutation of the class name binding can occur.
//
// TODO(improve-on-babel): Updating `ScopeFlags` for strict mode makes semantic correctly for the output,
// but actually the transform isn't right. Should wrap initializer in a strict mode IIFE so that
// initializer code runs in strict mode, as it was before within class body.
//
// TODO: Also re-parent child scopes.
struct StaticInitializerVisitor<'a, 'ctx, 'v> {
    /// `true` if class has name, or class has private properties, or `ScopeFlags` need updating.
    /// Any of these neccesitates walking the whole tree. If none of those apply, we only need to
    /// walk as far as functions and other constructs which define a `this`.
    walk_deep: bool,
    /// `true` if should make scopes sloppy mode
    make_sloppy_mode: bool,
    /// Incremented when entering a different `this` context, decremented when exiting it.
    /// `this` should be transformed when `this_depth == 0`.
    this_depth: u32,
    /// Main transform instance.
    class_properties: &'v mut ClassProperties<'a, 'ctx>,
    /// `TraverseCtx` object.
    ctx: &'v mut TraverseCtx<'a>,
}

impl<'a, 'ctx, 'v> StaticInitializerVisitor<'a, 'ctx, 'v> {
    fn new(
        class_properties: &'v mut ClassProperties<'a, 'ctx>,
        ctx: &'v mut TraverseCtx<'a>,
    ) -> Self {
        let make_sloppy_mode = !ctx.current_scope_flags().is_strict_mode();
        Self {
            walk_deep: make_sloppy_mode
                || class_properties.class_bindings.name.is_some()
                || class_properties.private_props_stack.last().is_some(),
            make_sloppy_mode,
            this_depth: 0,
            class_properties,
            ctx,
        }
    }
}

impl<'a, 'ctx, 'v> VisitMut<'a> for StaticInitializerVisitor<'a, 'ctx, 'v> {
    // TODO: Also need to call class visitors so private props stack is in correct state.
    // Otherwise, in this example, `#x` in `getInnerX` is resolved incorrectly
    // and `getInnerX()` will return 1 instead of 2.
    // We have to visit the inner class now rather than later after exiting outer class so that
    // `#y` in `getOuterY` resolves correctly too.
    // ```js
    // class Outer {
    //   #x = 1;
    //   #y = 1;
    //   static inner = class Inner {
    //     #x = 2;
    //     getInnerX() {
    //       return this.#x; // Should equal 2
    //     }
    //     getOuterY() {
    //       return this.#y; // Should equal 1
    //     }
    //   };
    // }
    // ```
    //
    // Need to save all per-class state (`insert_before` etc), and restore it again after.
    // Using a stack would be overkill because nested classes in static blocks will be rare.

    #[inline]
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            // `this`
            Expression::ThisExpression(this_expr) => {
                let span = this_expr.span;
                self.replace_this_with_temp_var(expr, span);
                return;
            }
            // `delete this` / `delete object?.#prop.xyz`
            Expression::UnaryExpression(unary_expr) => {
                if unary_expr.operator == UnaryOperator::Delete {
                    match &unary_expr.argument {
                        Expression::ThisExpression(_) => {
                            let span = unary_expr.span;
                            self.replace_delete_this_with_true(expr, span);
                            return;
                        }
                        Expression::ChainExpression(_) => {
                            // Call directly into `transform_unary_expression_impl` rather than
                            // main entry point `transform_unary_expression`. We already checked that
                            // `expr` is `delete <chain expression>`, so can avoid checking that again.
                            self.class_properties.transform_unary_expression_impl(expr, self.ctx);
                        }
                        _ => {}
                    }
                }
            }
            // `object.#prop`
            Expression::PrivateFieldExpression(_) => {
                self.class_properties.transform_private_field_expression(expr, self.ctx);
            }
            // `object.#prop()`
            Expression::CallExpression(_) => {
                self.class_properties.transform_call_expression(expr, self.ctx);
            }
            // `object.#prop = value`, `object.#prop += value`, `object.#prop ??= value` etc
            Expression::AssignmentExpression(_) => {
                self.class_properties.transform_assignment_expression(expr, self.ctx);
            }
            // `object.#prop++`, `--object.#prop`
            Expression::UpdateExpression(_) => {
                self.class_properties.transform_update_expression(expr, self.ctx);
            }
            // `object?.#prop`
            Expression::ChainExpression(_) => {
                self.class_properties.transform_chain_expression(expr, self.ctx);
            }
            // "object.#prop`xyz`"
            Expression::TaggedTemplateExpression(_) => {
                self.class_properties.transform_tagged_template_expression(expr, self.ctx);
            }
            _ => {}
        }

        walk_mut::walk_expression(self, expr);
    }

    #[inline]
    fn visit_assignment_target(&mut self, target: &mut AssignmentTarget<'a>) {
        // `[object.#prop] = []`
        self.class_properties.transform_assignment_target(target, self.ctx);
        walk_mut::walk_assignment_target(self, target);
    }

    /// Transform reference to class name to temp var
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        self.replace_class_name_with_temp_var(ident);
    }

    /// Convert scope to sloppy mode if `self.make_sloppy_mode == true`
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        if self.make_sloppy_mode {
            *self.ctx.scopes_mut().get_flags_mut(scope_id.get().unwrap()) -= ScopeFlags::StrictMode;
        }
    }

    // Increment `this_depth` when entering code where `this` refers to a different `this`
    // from `this` within this class, and decrement it when exiting.
    // Therefore `this_depth == 0` when `this` refers to the `this` which needs to be transformed.
    //
    // Or, if class has no name, class has no private properties, and `ScopeFlags` don't need updating,
    // stop traversing entirely. No private field accesses need to be transformed, and no scopes need
    // flags updating, so no point searching for them.
    //
    // Also set `make_sloppy_mode = false` while traversing a construct which is strict mode.

    #[inline]
    fn visit_function(&mut self, func: &mut Function<'a>, flags: ScopeFlags) {
        let parent_sloppy_mode = self.make_sloppy_mode;
        if self.make_sloppy_mode && func.has_use_strict_directive() {
            // Function has a `"use strict"` directive in body
            self.make_sloppy_mode = false;
        }

        if self.walk_deep {
            self.this_depth += 1;
            walk_mut::walk_function(self, func, flags);
            self.this_depth -= 1;
        }

        self.make_sloppy_mode = parent_sloppy_mode;
    }

    #[inline]
    fn visit_arrow_function_expression(&mut self, func: &mut ArrowFunctionExpression<'a>) {
        let parent_sloppy_mode = self.make_sloppy_mode;
        if self.make_sloppy_mode && func.has_use_strict_directive() {
            // Arrow function has a `"use strict"` directive in body
            self.make_sloppy_mode = false;
        }

        walk_mut::walk_arrow_function_expression(self, func);

        self.make_sloppy_mode = parent_sloppy_mode;
    }

    #[inline]
    fn visit_class(&mut self, class: &mut Class<'a>) {
        let parent_sloppy_mode = self.make_sloppy_mode;
        // Classes are always strict mode
        self.make_sloppy_mode = false;

        walk_mut::walk_class(self, class);

        self.make_sloppy_mode = parent_sloppy_mode;
    }

    #[inline]
    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        if self.walk_deep {
            self.this_depth += 1;
            walk_mut::walk_static_block(self, block);
            self.this_depth -= 1;
        }
    }

    #[inline]
    fn visit_ts_module_block(&mut self, block: &mut TSModuleBlock<'a>) {
        let parent_sloppy_mode = self.make_sloppy_mode;
        if self.make_sloppy_mode && block.has_use_strict_directive() {
            // Block has a `"use strict"` directive in body
            self.make_sloppy_mode = false;
        }

        if self.walk_deep {
            self.this_depth += 1;
            walk_mut::walk_ts_module_block(self, block);
            self.this_depth -= 1;
        }

        self.make_sloppy_mode = parent_sloppy_mode;
    }

    #[inline]
    fn visit_property_definition(&mut self, prop: &mut PropertyDefinition<'a>) {
        // `this` in computed key of property or method refers to `this` of parent class.
        // So visit computed `key` within current `this` scope,
        // but increment `this_depth` before visiting `value`.
        // ```js
        // class Outer {
        //   static prop = class Inner { [this] = 1; };
        // }
        // ```
        // Don't visit `type_annotation` field because can't contain `this` or private props.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }

        if self.walk_deep {
            if let Some(value) = &mut prop.value {
                self.this_depth += 1;
                self.visit_expression(value);
                self.this_depth -= 1;
            }
        }
    }

    #[inline]
    fn visit_accessor_property(&mut self, prop: &mut AccessorProperty<'a>) {
        // Treat `key` and `value` in same way as `visit_property_definition` above.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }

        if self.walk_deep {
            if let Some(value) = &mut prop.value {
                self.this_depth += 1;
                self.visit_expression(value);
                self.this_depth -= 1;
            }
        }
    }
}

impl<'a, 'ctx, 'v> StaticInitializerVisitor<'a, 'ctx, 'v> {
    /// Replace `this` with reference to temp var for class.
    fn replace_this_with_temp_var(&mut self, expr: &mut Expression<'a>, span: Span) {
        if self.this_depth == 0 {
            let temp_binding = self.class_properties.get_temp_binding(self.ctx);
            *expr = temp_binding.create_spanned_read_expression(span, self.ctx);
        }
    }

    /// Replace reference to class name with reference to temp var for class.
    fn replace_class_name_with_temp_var(&mut self, ident: &mut IdentifierReference<'a>) {
        // Check identifier is reference to class name
        let class_name_symbol_id = self.class_properties.class_bindings.name_symbol_id();
        let Some(class_name_symbol_id) = class_name_symbol_id else { return };

        let reference_id = ident.reference_id();
        let reference = self.ctx.symbols().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else { return };

        if symbol_id != class_name_symbol_id {
            return;
        }

        // Identifier is reference to class name. Rename it.
        let temp_binding = self.class_properties.get_temp_binding(self.ctx);
        ident.name = temp_binding.name.clone();

        let symbols = self.ctx.symbols_mut();
        symbols.get_reference_mut(reference_id).set_symbol_id(temp_binding.symbol_id);
        symbols.delete_resolved_reference(symbol_id, reference_id);
        symbols.add_resolved_reference(temp_binding.symbol_id, reference_id);
    }

    /// Replace `delete this` with `true`.
    fn replace_delete_this_with_true(&self, expr: &mut Expression<'a>, span: Span) {
        if self.this_depth == 0 {
            *expr = self.ctx.ast.expression_boolean_literal(span, true);
        }
    }
}

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    fn get_temp_binding(&mut self, ctx: &mut TraverseCtx<'a>) -> &BoundIdentifier<'a> {
        // `PrivateProps` is the source of truth for bindings if class has private props
        // because other visitors which transform private fields may create a temp binding
        // and store it on `PrivateProps`
        let class_bindings = match self.private_props_stack.last_mut() {
            Some(private_props) => &mut private_props.class_bindings,
            None => &mut self.class_bindings,
        };

        class_bindings.get_or_init_temp_binding(ctx)
    }
}
