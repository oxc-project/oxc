//! Named evaluation of classes in decorated positions.

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_str::{JSStr, Str};
use oxc_traverse::Ancestor;

use crate::{
    common::{
        computed_key::create_computed_key_temp_var,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    utils::{class_name::contextual_class_name, decorated_class::class_lowered_for_decorators},
};

use super::StandardDecoratorState;

impl<'a> StandardDecoratorState<'a> {
    /// Name of a non-computed property key, for named evaluation of anonymous classes.
    pub(super) fn static_key_name(
        key: &PropertyKey<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<JSStr<'a>> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Some(ident.name.into()),
            PropertyKey::StringLiteral(lit) => Some(lit.value),
            PropertyKey::PrivateIdentifier(ident) => {
                Some(Str::from_strs_array_in(["#", &ident.name], ctx).into())
            }
            _ => None,
        }
    }

    /// Preserve named evaluation before a property initializer is moved or its key is rewritten.
    pub(crate) fn capture_class_initializer_name(
        &mut self,
        key: &mut PropertyKey<'a>,
        computed: bool,
        value: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ClassExpression(class) = value.without_parentheses() else { return };
        if !self.enabled || class.id.is_some() || !class_lowered_for_decorators(class) {
            return;
        }
        let name = if computed {
            let key_expr = key.as_expression_mut().unwrap();
            let value = helper_call_expr(
                Helper::ToPropertyKey,
                ArenaVec::from_value_in(Argument::from(key_expr.take_in(ctx)), ctx),
                ctx,
            );
            let (assignment, reference) = create_computed_key_temp_var(value, ctx);
            *key_expr = assignment;
            reference
        } else {
            Self::decorator_key_expr(key, ctx)
        };
        self.named_classes.insert(class.address(), name);
    }

    pub(crate) fn capture_object_property_class_name(
        &mut self,
        prop: &mut ObjectProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // A prototype setter does not perform named evaluation.
        if !prop.computed
            && Self::static_key_name(&prop.key, ctx).is_some_and(|name| name == "__proto__")
        {
            return;
        }
        self.capture_class_initializer_name(&mut prop.key, prop.computed, &prop.value, ctx);
    }

    /// Infer name of an anonymous class expression (NamedEvaluation), from its parent node.
    pub(crate) fn infer_class_expression_name(
        &mut self,
        class_address: Address,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if let Some(name) = self.named_classes.remove(&class_address) {
            return Some(name);
        }
        let parent = ctx
            .ancestors()
            .find(|ancestor| !matches!(ancestor, Ancestor::ParenthesizedExpressionExpression(_)))?;
        let name = match parent {
            Ancestor::PropertyDefinitionValue(prop) => {
                self.accessor_names.get(&parent.address()).copied().or_else(|| {
                    if *prop.computed() { None } else { Self::static_key_name(prop.key(), ctx) }
                })
            }
            _ => contextual_class_name(&parent).map(Into::into),
        };
        name.map(|name| Expression::new_string_literal(SPAN, name, None, ctx))
    }

    pub(crate) fn set_inferred_class_name(
        class: Expression<'a>,
        name: Option<Expression<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let Some(name) = name else { return class };
        helper_call_expr(
            Helper::SetClassName,
            ArenaVec::from_array_in([Argument::from(class), Argument::from(name)], ctx),
            ctx,
        )
    }
}
