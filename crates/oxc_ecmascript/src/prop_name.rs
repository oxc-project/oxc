use oxc_ast::ast::{
    ClassElement, MethodDefinition, ObjectProperty, ObjectPropertyKind, PropertyDefinition,
    PropertyKey, PropertyKind,
};
use oxc_span::Span;

/// [`PropName`](https://tc39.es/ecma262/#sec-static-semantics-propname)
pub trait PropName {
    fn prop_name(&self) -> Option<(&str, Span)>;
}

impl<'a> PropName for ObjectPropertyKind<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.prop_name(),
            ObjectPropertyKind::SpreadProperty(_) => None,
        }
    }
}

impl<'a> PropName for ObjectProperty<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.kind != PropertyKind::Init || self.method || self.shorthand || self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

impl<'a> PropName for PropertyKey<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            PropertyKey::StaticIdentifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::Identifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::StringLiteral(lit) => Some((&lit.value, lit.span)),
            _ => None,
        }
    }
}

impl<'a> PropName for ClassElement<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ClassElement::MethodDefinition(def) => def.prop_name(),
            ClassElement::PropertyDefinition(def) => def.prop_name(),
            _ => None,
        }
    }
}

impl<'a> PropName for MethodDefinition<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

impl<'a> PropName for PropertyDefinition<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.computed {
            return None;
        }
        self.key.prop_name()
    }
}
