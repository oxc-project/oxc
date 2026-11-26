//! Integration of standard auto-accessors with class bindings and class-properties lowering.

use oxc_allocator::UnstableAddress;
use oxc_ast::ast::*;
use oxc_traverse::{Ancestor, BoundIdentifier};

use crate::context::TraverseCtx;

use super::{ClassBindings, ClassProperties};

/// Auto-accessor setup produced before the class is pushed onto the class stack.
pub(super) struct DecoratorBodyEntry<'a> {
    pub static_accessor_binding: Option<BoundIdentifier<'a>>,
    pub has_auto_accessor: bool,
}

impl DecoratorBodyEntry<'_> {
    /// Lowered auto-accessors also own their static blocks.
    pub fn lowers_static_blocks(&self) -> bool {
        self.has_auto_accessor
    }
}

impl<'a> ClassProperties<'a> {
    /// Capture names before rewriting keys, then lower this class's auto-accessors.
    pub(super) fn on_body_entry(
        &mut self,
        body: &mut ClassBody<'a>,
        class_name_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> DecoratorBodyEntry<'a> {
        let Ancestor::ClassBody(class) = ctx.parent() else { unreachable!() };
        let class_scope_id = class.scope_id().get().unwrap();
        let lowering = ctx.state.class_lowering_of(class_scope_id);
        let standard = self.decorator_state.enabled();
        let has_auto_accessor = standard && lowering.has_auto_accessor();
        if standard {
            for element in &mut body.body {
                let (key, value) = match element {
                    ClassElement::PropertyDefinition(prop) => {
                        let prop = &mut **prop;
                        (&mut prop.key, &prop.value)
                    }
                    ClassElement::AccessorProperty(prop) => {
                        let prop = &mut **prop;
                        (&mut prop.key, &prop.value)
                    }
                    _ => continue,
                };
                if let Some(value) = value {
                    self.decorator_state.capture_class_initializer_name(key, value, ctx);
                }
            }
        }
        if !lowering.lowers_fields() {
            return DecoratorBodyEntry { static_accessor_binding: None, has_auto_accessor };
        }
        let static_accessor_binding = (has_auto_accessor
            && body.body.iter().any(|element| {
                matches!(element, ClassElement::AccessorProperty(prop) if prop.r#static && !prop.r#type.is_abstract())
            }))
        .then(|| {
            ClassBindings::create_temp_binding(class_name_binding, ctx.current_hoist_scope_id(), ctx)
        });
        if has_auto_accessor {
            self.decorator_state.lower_auto_accessors(
                body,
                class_scope_id,
                static_accessor_binding.as_ref(),
                false,
                ctx,
            );
        }
        DecoratorBodyEntry { static_accessor_binding, has_auto_accessor }
    }

    /// Infer the name of an anonymous class expression whose auto-accessors were lowered.
    pub(super) fn on_class_exit(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let details = self.current_class();
        let inferred_name =
            if !details.is_declaration && class.id.is_none() && details.has_auto_accessor {
                self.decorator_state.infer_class_expression_name(class.unstable_address(), ctx)
            } else {
                None
            };
        self.transform_class_elements(class, ctx);
        inferred_name
    }
}
