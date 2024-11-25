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
    /// Transform any `this` in static property initializer to reference to class name.
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

        let mut replacer = StaticInitializerVisitor::new(class_name_binding, ctx);
        replacer.visit_expression(value);
    }
}

struct StaticInitializerVisitor<'a, 'v> {
    class_name_binding: &'v BoundIdentifier<'a>,
    ctx: &'v mut TraverseCtx<'a>,
}

impl<'a, 'v> StaticInitializerVisitor<'a, 'v> {
    fn new(class_name_binding: &'v BoundIdentifier<'a>, ctx: &'v mut TraverseCtx<'a>) -> Self {
        Self { class_name_binding, ctx }
    }
}

impl<'a, 'v> VisitMut<'a> for StaticInitializerVisitor<'a, 'v> {
    #[inline]
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::ThisExpression(this_expr) => {
                let span = this_expr.span;
                self.replace_this_with_class_name(expr, span);
                return;
            }
            Expression::UnaryExpression(unary_expr) => {
                if unary_expr.operator == UnaryOperator::Delete
                    && matches!(&unary_expr.argument, Expression::ThisExpression(_))
                {
                    let span = unary_expr.span;
                    self.replace_delete_this_with_true(expr, span);
                    return;
                }
            }
            _ => {}
        }

        walk_mut::walk_expression(self, expr);
    }

    // Stop traversing where scope of current `this` ends
    #[inline]
    fn visit_function(&mut self, _func: &mut Function<'a>, _flags: ScopeFlags) {}

    #[inline]
    fn visit_static_block(&mut self, _block: &mut StaticBlock) {}

    #[inline]
    fn visit_ts_module_block(&mut self, _block: &mut TSModuleBlock<'a>) {}

    #[inline]
    fn visit_property_definition(&mut self, prop: &mut PropertyDefinition<'a>) {
        // `this` in computed key of property or method refers to `this` of parent class.
        // So visit computed `key`, but not `value`.
        // ```js
        // class Outer {
        //   static prop = class Inner { [this] = 1; };
        // }
        // ```
        // Don't visit `type_annotation` field because can't contain `this`.
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

impl<'a, 'v> StaticInitializerVisitor<'a, 'v> {
    /// Replace `this` with reference to class name binding.
    fn replace_this_with_class_name(&mut self, expr: &mut Expression<'a>, span: Span) {
        *expr = self.class_name_binding.create_spanned_read_expression(span, self.ctx);
    }

    /// Replace `delete this` with `true`.
    fn replace_delete_this_with_true(&self, expr: &mut Expression<'a>, span: Span) {
        *expr = self.ctx.ast.expression_boolean_literal(span, true);
    }
}
