//! Emission of decorator calls and class initialization.

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{number::NumberBase, symbol::SymbolFlags};
use oxc_traverse::BoundIdentifier;

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
    utils::ast_builder::{
        create_assignment, create_export_default_class_reference,
        create_export_named_class_reference, create_new_weakset, create_variable_declaration,
    },
};

use super::{
    ClassDecoratorBindings, ClassDecoratorState, DecoratorElementKind, FLAG_PRIVATE, FLAG_STATIC,
    PrivateDecoratorAccess, StandardDecoratorState,
};

impl<'a> StandardDecoratorState<'a> {
    pub(crate) fn has_declaration_rewrites(&self) -> bool {
        !self.declaration_rewrites.is_empty()
    }

    pub(crate) fn record_declaration_rewrite(
        &mut self,
        class: Address,
        bindings: (BoundIdentifier<'a>, BoundIdentifier<'a>),
    ) {
        self.declaration_rewrites.insert(class, bindings);
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
        let (class, export) = match stmt {
            Statement::ClassDeclaration(class) => (class, Export::None),
            Statement::ExportDeclaration(export) => match &mut export.declaration {
                Declaration::ClassDeclaration(class) => (class, Export::Named),
                _ => return,
            },
            Statement::ExportDefaultDeclaration(export) => match &mut export.declaration {
                ExportDefaultDeclarationKind::ClassDeclaration(class) => (class, Export::Default),
                _ => return,
            },
            _ => return,
        };
        let Some((inner, outer)) = self.declaration_rewrites.remove(&class.address()) else {
            return;
        };

        // Class name becomes a binding in class scope. Outer binding becomes `let Foo`.
        let class_scope_id = class.scope_id();
        let id = class.id.as_mut().unwrap();
        let class_name =
            ctx.generate_spanned_binding(id.name, id.span, class_scope_id, SymbolFlags::Class);
        id.symbol_id.set(Some(class_name.symbol_id));
        *ctx.scoping_mut().symbol_flags_mut(outer.symbol_id) = SymbolFlags::BlockScopedVariable;

        class.r#type = ClassType::ClassExpression;
        let class_expr = Expression::ClassExpression(class.take_in_box(ctx));
        let new_stmt = Self::create_let(&inner, class_expr, ctx);
        let outer_stmt = Self::create_let(&outer, inner.create_read_expression(ctx), ctx);

        ctx.state.statement_injector.move_insertions(stmt, &new_stmt);
        ctx.state.statement_injector.insert_after(&new_stmt, outer_stmt);
        match export {
            Export::None => {}
            Export::Named => {
                let export = create_export_named_class_reference(&outer, ctx);
                ctx.state.statement_injector.insert_after(&new_stmt, export);
            }
            Export::Default => {
                let export = create_export_default_class_reference(&outer, ctx);
                ctx.state.statement_injector.insert_after(&new_stmt, export);
            }
        }
        *stmt = new_stmt;
    }

    pub(crate) fn create_let(
        binding: &BoundIdentifier<'a>,
        init: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        let declarator = VariableDeclarator::new(
            SPAN,
            binding
                .create_spanned_binding_pattern(ctx.scoping().symbol_span(binding.symbol_id), ctx),
            None,
            Some(init),
            false,
            ctx,
        );
        Statement::from(Declaration::new_variable_declaration(
            SPAN,
            VariableDeclarationKind::Let,
            [declarator],
            false,
            ctx,
        ))
    }

    /// Exit phase: `__decorateElement(_init, flags, key, _x_dec, Class)`
    pub(crate) fn push_decorate_element(
        &mut self,
        kind: DecoratorElementKind,
        is_static: bool,
        key: Expression<'a>,
        decorators: &BoundIdentifier<'a>,
        private: Option<&PrivateDecoratorAccess<'a>>,
        state: &ClassDecoratorState<'a>,
        target: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let flags = kind as u32
            | if is_static { FLAG_STATIC } else { 0 }
            | if private.is_some() { FLAG_PRIVATE } else { 0 };
        let init = &state.init;
        let target = target.create_read_expression(ctx);
        let mut arguments = ArenaVec::from_array_in(
            [
                Argument::from(init.create_read_expression(ctx)),
                number(flags, ctx),
                Argument::from(key),
                Argument::from(decorators.create_read_expression(ctx)),
                Argument::from(target),
            ],
            ctx,
        );
        let mut writebacks = Vec::new();
        if let Some(private) = private {
            let brand = &private.brand;
            let getter = private.getter.clone();
            let setter = private.setter.clone();
            if kind == DecoratorElementKind::Method {
                writebacks.push(("value", private.value.clone()));
            }
            if (kind == DecoratorElementKind::Getter || kind == DecoratorElementKind::Accessor)
                && let Some(getter) = getter
            {
                writebacks.push(("get", getter));
            }
            if (kind == DecoratorElementKind::Setter || kind == DecoratorElementKind::Accessor)
                && let Some(setter) = setter
            {
                writebacks.push(("set", setter));
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
        let mut call = helper_call_expr(Helper::DecorateElement, arguments, ctx);
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
        let init = state.init.clone();
        let static_method_extras = state.static_method_extras;

        let mut exprs = Vec::new();

        // `_init = __decoratorStart(base)`
        let base = match &self.exit.super_class.take() {
            Some(binding) => binding.create_read_expression(ctx),
            None => Expression::new_null_literal(SPAN, ctx),
        };
        let start = helper_call_expr(
            Helper::DecoratorStart,
            ArenaVec::from_value_in(Argument::from(base), ctx),
            ctx,
        );
        exprs.push(create_assignment(&init, start, SPAN, ctx));

        for bucket in &mut self.exit.elements {
            exprs.append(bucket);
        }

        if let Some(brand) = bindings.static_brand {
            let weakset = create_new_weakset(ctx);
            exprs.insert(1, create_assignment(brand, weakset, SPAN, ctx));
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
            let mut call = helper_call_expr(Helper::DecorateElement, arguments, ctx);

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
            exprs.push(helper_call_expr(Helper::DecoratorMetadata, arguments, ctx));
        }

        if static_method_extras {
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
    Argument::new_numeric_literal(SPAN, f64::from(value), None, NumberBase::Decimal, ctx)
}

impl<'a> ClassDecoratorState<'a> {
    /// Declare a class-scoped temporary.
    ///
    /// A decorated class uses `let`, because a class decorator can replace the binding.
    pub(crate) fn declare_binding(
        decorators: Option<&ClassDecoratorState<'a>>,
        binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if decorators.is_some() {
            ctx.state.var_declarations.insert_let(binding, None, &ctx.ast);
        } else {
            ctx.state.var_declarations.insert_var(binding, &ctx.ast);
        }
    }

    /// Declare and initialize a class-scoped temporary, using `let` for a decorated class.
    pub(crate) fn create_binding_declaration(
        decorators: Option<&ClassDecoratorState<'a>>,
        binding: &BoundIdentifier<'a>,
        init: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        if decorators.is_some() {
            StandardDecoratorState::create_let(binding, init, ctx)
        } else {
            create_variable_declaration(binding, init, ctx)
        }
    }

    /// `__runInitializers(_init, ((5 + 2 * index) << 1) | 1, this)`
    pub(crate) fn decorated_field_extra_initializers(
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
    pub(crate) fn wrap_decorated_field_value(
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
