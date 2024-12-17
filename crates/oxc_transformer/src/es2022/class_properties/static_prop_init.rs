//! ES2022: Class Properties
//! Transform of static property initializers.

use std::cell::Cell;

use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};
use oxc_traverse::TraverseCtx;

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
            let class_details = self.current_class_mut();
            class_details.bindings.currently_transforming_static_property_initializers = is_it;
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
/// Also:
/// * Updates parent `ScopeId` of first level of scopes in initializer.
/// * Sets `ScopeFlags` of scopes to sloppy mode if code outside the class is sloppy mode.
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
/// then we only need to transform `this`, and re-parent first-level scopes. So can skip traversing
/// into functions and other contexts which have their own `this`.
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
    /// Incremented when entering scope, decremented when exiting it.
    /// Parent `ScopeId` should be updated when `scope_depth == 0`.
    /// Note: `scope_depth` does not aim to track scope depth completely accurately.
    /// Only requirement is to ensure that `scope_depth == 0` only when we're in first-level scope.
    /// So we don't bother incrementing + decrementing for scopes which are definitely not first level.
    /// e.g. `BlockStatement` or `ForStatement` must be in a function, and therefore we're already in a
    /// nested scope.
    scope_depth: u32,
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
        let walk_deep = if make_sloppy_mode {
            true
        } else {
            let class_details = class_properties.current_class();
            class_details.bindings.name.is_some() || class_details.private_props.is_some()
        };

        Self { walk_deep, make_sloppy_mode, this_depth: 0, scope_depth: 0, class_properties, ctx }
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
            // `super.prop`
            Expression::StaticMemberExpression(_) => {
                self.class_properties.transform_static_member_expression(expr, self.ctx);
            }
            // `super[prop]`
            Expression::ComputedMemberExpression(_) => {
                self.class_properties.transform_computed_member_expression(expr, self.ctx);
            }
            // `object.#prop`
            Expression::PrivateFieldExpression(_) => {
                self.class_properties.transform_private_field_expression(expr, self.ctx);
            }
            // `super.prop()` or `object.#prop()`
            Expression::CallExpression(call_expr) => {
                self.class_properties
                    .transform_call_expression_for_super_member_expr(call_expr, self.ctx);
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

    /// Convert scope to sloppy mode if `self.make_sloppy_mode == true`.
    // `#[inline]` because called from many `walk` functions and is small.
    #[inline]
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        if self.make_sloppy_mode {
            let scope_id = scope_id.get().unwrap();
            *self.ctx.scopes_mut().get_flags_mut(scope_id) -= ScopeFlags::StrictMode;
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

        self.reparent_scope_if_first_level(&func.scope_id);

        if self.walk_deep {
            self.this_depth += 1;
            self.scope_depth += 1;
            walk_mut::walk_function(self, func, flags);
            self.this_depth -= 1;
            self.scope_depth -= 1;
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

        self.reparent_scope_if_first_level(&func.scope_id);

        self.scope_depth += 1;
        walk_mut::walk_arrow_function_expression(self, func);
        self.scope_depth -= 1;

        self.make_sloppy_mode = parent_sloppy_mode;
    }

    #[inline]
    fn visit_class(&mut self, class: &mut Class<'a>) {
        let parent_sloppy_mode = self.make_sloppy_mode;
        // Classes are always strict mode
        self.make_sloppy_mode = false;

        self.reparent_scope_if_first_level(&class.scope_id);

        self.scope_depth += 1;
        walk_mut::walk_class(self, class);
        self.scope_depth -= 1;

        self.make_sloppy_mode = parent_sloppy_mode;
    }

    #[inline]
    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        // Not possible that `self.scope_depth == 0` here, because a `StaticBlock`
        // can only be in a class, and that class would be the first-level scope.
        // So no need to call `reparent_scope_if_first_level`.

        // `walk_deep` must be `true` or we couldn't get here, because a `StaticBlock`
        // must be in a class, and traversal would have stopped in `visit_class` if it wasn't
        self.this_depth += 1;
        walk_mut::walk_static_block(self, block);
        self.this_depth -= 1;
    }

    #[inline]
    fn visit_ts_module_block(&mut self, block: &mut TSModuleBlock<'a>) {
        // Not possible that `self.scope_depth == 0` here, because a `TSModuleBlock`
        // can only be in a function, and that function would be the first-level scope.
        // So no need to call `reparent_scope_if_first_level`.

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

        // Not possible that `self.scope_depth == 0` here, because a `PropertyDefinition`
        // can only be in a class, and that class would be the first-level scope.
        // So no need to call `reparent_scope_if_first_level`.

        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }

        // `walk_deep` must be `true` or we couldn't get here, because a `PropertyDefinition`
        // must be in a class, and traversal would have stopped in `visit_class` if it wasn't
        if let Some(value) = &mut prop.value {
            self.this_depth += 1;
            self.visit_expression(value);
            self.this_depth -= 1;
        }
    }

    #[inline]
    fn visit_accessor_property(&mut self, prop: &mut AccessorProperty<'a>) {
        // Not possible that `self.scope_depth == 0` here, because an `AccessorProperty`
        // can only be in a class, and that class would be the first-level scope.
        // So no need to call `reparent_scope_if_first_level`.

        // Treat `key` and `value` in same way as `visit_property_definition` above.
        // TODO: Are decorators in scope?
        self.visit_decorators(&mut prop.decorators);
        if prop.computed {
            self.visit_property_key(&mut prop.key);
        }

        // `walk_deep` must be `true` or we couldn't get here, because an `AccessorProperty`
        // must be in a class, and traversal would have stopped in `visit_class` if it wasn't
        if let Some(value) = &mut prop.value {
            self.this_depth += 1;
            self.visit_expression(value);
            self.this_depth -= 1;
        }
    }

    // Remaining visitors are the only other types which have a scope which can be first-level
    // when starting traversal from an `Expression`.
    // `BlockStatement` and all other statements would need to be within a function,
    // and that function would be the first-level scope.

    #[inline]
    fn visit_ts_conditional_type(&mut self, conditional: &mut TSConditionalType<'a>) {
        self.reparent_scope_if_first_level(&conditional.scope_id);

        // `check_type` field is outside `TSConditionalType`'s scope
        self.visit_ts_type(&mut conditional.check_type);

        self.enter_scope(ScopeFlags::empty(), &conditional.scope_id);

        self.scope_depth += 1;
        self.visit_ts_type(&mut conditional.extends_type);
        self.visit_ts_type(&mut conditional.true_type);
        self.scope_depth -= 1;

        // `false_type` field is outside `TSConditionalType`'s scope
        self.visit_ts_type(&mut conditional.false_type);
    }

    #[inline]
    fn visit_ts_method_signature(&mut self, signature: &mut TSMethodSignature<'a>) {
        self.reparent_scope_if_first_level(&signature.scope_id);

        self.scope_depth += 1;
        walk_mut::walk_ts_method_signature(self, signature);
        self.scope_depth -= 1;
    }

    #[inline]
    fn visit_ts_construct_signature_declaration(
        &mut self,
        signature: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        self.reparent_scope_if_first_level(&signature.scope_id);

        self.scope_depth += 1;
        walk_mut::walk_ts_construct_signature_declaration(self, signature);
        self.scope_depth -= 1;
    }

    #[inline]
    fn visit_ts_mapped_type(&mut self, mapped: &mut TSMappedType<'a>) {
        self.reparent_scope_if_first_level(&mapped.scope_id);

        self.scope_depth += 1;
        walk_mut::walk_ts_mapped_type(self, mapped);
        self.scope_depth -= 1;
    }
}

impl<'a, 'ctx, 'v> StaticInitializerVisitor<'a, 'ctx, 'v> {
    /// Replace `this` with reference to temp var for class.
    fn replace_this_with_temp_var(&mut self, expr: &mut Expression<'a>, span: Span) {
        if self.this_depth == 0 {
            let class_details = self.class_properties.current_class_mut();
            let temp_binding = class_details.bindings.get_or_init_temp_binding(self.ctx);
            *expr = temp_binding.create_spanned_read_expression(span, self.ctx);
        }
    }

    /// Replace reference to class name with reference to temp var for class.
    fn replace_class_name_with_temp_var(&mut self, ident: &mut IdentifierReference<'a>) {
        // Check identifier is reference to class name
        let class_details = self.class_properties.current_class_mut();
        let class_name_symbol_id = class_details.bindings.name_symbol_id();
        let Some(class_name_symbol_id) = class_name_symbol_id else { return };

        let reference_id = ident.reference_id();
        let reference = self.ctx.symbols().get_reference(reference_id);
        let Some(symbol_id) = reference.symbol_id() else { return };

        if symbol_id != class_name_symbol_id {
            return;
        }

        // Identifier is reference to class name. Rename it.
        let temp_binding = class_details.bindings.get_or_init_temp_binding(self.ctx);
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

    /// Update parent of scope to scope above class if this is a first-level scope.
    fn reparent_scope_if_first_level(&mut self, scope_id: &Cell<Option<ScopeId>>) {
        if self.scope_depth == 0 {
            let scope_id = scope_id.get().unwrap();
            let current_scope_id = self.ctx.current_scope_id();
            self.ctx.scopes_mut().change_parent_id(scope_id, Some(current_scope_id));
        }
    }
}
