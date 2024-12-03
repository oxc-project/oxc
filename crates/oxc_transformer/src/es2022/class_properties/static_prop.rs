//! ES2022: Class Properties
//! Transform of static property initializers.

use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::TraverseCtx;

use super::ClassProperties;

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform static property initializer.
    ///
    /// Replace `this` with temp var for class. Transform private fields.
    /// See below for full details of transforms.
    pub(super) fn transform_static_initializer(
        &mut self,
        value: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Replace references to class name with temp var

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
///      -> `var _C; x = (_C = class C {}, _C.x = _C.y)`
/// 2. Private fields which refer to private props of this class.
///    * Class declaration: `class C { static #x = 123; static y = this.#x; }`
///      -> `var _C; class C {}; _C = C; var _x = { _: 123 }; C.y = _assertClassBrand(_C, _C, _x)._;`
///    * Class expression: `x = class C { static #x = 123; static y = this.#x; }`
///      -> `var _C, _x; x = (_C = class C {}, _x = { _: 123 }, _C.y = _assertClassBrand(_C, _C, _x)._), _C)`
///
/// Reason we need to do this is because the initializer is being moved from inside the class to outside.
/// `this` outside the class refers to a different `this`, and private fields are only valid within the
/// class body. So we need to transform them.
///
/// Note that for class declarations, assignments are made to properties of original class name `C`,
/// but temp var `_C` is used in replacements for `this` and private fields. This is because class binding
/// `C` can be mutated, and the initializer may contain functions which are not executed immediately.
///
/// If this class defines no private properties, we only need to transform `this`, so can skip traversing
/// into functions and other contexts which have their own `this`.
///
/// Note: Those functions could contain private fields referring to a *parent* class's private props,
/// but we don't need to transform them here as they remain in same class scope.
//
// TODO: Also re-parent child scopes.
struct StaticInitializerVisitor<'a, 'ctx, 'v> {
    /// `true` if class has private properties.
    class_has_private_props: bool,
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
        Self {
            class_has_private_props: class_properties.private_props_stack.last().is_some(),
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
            // `delete this`
            Expression::UnaryExpression(unary_expr) => {
                if unary_expr.operator == UnaryOperator::Delete
                    && matches!(&unary_expr.argument, Expression::ThisExpression(_))
                {
                    let span = unary_expr.span;
                    self.replace_delete_this_with_true(expr, span);
                    return;
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
        self.class_properties.transform_assignment_target(target, self.ctx);
        walk_mut::walk_assignment_target(self, target);
    }

    // Increment `this_depth` when entering code where `this` refers to a different `this`
    // from `this` within this class, and decrement it when exiting.
    // Therefore `this_depth == 0` when `this` refers to the `this` which needs to be transformed.
    //
    // Or, if class has no private properties, stop traversing entirely. No private field accesses
    // need to be transformed, so no point searching for them.
    #[inline]
    fn visit_function(&mut self, func: &mut Function<'a>, flags: ScopeFlags) {
        if self.class_has_private_props {
            self.this_depth += 1;
            walk_mut::walk_function(self, func, flags);
            self.this_depth -= 1;
        }
    }

    #[inline]
    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        if self.class_has_private_props {
            self.this_depth += 1;
            walk_mut::walk_static_block(self, block);
            self.this_depth -= 1;
        }
    }

    #[inline]
    fn visit_ts_module_block(&mut self, block: &mut TSModuleBlock<'a>) {
        if self.class_has_private_props {
            self.this_depth += 1;
            walk_mut::walk_ts_module_block(self, block);
            self.this_depth -= 1;
        }
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

        if self.class_has_private_props {
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

        if self.class_has_private_props {
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
            // `PrivateProps` is the source of truth for bindings if class has private props
            // because other visitors which transform private fields may create a temp binding
            // and store it on `PrivateProps`
            let class_bindings = match self.class_properties.private_props_stack.last_mut() {
                Some(private_props) => &mut private_props.class_bindings,
                None => &mut self.class_properties.class_bindings,
            };

            let temp_binding = class_bindings.get_or_init_temp_binding(self.ctx);
            *expr = temp_binding.create_spanned_read_expression(span, self.ctx);
        }
    }

    /// Replace `delete this` with `true`.
    fn replace_delete_this_with_true(&self, expr: &mut Expression<'a>, span: Span) {
        if self.this_depth == 0 {
            *expr = self.ctx.ast.expression_boolean_literal(span, true);
        }
    }
}
