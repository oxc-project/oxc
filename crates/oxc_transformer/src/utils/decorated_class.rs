//! Detect decorators and auto-accessors in class syntax.

use oxc_ast::ast::{Class, ClassBody, ClassElement};

/// Whether a runtime class has decorators on the class itself or any of its elements.
pub fn class_has_decorators(class: &Class<'_>) -> bool {
    !class.declare && (!class.decorators.is_empty() || class_body_has_decorators(&class.body))
}

/// Whether a class contains runtime auto-accessors.
pub fn class_body_has_auto_accessors(body: &ClassBody<'_>) -> bool {
    body.body.iter().any(|element| {
        matches!(element, ClassElement::AccessorProperty(prop) if !prop.r#type.is_abstract())
    })
}

/// Returns `true` if any class element is decorated.
pub fn class_body_has_decorators(body: &ClassBody<'_>) -> bool {
    body.body.iter().any(|element| match element {
        ClassElement::MethodDefinition(method) => !method.decorators.is_empty(),
        ClassElement::PropertyDefinition(prop) => !prop.decorators.is_empty(),
        ClassElement::AccessorProperty(prop) => !prop.decorators.is_empty(),
        ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => false,
    })
}
