//! Classes whose fields class-properties does not lower.
//!
//! `ES2022` hands each class to one transform. Class-properties' transform hooks only run for the
//! classes whose fields it lowers. For every other class, `ES2022` calls these functions, which
//! keep the state that lowered classes and standard decorators read:
//!
//! * Native auto-accessors are lowered, and anonymous decorated classes in initializers are named.
//! * The class's private names are recorded on the class stack, so they shadow the same names of
//!   an enclosing lowered class.

use oxc_ast::ast::*;
use oxc_traverse::Ancestor;

use crate::{common::class_lowering::ClassLowering, context::TraverseCtx};

use super::{ClassBindings, ClassDetails, ClassProperties, IdentIndexMap, PrivateProps};

impl<'a> ClassProperties<'a> {
    /// Lower the auto-accessors of a native class.
    pub fn enter_unlowered_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        self.decorator_state.lower_native_auto_accessors(class, ctx);
    }

    /// Give a native class expression with computed auto-accessor keys its own scope.
    pub fn enter_unlowered_class_expression(
        &mut self,
        expr: &mut Expression<'a>,
        lowering: ClassLowering,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.decorator_state.wrap_native_accessor_expression(expr, lowering, ctx);
    }

    /// Capture names from the initializers, and record the class's private names.
    pub fn enter_unlowered_class_body(
        &mut self,
        body: &mut ClassBody<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // TS class declarations are not pushed onto the class stack
        let Ancestor::ClassBody(class) = ctx.parent() else { unreachable!() };
        if *class.declare() {
            return;
        }
        let is_declaration = *class.r#type() == ClassType::ClassDeclaration;
        debug_assert!(
            !ctx.state.class_lowering_of(class.scope_id().get().unwrap()).lowers_fields()
        );
        self.capture_initializer_names(body, ctx);

        // Register private names only when they can shadow a lowered name in an enclosing
        // class. Other unresolved names belong to untouched classes and are left as-is.
        let mut private_names = IdentIndexMap::default();
        if self.private_field_count != 0 {
            for element in &body.body {
                let key = match element {
                    ClassElement::PropertyDefinition(prop) => &prop.key,
                    ClassElement::MethodDefinition(method) => &method.key,
                    ClassElement::AccessorProperty(prop) => &prop.key,
                    _ => continue,
                };
                if let PropertyKey::PrivateIdentifier(ident) = key {
                    private_names.insert(ident.name, ());
                }
            }
        }
        self.classes_stack.push(ClassDetails {
            is_declaration,
            private_method_helpers_in_class_sequence: false,
            is_transform_required: false,
            private_props: PrivateProps::Native(private_names),
            bindings: ClassBindings::dummy(false),
            decorators: None,
        });
    }

    /// Pop the class's entry off the class stack.
    pub fn exit_unlowered_class(&mut self, class: &Class<'a>) {
        if class.declare {
            return;
        }
        let class_details = self.classes_stack.pop();
        debug_assert!(class_details.private_props.lowered().is_none());
    }
}
