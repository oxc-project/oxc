//! Named evaluation of classes in decorated positions.

use oxc_allocator::{Address, ArenaVec, GetAddress, TakeIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::Ancestor;

use crate::{
    common::{
        computed_key::create_computed_key_temp_var,
        helper_loader::{Helper, helper_call_expr},
    },
    context::TraverseCtx,
    decorator::accessor::static_key_name,
    utils::{ast_builder::create_to_property_key, class_name::contextual_class_name},
};

use super::StandardDecoratorState;

impl<'a> StandardDecoratorState<'a> {
    /// Preserve named evaluation before a property initializer is moved or its key is rewritten.
    pub(crate) fn capture_class_initializer_name(
        &mut self,
        key: &mut PropertyKey<'a>,
        value: &Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TypeScript wrappers are erased, so they do not prevent named evaluation.
        let Expression::ClassExpression(class) = value.get_inner_expression() else { return };
        if !self.enabled || class.id.is_some() || self.named_classes.contains_key(&class.address())
        {
            return;
        }
        let lowering = ctx.state.class_lowering_before_entry(class);
        if !lowering.needs_per_evaluation_state() {
            return;
        }
        let name = if key.is_computed() {
            let key_expr = key.as_expression_mut().unwrap();
            let value = create_to_property_key(key_expr.take_in(ctx), ctx);
            let (assignment, reference) = create_computed_key_temp_var(value, ctx);
            *key_expr = assignment;
            reference
        } else {
            Self::decorator_key_expr(key, ctx)
        };
        self.named_classes.insert(class.address(), name);
    }

    /// Capture the names of anonymous classes which are values of an object literal's properties,
    /// before any key is rewritten.
    pub(crate) fn capture_object_class_names(
        &mut self,
        object: &mut ObjectExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for prop in &mut object.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else { continue };
            let prop = &mut **prop;
            // Almost no property value is a class, so check that before the key.
            if !matches!(prop.value.get_inner_expression(), Expression::ClassExpression(_)) {
                continue;
            }
            // A prototype setter does not perform named evaluation.
            if prop.key.is_specific_literal_name("__proto__") {
                continue;
            }
            self.capture_class_initializer_name(&mut prop.key, &prop.value, ctx);
        }
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
        let parent = ctx.ancestors().find(|ancestor| {
            !matches!(
                ancestor,
                Ancestor::ParenthesizedExpressionExpression(_)
                    | Ancestor::TSAsExpressionExpression(_)
                    | Ancestor::TSSatisfiesExpressionExpression(_)
                    | Ancestor::TSNonNullExpressionExpression(_)
                    | Ancestor::TSTypeAssertionExpression(_)
            )
        })?;
        let name = match parent {
            Ancestor::PropertyDefinitionValue(prop) => {
                if prop.key().is_computed() {
                    None
                } else {
                    static_key_name(prop.key(), ctx)
                }
            }
            _ => contextual_class_name(&parent).map(Into::into),
        };
        name.map(|name| Expression::new_string_literal(SPAN, name, None, ctx))
    }

    pub(crate) fn set_inferred_class_name(
        class: Expression<'a>,
        name: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        helper_call_expr(
            Helper::SetClassName,
            ArenaVec::from_array_in([Argument::from(class), Argument::from(name)], ctx),
            ctx,
        )
    }
}
