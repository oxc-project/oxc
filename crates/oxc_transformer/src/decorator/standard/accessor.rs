//! Lowering of `accessor` properties.

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
        class_lowering::ClassLowering,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    utils::{
        ast_builder::wrap_expression_in_arrow_function_iife,
        decorated_class::{class_body_has_static_auto_accessor, is_auto_accessor},
    },
};

use super::StandardDecoratorState;
use crate::decorator::accessor::{AccessorKeys, GeneratedAccessor, lower_auto_accessors};

impl<'a> StandardDecoratorState<'a> {
    /// Keep computed accessor keys local to each evaluation of a parameter initializer.
    pub(crate) fn wrap_native_accessor_expression(
        &mut self,
        expression: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = expression else { return };
        if !self.enabled()
            || !class
                .body
                .body
                .iter()
                .any(|element| is_auto_accessor(element) && element.computed())
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
        if !ctx.state.class_lowering_before_entry(class).is_native() {
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
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // The caller only passes native classes.
        if class.id.is_some() {
            return;
        }
        // A static auto-accessor implies `has_auto_accessor`, so most classes skip the body scan.
        let needs_name = self.named_classes.contains_key(&class.unstable_address())
            || (lowering.has_auto_accessor() && class_body_has_static_auto_accessor(&class.body));
        if !needs_name {
            return;
        }

        let name = self
            .infer_class_expression_name(class.unstable_address(), ctx)
            .unwrap_or_else(|| Expression::new_string_literal(SPAN, "", None, ctx));
        let binding = ctx.generate_uid("class", class.scope_id(), SymbolFlags::Class);
        class.id = Some(binding.create_binding_identifier(ctx));
        let call =
            Self::set_inferred_class_name(Expression::new_this_expression(SPAN, ctx), name, ctx);
        let scope_id = ctx.create_child_scope(class.scope_id(), ScopeFlags::ClassStaticBlock);
        let block = ClassElement::new_static_block_with_scope_id(
            SPAN,
            ArenaVec::from_value_in(Statement::new_expression_statement(SPAN, call, ctx), ctx),
            scope_id,
            ctx,
        );
        class.body.body.insert(0, block);
    }

    /// Entry phase: lower plain auto-accessors of a class whose fields stay native.
    ///
    /// Runs on entering the class, before the standalone static-block transform scans its body,
    /// so that transform also lowers the static block inserted here.
    pub(crate) fn lower_native_auto_accessors(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let lowering = ctx.state.class_lowering_of(class.scope_id());
        if !lowering.is_native() || class.declare {
            return;
        }
        self.prepare_native_auto_accessors(class, lowering, ctx);
        if lowering.has_auto_accessor() {
            let binding = class.id.as_ref().map(BoundIdentifier::from_binding_ident);
            let scope_id = class.scope_id();
            self.lower_auto_accessors(&mut class.body, scope_id, binding.as_ref(), true, ctx);
        }
    }

    /// Entry phase: lower `accessor` properties to private storage + getter/setter pair.
    ///
    /// `accessor x = 1` -> `#_x_accessor_storage = 1; get x() { return this.#_x_accessor_storage; } set x(value) { ... }`
    pub(crate) fn lower_auto_accessors(
        &mut self,
        body: &mut ClassBody<'a>,
        class_scope_id: ScopeId,
        static_binding: Option<&BoundIdentifier<'a>>,
        native_fields: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let keys = if native_fields { AccessorKeys::Native } else { AccessorKeys::Lowered };
        lower_auto_accessors(
            body,
            class_scope_id,
            static_binding,
            keys,
            |GeneratedAccessor { storage, setter }, ctx| {
                if !native_fields {
                    return;
                }
                let key = &setter.key;
                let Some(init) = storage.value.as_mut() else { return };
                if let Expression::ClassExpression(class) = init.without_parentheses()
                    && class.id.is_none()
                    && !self.named_classes.contains_key(&class.address())
                    && !ctx.state.class_lowering_before_entry(class).is_preserved()
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
