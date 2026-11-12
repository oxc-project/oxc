//! Lowering of decorated `accessor` properties.

use std::mem;

use rustc_hash::FxHashSet;

use oxc_allocator::{ArenaVec, GetAddress, TakeIn, UnstableAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::Str;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::{
        computed_key::create_computed_key_temp_var,
        duplicate::duplicate_expression,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    utils::{
        ast_builder::{create_accessor_method, wrap_expression_in_arrow_function_iife},
        decorated_class::{class_body_has_auto_accessors, class_lowered_for_decorators},
    },
};

use super::StandardDecoratorState;

impl<'a> StandardDecoratorState<'a> {
    /// Keep computed accessor keys local to each evaluation of a parameter initializer.
    pub(crate) fn wrap_native_accessor_expression(
        &mut self,
        expression: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = expression else { return };
        if !self.enabled()
            || class_lowered_for_decorators(class)
            || !class.body.body.iter().any(|element| {
                matches!(element, ClassElement::AccessorProperty(prop) if prop.computed && !prop.r#type.is_abstract())
            })
            || !ctx
                .ancestors()
                .take_while(|ancestor| {
                    !matches!(
                        ancestor,
                        Ancestor::FunctionBodyStatements(_)
                            | Ancestor::ArrowFunctionExpressionBody(_)
                    )
                })
                .any(|ancestor| matches!(ancestor, Ancestor::FormalParameterInitializer(_)))
        {
            return;
        }
        if class.id.is_none()
            && let Some(name) = self.infer_class_expression_name(class.unstable_address(), ctx)
        {
            self.named_classes.insert(class.unstable_address(), name);
        }
        *expression = wrap_expression_in_arrow_function_iife(expression.take_in(ctx), ctx);
    }

    /// Give native static accessors a stable class-local receiver.
    ///
    /// An anonymous class needs an internal name for its generated getter and setter. Restore its
    /// contextual name before user static initializers can observe it.
    pub(crate) fn prepare_native_auto_accessors(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.enabled()
            || class.declare
            || class.id.is_some()
            || class_lowered_for_decorators(class)
            || (!self.named_classes.contains_key(&class.unstable_address())
                && !class.body.body.iter().any(|element| {
                    matches!(element, ClassElement::AccessorProperty(prop) if prop.r#static && !prop.r#type.is_abstract())
                }))
        {
            return;
        }

        let name = self
            .infer_class_expression_name(class.unstable_address(), ctx)
            .unwrap_or_else(|| Expression::new_string_literal(SPAN, "", None, ctx));
        let binding = ctx.generate_uid("class", class.scope_id(), SymbolFlags::Class);
        class.id = Some(binding.create_binding_identifier(ctx));
        let call = helper_call_expr(
            Helper::SetClassName,
            ArenaVec::from_array_in(
                [Argument::from(Expression::new_this_expression(SPAN, ctx)), Argument::from(name)],
                ctx,
            ),
            ctx,
        );
        let scope_id = ctx.create_child_scope(class.scope_id(), ScopeFlags::ClassStaticBlock);
        let block = ClassElement::new_static_block_with_scope_id(
            SPAN,
            ArenaVec::from_value_in(Statement::new_expression_statement(SPAN, call, ctx), ctx),
            scope_id,
            ctx,
        );
        class.body.body.insert(0, block);
    }

    /// Entry phase: lower `accessor` properties to private storage + getter/setter pair.
    ///
    /// `@dec accessor x = 1` -> `#_x_accessor_storage = 1; @dec get x() { return this.#_x_accessor_storage; } set x(value) { ... }`
    ///
    /// Decorators move to the getter, which is recorded so it's decorated with `accessor` kind.
    pub(crate) fn lower_auto_accessors(
        &mut self,
        body: &mut ClassBody<'a>,
        class_scope_id: ScopeId,
        static_binding: Option<&BoundIdentifier<'a>>,
        native_fields: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !class_body_has_auto_accessors(body) {
            return;
        }

        // Private names are matched by name, so generated storage names must not clash
        // with existing private names, or with each other (`accessor x` + `static accessor x`).
        let mut used_names: FxHashSet<&str> = body
            .body
            .iter()
            .filter_map(|element| match element {
                ClassElement::PropertyDefinition(p) => p.key.private_name(),
                ClassElement::MethodDefinition(m) => m.key.private_name(),
                ClassElement::AccessorProperty(p) => p.key.private_name(),
                _ => None,
            })
            .map(|name| name.as_str())
            .collect();
        let mut unique_name = |base: Str<'a>, ctx: &TraverseCtx<'a>| -> Str<'a> {
            let mut name = base;
            let mut n = 2u32;
            while used_names.contains(name.as_str()) {
                name = Str::from_strs_array_in([&base, &n.to_string()], ctx);
                n += 1;
            }
            used_names.insert(name.as_str());
            name
        };

        let mut new_body = ArenaVec::with_capacity_in(body.body.len() + 4, ctx);
        let mut counter = 0u32;
        for element in body.body.drain(..) {
            let ClassElement::AccessorProperty(accessor) = element else {
                new_body.push(element);
                continue;
            };
            if accessor.r#type.is_abstract() {
                new_body.push(ClassElement::AccessorProperty(accessor));
                continue;
            }

            let mut accessor = accessor.unbox();
            let is_static = accessor.r#static;
            let mut computed = accessor.computed;
            let decorators = mem::replace(&mut accessor.decorators, ArenaVec::new_in(ctx));
            let is_decorated = !decorators.is_empty();

            let original_name = Self::static_key_name(&accessor.key, ctx);
            let (storage_name, getter_key, setter_key) = match accessor.key {
                PropertyKey::StaticIdentifier(ident) => {
                    let name =
                        Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                    let key = PropertyKey::StaticIdentifier(ident);
                    let key2 = PropertyKey::new_static_identifier(
                        SPAN,
                        match &key {
                            PropertyKey::StaticIdentifier(ident) => ident.name,
                            _ => unreachable!(),
                        },
                        ctx,
                    );
                    (name, key, key2)
                }
                PropertyKey::PrivateIdentifier(ident) => {
                    let name =
                        Str::from_strs_array_in(["_", &ident.name, "_accessor_storage"], ctx);
                    let key2 = PropertyKey::new_private_identifier(SPAN, ident.name, ctx);
                    (name, PropertyKey::PrivateIdentifier(ident), key2)
                }
                key => {
                    let counter_str = counter.to_string();
                    counter += 1;
                    let name =
                        Str::from_strs_array_in(["_", &counter_str, "_accessor_storage"], ctx);
                    let mut key_expr = key.into_expression();
                    if native_fields {
                        computed = true;
                        key_expr = helper_call_expr(
                            Helper::ToPropertyKey,
                            ArenaVec::from_value_in(Argument::from(key_expr), ctx),
                            ctx,
                        );
                    }
                    let (assignment, reference) = if native_fields {
                        create_computed_key_temp_var(key_expr, ctx)
                    } else {
                        duplicate_expression(key_expr, true, ctx)
                    };
                    (name, PropertyKey::from(assignment), PropertyKey::from(reference))
                }
            };

            let storage_name = unique_name(storage_name, ctx);
            let mut value = accessor.value.take();
            if native_fields && let Some(init) = value.as_mut() {
                if let Expression::ClassExpression(class) = init.without_parentheses()
                    && class.id.is_none()
                    && !self.named_classes.contains_key(&class.address())
                {
                    let name = Self::decorator_key_expr(&setter_key, ctx);
                    self.named_classes.insert(class.address(), name);
                }
                let anonymous_function = match init.without_parentheses() {
                    Expression::FunctionExpression(function) => function.id.is_none(),
                    Expression::ArrowFunctionExpression(_) => true,
                    _ => false,
                };
                if anonymous_function {
                    let name = Self::decorator_key_expr(&setter_key, ctx);
                    *init = helper_call_expr(
                        Helper::SetFunctionName,
                        ArenaVec::from_array_in(
                            [Argument::from(init.take_in(ctx)), Argument::from(name)],
                            ctx,
                        ),
                        ctx,
                    );
                }
            }

            // `#_x_accessor_storage = value`
            let storage = ClassElement::new_property_definition(
                SPAN,
                PropertyDefinitionType::PropertyDefinition,
                [],
                PropertyKey::new_private_identifier(SPAN, storage_name, ctx),
                None,
                value,
                false,
                is_static,
                false,
                false,
                false,
                false,
                false,
                None,
                ctx,
            );
            if let ClassElement::PropertyDefinition(prop) = &storage {
                if is_decorated {
                    self.accessor_storage.insert(prop.address());
                }
                if let Some(name) = &original_name {
                    self.accessor_names.insert(prop.address(), *name);
                }
            }
            new_body.push(storage);

            let getter = create_accessor_method(
                decorators,
                getter_key,
                MethodDefinitionKind::Get,
                computed,
                is_static,
                storage_name,
                if is_static { static_binding } else { None },
                class_scope_id,
                None,
                ctx,
            );
            if is_decorated && let ClassElement::MethodDefinition(method) = &getter {
                self.accessor_getters.insert(method.address());
            }
            new_body.push(getter);

            new_body.push(create_accessor_method(
                ArenaVec::new_in(ctx),
                setter_key,
                MethodDefinitionKind::Set,
                computed,
                is_static,
                storage_name,
                if is_static { static_binding } else { None },
                class_scope_id,
                None,
                ctx,
            ));
        }
        body.body = new_body;
    }
}
