//! Detect decorators and auto-accessors in class syntax.

use oxc_ast::ast::{Class, ClassBody, ClassElement, PrivateIdentifier};
use oxc_ast_visit::VisitJs;
use oxc_str::Str;
use rustc_hash::FxHashSet;

use crate::context::TraverseCtx;

/// Reserve declarations and references, including uses of enclosing private names in nested classes.
#[derive(Clone, Default)]
pub struct PrivateNames<'a>(FxHashSet<&'a str>);

impl<'a> VisitJs<'a> for PrivateNames<'a> {
    fn visit_private_identifier(&mut self, ident: &PrivateIdentifier<'a>) {
        self.0.insert(ident.name.as_str());
    }
}

impl<'a> PrivateNames<'a> {
    pub fn generate(&mut self, base: Str<'a>, ctx: &TraverseCtx<'a>) -> Str<'a> {
        let mut name = base;
        let mut n = 2u32;
        while !self.0.insert(name.as_str()) {
            name = Str::from_strs_array_in([&base, itoa::Buffer::new().format(n)], ctx);
            n += 1;
        }
        name
    }
}

/// Whether a runtime class has decorators on the class itself or any of its elements.
pub fn class_has_decorators(class: &Class<'_>) -> bool {
    !class.declare && (!class.decorators.is_empty() || class_body_has_decorators(&class.body))
}

/// Whether a class contains runtime auto-accessors.
pub fn class_body_has_auto_accessors(body: &ClassBody<'_>) -> bool {
    body.body.iter().any(is_auto_accessor)
}

/// Whether a class element is a runtime auto-accessor.
pub fn is_auto_accessor(element: &ClassElement<'_>) -> bool {
    matches!(element, ClassElement::AccessorProperty(prop) if !prop.r#type.is_abstract())
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

/// Whether legacy decorators lower this class by decorating the class itself, which reassigns
/// the class binding to the value returned by `_decorate`.
///
/// That is the case for a runtime class declaration whose class or constructor parameters are
/// decorated.
pub fn legacy_decorators_reassign_class_binding(class: &Class<'_>) -> bool {
    if class.is_expression() || class.declare {
        return false;
    }
    !class.decorators.is_empty()
        || class.body.body.iter().any(|element| {
            matches!(element, ClassElement::MethodDefinition(method)
                if method.kind.is_constructor()
                    && method.value.params.items.iter().any(|param| !param.decorators.is_empty()))
        })
}
