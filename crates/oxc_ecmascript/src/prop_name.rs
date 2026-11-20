use oxc_ast::ast::{
    ClassElement, MethodDefinition, ObjectPropertyKind, PropertyDefinition, PropertyKey,
};
use oxc_span::Span;

/// [`PropName`](https://tc39.es/ecma262/#sec-static-semantics-propname)
pub trait PropName {
    fn prop_name(&self) -> Option<(&str, Span)>;
}

impl PropName for ObjectPropertyKind<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        let key = match self {
            ObjectPropertyKind::KeyValueProperty(prop) => &prop.key,
            ObjectPropertyKind::MethodProperty(prop) => &prop.key,
            ObjectPropertyKind::ShorthandProperty(_) | ObjectPropertyKind::SpreadProperty(_) => {
                return None;
            }
        };
        if key.is_computed() {
            return None;
        }
        key.prop_name()
    }
}

impl PropName for PropertyKey<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        match self {
            PropertyKey::StaticIdentifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::Identifier(ident) => Some((&ident.name, ident.span)),
            PropertyKey::StaticStringLiteral(lit) | PropertyKey::StringLiteral(lit) => {
                Some((lit.value.as_str()?, lit.span))
            }
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
        if self.key.is_computed() {
            return None;
        }
        self.key.prop_name()
    }
}

impl PropName for PropertyDefinition<'_> {
    fn prop_name(&self) -> Option<(&str, Span)> {
        if self.key.is_computed() {
            return None;
        }
        self.key.prop_name()
    }
}
