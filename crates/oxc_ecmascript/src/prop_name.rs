use oxc_ast::ast::{
    ClassElement, MethodDefinition, ObjectProperty, ObjectPropertyKind, PropertyDefinition,
    PropertyKey,
};
use oxc_span::Span;

/// [`PropName`](https://tc39.es/ecma262/#sec-static-semantics-propname)
pub trait PropName {
    fn prop_name(&self) -> Option<(&str, Span)>;
}

impl PropName for ObjectPropertyKind<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.prop_name(),
            ObjectPropertyKind::SpreadProperty(_) => None,
        }
    }
}

impl PropName for ObjectProperty<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.shorthand || self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

impl PropName for PropertyKey<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            PropertyKey::StaticIdentifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::Identifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::StringLiteral(lit) => Some((&lit.value, lit.span)),
            _ => None,
        }
    }
}

impl PropName for ClassElement<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            ClassElement::MethodDefinition(def) => def.prop_name(),
            ClassElement::PropertyDefinition(def) => def.prop_name(),
            _ => None,
        }
    }
}

impl PropName for MethodDefinition<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.computed {
            return None;
        }
        self.key.prop_name()
    }
}

impl PropName for PropertyDefinition<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.computed {
            return None;
        }
        self.key.prop_name()
    }
}
