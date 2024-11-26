//! ES2022: Class Properties
//! Transform of static property initializers.

use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::{BoundIdentifier, TraverseCtx};

use super::{ClassName, ClassProperties};

impl<'a, 'ctx> ClassProperties<'a, 'ctx> {
    /// Transform any `this` in static property initializer to reference to class name,
    /// and transform private field accesses (`object.#prop`).
    pub(super) fn transform_static_initializer(
        &mut self,
        value: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TODO: Insert temp var if class binding is mutated.

        let ClassName::Binding(class_name_binding) = &self.class_name else {
            // Binding is initialized in 1st pass in `transform_class` when a static prop is found
            unreachable!();
        };
        // Unfortunately have to clone, because also pass `&mut self` to `StaticInitializerVisitor::new`
        let class_name_binding = class_name_binding.clone();

        let mut replacer = StaticInitializerVisitor::new(class_name_binding, self, ctx);
        replacer.visit_expression(value);
    }
}

/// Visitor to transform:
///
/// 1. `this` to class name.
///    `class C { static x = this.y; }` -> `class C {}; C.x = C.y;`
/// 2. Private fields which refer to private props of this class.
///    `class C { static #x = 123; static.#y = this.#x; }`
///    -> `class C {}; var _x = { _: 123 }; _defineProperty(C, "y", _assertClassBrand(C, C, _x)._);`
///
/// Reason we need to do this is because the initializer is being moved from inside the class to outside.
/// `this` outside the class refers to a different `this`, and private fields are only valid within the
/// class body. So we need to transform them.
///
/// If this class defines no private properties, we only need to transform `this`, so can skip traversing
/// into functions and other contexts which have their own `this`.
///
/// Note: Those functions could contain private fields referring to a *parent* class's private props,
/// but we don't need to transform them here as they remain in same class scope.
//
// TODO: Also re-parent child scopes.
struct StaticInitializerVisitor<'a, 'ctx, 'v> {
    /// Binding for class name.
    class_name_binding: BoundIdentifier<'a>,
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
        class_name_binding: BoundIdentifier<'a>,
        class_properties: &'v mut ClassProperties<'a, 'ctx>,
        ctx: &'v mut TraverseCtx<'a>,
    ) -> Self {
        Self {
            class_name_binding,
            class_has_private_props: class_properties.private_props_stack.last().is_some(),
            this_depth: 0,
            class_properties,
            ctx,
        }
    }
}

impl<'a, 'ctx, 'v> VisitMut<'a> for StaticInitializerVisitor<'a, 'ctx, 'v> {
    #[inline]
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            // `this`
            Expression::ThisExpression(this_expr) => {
                let span = this_expr.span;
                self.replace_this_with_class_name(expr, span);
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
    /// Replace `this` with reference to class name binding.
    fn replace_this_with_class_name(&mut self, expr: &mut Expression<'a>, span: Span) {
        if self.this_depth == 0 {
            *expr = self.class_name_binding.create_spanned_read_expression(span, self.ctx);
        }
    }

    /// Replace `delete this` with `true`.
    fn replace_delete_this_with_true(&self, expr: &mut Expression<'a>, span: Span) {
        if self.this_depth == 0 {
            *expr = self.ctx.ast.expression_boolean_literal(span, true);
        }
    }
}
