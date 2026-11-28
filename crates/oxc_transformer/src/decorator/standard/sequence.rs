//! Emission of decorator calls and class initialization.

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn};
use oxc_ast::ast::*;
use oxc_span::{SPAN, Span};
use oxc_str::Str;
use oxc_syntax::symbol::SymbolFlags;
use oxc_traverse::BoundIdentifier;

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    utils::ast_builder::{
        create_assignment, create_export_class_reference_as, create_export_named_class_reference,
        create_let_declaration, create_new_weakset,
    },
};

use super::{
    CLASS_KIND, ClassDecoratorBindings, ClassDecoratorState, DeclarationRewrite, DecoratedElement,
    DecoratorElementKind, FLAG_PRIVATE, FLAG_STATIC, PrivateDecoratorAccess,
    StandardDecoratorState, run_initializers,
};

impl<'a> StandardDecoratorState<'a> {
    pub(crate) fn has_pending_declaration_rewrite(&self) -> bool {
        self.pending_declaration_rewrite.is_some()
    }

    /// Schedule the rewrite of a decorated class declaration which has just exited, for the exit
    /// of the statement containing it. No other statement exits in between.
    pub(crate) fn schedule_declaration_rewrite(
        &mut self,
        class: Address,
        (inner, outer): (BoundIdentifier<'a>, BoundIdentifier<'a>),
    ) {
        debug_assert!(self.pending_declaration_rewrite.is_none());
        self.pending_declaration_rewrite = Some(DeclarationRewrite { class, inner, outer });
    }

    /// Exit phase: convert a decorated class declaration whose name is referenced inside the class body.
    ///
    /// `class Foo {}` -> `let _Foo = class Foo {}; /* decorators */ let Foo = _Foo;`
    pub(crate) fn rewrite_decorated_class_statement(
        &mut self,
        stmt: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        enum Export {
            None,
            Named,
            Default,
        }
        let Some(DeclarationRewrite { class: class_address, inner, outer }) =
            self.pending_declaration_rewrite.take()
        else {
            return;
        };
        let (class, export) = match stmt {
            Statement::ClassDeclaration(class) => (class, Export::None),
            Statement::ExportDeclaration(export) => match &mut export.declaration {
                Declaration::ClassDeclaration(class) => (class, Export::Named),
                _ => unreachable!("a class declaration exits only inside a class statement"),
            },
            Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
                ExportDefaultDeclarationKind::ClassDeclaration(class) => (class, Export::Default),
                _ => unreachable!("a class declaration exits only inside a class statement"),
            },
            _ => unreachable!("a class declaration exits only inside a class statement"),
        };
        debug_assert_eq!(class.address(), class_address);

        // Class name becomes a binding in class scope. Outer binding becomes `let Foo`.
        let class_scope_id = class.scope_id();
        let id = class.id.as_mut().unwrap();
        let class_name =
            ctx.generate_spanned_binding(id.name, id.span, class_scope_id, SymbolFlags::Class);
        id.symbol_id.set(Some(class_name.symbol_id));
        *ctx.scoping_mut().symbol_flags_mut(outer.symbol_id) = SymbolFlags::BlockScopedVariable;

        class.r#type = ClassType::ClassExpression;
        let class_expr = Expression::ClassExpression(class.take_in_box(ctx));
        let new_stmt = create_let_declaration(&inner, class_expr, ctx);
        let outer_stmt = create_let_declaration(&outer, inner.create_read_expression(ctx), ctx);

        ctx.state.statement_injector.move_insertions(stmt, &new_stmt);
        ctx.state.statement_injector.insert_after(&new_stmt, outer_stmt);
        match export {
            Export::None => {}
            Export::Named => {
                let export = create_export_named_class_reference(&outer, ctx);
                ctx.state.statement_injector.insert_after(&new_stmt, export);
            }
            Export::Default => {
                // `export { Foo as default }` keeps the export live, as `export default class Foo`
                // does. `export default Foo` would snapshot the binding.
                let export = create_export_class_reference_as(&outer, Str::from("default"), ctx);
                ctx.state.statement_injector.insert_after(&new_stmt, export);
            }
        }
        *stmt = new_stmt;
    }

    /// Exit phase: `decorateElement(_init, flags, key, _x_dec, Class)`
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
        if let Some(original_class) = &state.original_class {
            let class = bindings.target.create_read_expression(ctx);
            exprs.push(create_assignment(original_class, class, SPAN, ctx));
        }

        // `_init = decoratorStart(base)`
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
            // `Class = decorateElement(_init, 0, "Class", _Class_decorators, Class)`
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
                    number(CLASS_KIND, ctx),
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
            // `decoratorMetadata(_init, Class)`
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
            exprs.push(state.run_initializers_call(
                run_initializers::STATIC_METHOD_EXTRAS,
                target,
                ctx,
            ));
        }

        let class_extras = class_decorators.map(|_| {
            let target = bindings.target.create_read_expression(ctx);
            state.run_initializers_call(run_initializers::CLASS_EXTRAS, target, ctx)
        });

        (exprs, class_extras)
    }
}

fn number<'a>(value: u32, ctx: &TraverseCtx<'a>) -> Argument<'a> {
    Argument::new_numeric_literal(SPAN, f64::from(value), None, ctx)
}

impl<'a> ClassDecoratorState<'a> {
    /// Reserve a binding for the original constructor used in lowered `super` accesses.
    ///
    /// Decorator emission assigns it before any class decorator can replace the constructor.
    pub(crate) fn get_or_init_original_class_for_super(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
    ) -> &BoundIdentifier<'a> {
        self.original_class.get_or_insert_with(|| {
            let binding = ctx.generate_uid(
                "originalClass",
                ctx.current_block_scope_id(),
                SymbolFlags::BlockScopedVariable,
            );
            ctx.state.var_declarations.insert_let(&binding, None, &ctx.ast);
            binding
        })
    }

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

    /// `runInitializers(_init, run_initializers::element_extras(index), this)`
    fn decorated_field_extra_initializers(
        &self,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        self.run_initializers_call(
            run_initializers::element_extras(index),
            Expression::new_this_expression(SPAN, ctx),
            ctx,
        )
    }

    /// `runInitializers(_init, flags, target)`
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

    /// `runInitializers(_init, run_initializers::element_initializers(index), this, value)`
    fn wrap_decorated_field_value(
        &self,
        value: Option<Expression<'a>>,
        index: u32,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let init = &self.init;
        let mut arguments = ArenaVec::with_capacity_in(4, ctx);
        arguments.push(Argument::from(init.create_read_expression(ctx)));
        arguments.push(number(run_initializers::element_initializers(index), ctx));
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
