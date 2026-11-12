//! Lowering of decorated `accessor` properties.

use oxc_allocator::{ArenaVec, GetAddress, TakeIn, UnstableAddress};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolFlags,
};
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::{
    common::{
        class_lowering::ClassLoweringMode,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    utils::ast_builder::wrap_expression_in_arrow_function_iife,
};

use super::{ClassDecoratorTables, StandardDecoratorState};

impl<'a> StandardDecoratorState<'a> {
    /// Keep computed accessor keys local to each evaluation of a parameter initializer.
    pub(crate) fn wrap_native_accessor_expression(
        &mut self,
        expression: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = expression else { return };
        if !self.enabled()
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
        // A class which lowers its own decorators and accessors wraps them when it is entered.
        if ctx.state.class_lowering(class).mode() != ClassLoweringMode::Native {
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
        let lowering = ctx.state.class_lowering(class);
        if lowering.mode() != ClassLoweringMode::Native
            || class.declare
            || class.id.is_some()
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
        mut tables: Option<&mut ClassDecoratorTables<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        super::super::accessor::lower_auto_accessors(
            body,
            class_scope_id,
            static_binding,
            native_fields,
            |storage, getter, key, name, ctx| {
                if !getter.decorators.is_empty()
                    && let Some(tables) = &mut tables
                {
                    tables.accessor_storage.insert(storage.address());
                    tables.accessor_getters.insert(getter.address());
                }
                if let Some(name) = name {
                    self.accessor_names.insert(storage.address(), name);
                }
                if !native_fields {
                    return;
                }
                let Some(init) = storage.value.as_mut() else { return };
                if let Expression::ClassExpression(class) = init.without_parentheses()
                    && class.id.is_none()
                    && !self.named_classes.contains_key(&class.address())
                {
                    let name = Self::decorator_key_expr(key, ctx);
                    self.named_classes.insert(class.address(), name);
                }
                let anonymous_function = match init.without_parentheses() {
                    Expression::FunctionExpression(function) => function.id.is_none(),
                    Expression::ArrowFunctionExpression(_) => true,
                    _ => false,
                };
                if anonymous_function {
                    let name = Self::decorator_key_expr(key, ctx);
                    *init = helper_call_expr(
                        Helper::SetFunctionName,
                        ArenaVec::from_array_in(
                            [Argument::from(init.take_in(ctx)), Argument::from(name)],
                            ctx,
                        ),
                        ctx,
                    );
                }
            },
            ctx,
        );
    }
}
