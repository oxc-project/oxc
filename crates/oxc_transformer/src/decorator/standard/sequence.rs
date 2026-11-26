//! Emission of decorator calls and class initialization.

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::{SPAN, Span};
use oxc_str::Str;
use oxc_traverse::BoundIdentifier;

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    utils::ast_builder::{create_assignment, create_new_weakset},
};

use super::{
    ClassDecoratorBindings, ClassDecoratorState, DecoratedElement, DecoratorElementKind,
    FLAG_PRIVATE, FLAG_STATIC, PrivateDecoratorAccess, StandardDecoratorState,
};

impl<'a> StandardDecoratorState<'a> {
    /// Exit phase: `__decorateElement(_init, flags, key, _x_dec, Class)`
    pub(crate) fn push_decorate_element(
        &mut self,
        element: DecoratedElement<'_, 'a>,
        private: Option<&PrivateDecoratorAccess<'a>>,
        state: &ClassDecoratorState<'a>,
        target: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let DecoratedElement { kind, is_static, key, decorators, public_descriptor, span } =
            element;
        let flags = kind as u32
            | if is_static { FLAG_STATIC } else { 0 }
            | if private.is_some() { FLAG_PRIVATE } else { 0 };
        let init = &state.init;
        let target = target.create_read_expression(ctx);
        let decorators = match decorators {
            Some(binding) => binding.create_read_expression(ctx),
            None => Expression::new_array_expression(SPAN, ArenaVec::new_in(ctx), ctx),
        };
        let mut arguments = ArenaVec::from_array_in(
            [
                Argument::from(init.create_read_expression(ctx)),
                number(flags, ctx),
                Argument::from(key),
                Argument::from(decorators),
                Argument::from(target),
            ],
            ctx,
        );
        if let Some(index) = public_descriptor {
            let descriptors = crate::utils::ast_builder::create_property_access(
                SPAN,
                init.create_read_expression(ctx),
                "p",
                ctx,
            );
            let index = u32::try_from(index).unwrap();
            let index = Expression::new_numeric_literal(SPAN, f64::from(index), None, ctx);
            let descriptor = MemberExpression::new_computed_member_expression(
                SPAN,
                descriptors,
                index,
                false,
                ctx,
            );
            arguments.push(Argument::from(Expression::from(descriptor)));
        }
        let mut writebacks = Vec::new();
        if let Some(private) = private {
            let brand = &private.brand;
            if kind == DecoratorElementKind::Method {
                writebacks.push(("value", private.value.clone()));
            }
            if (kind == DecoratorElementKind::Getter || kind == DecoratorElementKind::Accessor)
                && let Some(getter) = &private.getter
            {
                writebacks.push(("get", getter.clone()));
            }
            if (kind == DecoratorElementKind::Setter || kind == DecoratorElementKind::Accessor)
                && let Some(setter) = &private.setter
            {
                writebacks.push(("set", setter.clone()));
            }
            let mut properties = ArenaVec::with_capacity_in(writebacks.len(), ctx);
            for (key, binding) in &writebacks {
                let key = PropertyKey::new_static_identifier(SPAN, Str::from_str_in(key, ctx), ctx);
                let value = binding.create_read_expression(ctx);
                properties.push(ObjectPropertyKind::new_object_property(
                    SPAN,
                    PropertyKind::Init,
                    key,
                    value,
                    false,
                    false,
                    ctx,
                ));
            }
            let descriptor = Expression::new_object_expression(SPAN, properties, ctx);
            let info = Expression::new_array_expression(
                SPAN,
                [
                    ArrayExpressionElement::from(brand.create_read_expression(ctx)),
                    ArrayExpressionElement::from(descriptor),
                ],
                ctx,
            );
            arguments.push(Argument::from(info));
        }
        let mut call =
            with_call_span(helper_call_expr(Helper::DecorateElement, arguments, ctx), span);
        if kind == DecoratorElementKind::Accessor && private.is_some() {
            let descriptor = ctx.generate_uid_in_current_hoist_scope("descriptor");
            ctx.state.var_declarations.insert_var(&descriptor, &ctx.ast);
            let mut expressions = ArenaVec::new_in(ctx);
            expressions.push(create_assignment(&descriptor, call, SPAN, ctx));
            for (key, binding) in writebacks {
                let value = crate::utils::ast_builder::create_property_access(
                    SPAN,
                    descriptor.create_read_expression(ctx),
                    key,
                    ctx,
                );
                expressions.push(create_assignment(&binding, value, SPAN, ctx));
            }
            call = Expression::new_sequence_expression(SPAN, expressions, ctx);
        } else if let Some((_, binding)) = writebacks.first() {
            call = create_assignment(binding, call, SPAN, ctx);
        }
        self.exit.elements[kind.bucket(is_static) as usize].push(call);
    }

    /// Exit phase: build the decorator application sequence which runs after the class definition,
    /// and the trailing class extra initializers.
    ///
    /// Returns `(before_static_members, after_static_members)`.
    pub(crate) fn build_decorator_sequence(
        &mut self,
        class: &Class<'a>,
        state: &ClassDecoratorState<'a>,
        bindings: &ClassDecoratorBindings<'_, 'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Vec<Expression<'a>>, Option<Expression<'a>>) {
        let init = &state.init;

        let mut exprs = Vec::new();

        // `_init = __decoratorStart(base)`
        let base = match self.exit.super_class.take() {
            Some(binding) => binding.create_read_expression(ctx),
            None => Expression::new_null_literal(SPAN, ctx),
        };
        let mut arguments = ArenaVec::from_value_in(Argument::from(base), ctx);
        if !self.exit.public_descriptors.is_empty() {
            arguments.push(Argument::from(bindings.target.create_read_expression(ctx)));
            let elements = Expression::new_array_expression(
                SPAN,
                ArenaVec::from_iter_in(
                    self.exit.public_descriptors.drain(..).map(ArrayExpressionElement::from),
                    ctx,
                ),
                ctx,
            );
            arguments.push(Argument::from(elements));
        }
        let start =
            with_call_span(helper_call_expr(Helper::DecoratorStart, arguments, ctx), class.span);
        exprs.push(create_assignment(init, start, SPAN, ctx));

        if let Some(brand) = bindings.static_brand {
            let weakset = create_new_weakset(ctx);
            exprs.push(create_assignment(brand, weakset, SPAN, ctx));
            let target = bindings.target.create_read_expression(ctx);
            let callee = crate::utils::ast_builder::create_property_access(
                SPAN,
                brand.create_read_expression(ctx),
                "add",
                ctx,
            );
            exprs.push(Expression::new_call_expression(
                SPAN,
                callee,
                None,
                [Argument::from(target)],
                false,
                ctx,
            ));
        }

        for bucket in &mut self.exit.elements {
            exprs.append(bucket);
        }

        let class_decorators = self.exit.class_decorators.take();
        if let Some(decorators) = &class_decorators {
            // `Class = __decorateElement(_init, 0, "Class", _Class_decorators, Class)`
            let inferred_name = self.exit.class_name.take();
            let name = match &class.id {
                Some(id) => Expression::new_string_literal(SPAN, id.name, None, ctx),
                None => inferred_name
                    .unwrap_or_else(|| Expression::new_string_literal(SPAN, "", None, ctx)),
            };
            let target = bindings.target.create_read_expression(ctx);
            let arguments = ArenaVec::from_array_in(
                [
                    Argument::from(init.create_read_expression(ctx)),
                    number(0, ctx),
                    Argument::from(name),
                    Argument::from(decorators.create_read_expression(ctx)),
                    Argument::from(target),
                ],
                ctx,
            );
            let mut call = with_call_span(
                helper_call_expr(Helper::DecorateElement, arguments, ctx),
                class.span,
            );

            let mut targets = Vec::with_capacity(2);
            if let Some(name) = bindings.declaration {
                targets.push(name.clone());
            }
            if let Some(temp) = bindings.temporary
                && targets.iter().all(|t| t.symbol_id != temp.symbol_id)
            {
                targets.push(temp.clone());
            }
            for target in targets {
                call = create_assignment(&target, call, SPAN, ctx);
            }
            exprs.push(call);
        } else {
            // `__decoratorMetadata(_init, Class)`
            let target = bindings.target.create_read_expression(ctx);
            let arguments = ArenaVec::from_array_in(
                [Argument::from(init.create_read_expression(ctx)), Argument::from(target)],
                ctx,
            );
            let metadata = helper_call_expr(Helper::DecoratorMetadata, arguments, ctx);
            exprs.push(with_call_span(metadata, class.span));
        }

        if state.static_method_extras {
            let target = bindings.target.create_read_expression(ctx);
            exprs.push(state.run_initializers_call(3, target, ctx));
        }

        let class_extras = class_decorators.map(|_| {
            let target = bindings.target.create_read_expression(ctx);
            state.run_initializers_call(1, target, ctx)
        });

        (exprs, class_extras)
    }
}

fn number<'a>(value: u32, ctx: &TraverseCtx<'a>) -> Argument<'a> {
    Argument::new_numeric_literal(SPAN, f64::from(value), None, ctx)
}

impl<'a> ClassDecoratorState<'a> {
    /// Wrap a decorated field's value and return its extra initializers.
    pub(crate) fn wrap_decorated_field(
        &self,
        value: &mut Option<Expression<'a>>,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        *value = Some(self.wrap_decorated_field_value(value.take(), index, ctx));
        self.decorated_field_extra_initializers(index, ctx)
    }

    /// `__runInitializers(_init, ((5 + 2 * index) << 1) | 1, this)`
    fn decorated_field_extra_initializers(
        &self,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.run_initializers_call(
            ((5 + 2 * index) << 1) | 1,
            Expression::new_this_expression(SPAN, ctx),
            ctx,
        )
    }

    /// `__runInitializers(_init, flags, target)`
    pub(crate) fn run_initializers_call(
        &self,
        flags: u32,
        target: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let init = &self.init;
        let arguments = ArenaVec::from_array_in(
            [
                Argument::from(init.create_read_expression(ctx)),
                number(flags, ctx),
                Argument::from(target),
            ],
            ctx,
        );
        helper_call_expr(Helper::RunInitializers, arguments, ctx)
    }

    /// `__runInitializers(_init, (4 + 2 * index) << 1, this, value)`
    fn wrap_decorated_field_value(
        &self,
        value: Option<Expression<'a>>,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let init = &self.init;
        let mut arguments = ArenaVec::with_capacity_in(4, ctx);
        arguments.push(Argument::from(init.create_read_expression(ctx)));
        arguments.push(number((4 + 2 * index) << 1, ctx));
        arguments.push(Argument::new_this_expression(SPAN, ctx));
        if let Some(value) = value {
            arguments.push(Argument::from(value));
        }
        helper_call_expr(Helper::RunInitializers, arguments, ctx)
    }
}

/// Map a generated helper call to `span`, so a stack trace through it points at the decorators.
fn with_call_span(mut call: Expression<'_>, span: Span) -> Expression<'_> {
    if let Expression::CallExpression(call) = &mut call {
        call.span = span;
    }
    call
}
