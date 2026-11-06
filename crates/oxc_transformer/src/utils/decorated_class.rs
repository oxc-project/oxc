//! Whether class lowering has to run because of decorators.
//!
//! These predicates are pure questions about the AST shape of a class, so they live outside the
//! standard decorator module and can be used by the transforms which decide class ownership.

use oxc_ast::ast::{Class, ClassBody, ClassElement};

/// Returns `true` if class properties transform lowers this class because of standard decorators:
/// class is decorated, or contains `accessor` properties.
///
/// Other transforms (TypeScript) use this to know class fields of this class are handled by
/// class properties transform, even when the target supports class fields.
pub fn class_lowered_for_decorators(class: &Class<'_>) -> bool {
    !class.declare && class_body_lowered_for_decorators(!class.decorators.is_empty(), &class.body)
}

/// Same as [`class_lowered_for_decorators`], for use when only the class body is accessible.
pub fn class_body_lowered_for_decorators(has_class_decorators: bool, body: &ClassBody<'_>) -> bool {
    has_class_decorators
        || class_body_has_decorators(body)
        || body
            .body
            .iter()
            .any(|e| matches!(e, ClassElement::AccessorProperty(p) if !p.r#type.is_abstract()))
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
