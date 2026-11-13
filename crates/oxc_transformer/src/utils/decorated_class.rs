//! Queries about decorated class elements.

use oxc_ast::ast::{ClassBody, ClassElement};

/// Whether a class contains runtime auto-accessors.
pub fn class_body_has_auto_accessors(body: &ClassBody<'_>) -> bool {
    body.body.iter().any(|element| {
        matches!(element, ClassElement::AccessorProperty(prop) if !prop.r#type.is_abstract())
    })
}
