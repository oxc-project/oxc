use oxc_span::Span;

use crate::ast::*;

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
            PropertyKey::Identifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::PrivateIdentifier(_) => None,
            PropertyKey::Expression(expr) => match &expr {
                Expression::Identifier(ident) => Some((&ident.name, ident.span)),
                Expression::StringLiteral(lit) => Some((&lit.value, lit.span)),
                _ => None,
            },
        }
    }
}

impl<'a> PropName for ClassElement<'a> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ClassElement::MethodDefinition(def) => def.prop_name(),
            ClassElement::TSAbstractMethodDefinition(def) => def.method_definition.prop_name(),
            ClassElement::PropertyDefinition(def) => def.prop_name(),
            ClassElement::TSAbstractPropertyDefinition(def) => def.property_definition.prop_name(),
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
