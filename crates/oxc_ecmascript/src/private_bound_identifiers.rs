use oxc_ast::ast::{
    AccessorProperty, ClassElement, MethodDefinition, PrivateIdentifier, PropertyDefinition,
    PropertyKey,
};

/// [`PrivateBoundIdentifiers`](https://tc39.es/ecma262/#sec-static-semantics-privateboundidentifiers)
pub trait PrivateBoundIdentifiers {
    fn private_bound_identifiers(&self) -> Option<PrivateIdentifier>;
}

impl<'a> PrivateBoundIdentifiers for ClassElement<'a> {
    fn private_bound_identifiers(&self) -> Option<PrivateIdentifier> {
        match self {
            ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => None,
            ClassElement::MethodDefinition(def) => def.private_bound_identifiers(),
            ClassElement::PropertyDefinition(def) => def.private_bound_identifiers(),
            ClassElement::AccessorProperty(def) => def.private_bound_identifiers(),
        }
    }
}

impl<'a> PrivateBoundIdentifiers for MethodDefinition<'a> {
    fn private_bound_identifiers(&self) -> Option<PrivateIdentifier> {
        self.value.body.as_ref()?;
        if let PropertyKey::PrivateIdentifier(ident) = &self.key {
            return Some((*ident).clone());
        }
        None
    }
}

impl<'a> PrivateBoundIdentifiers for PropertyDefinition<'a> {
    fn private_bound_identifiers(&self) -> Option<PrivateIdentifier> {
        if let PropertyKey::PrivateIdentifier(ident) = &self.key {
            return Some((*ident).clone());
        }
        None
    }
}

impl<'a> PrivateBoundIdentifiers for AccessorProperty<'a> {
    fn private_bound_identifiers(&self) -> Option<PrivateIdentifier> {
        if let PropertyKey::PrivateIdentifier(ident) = &self.key {
            return Some((*ident).clone());
        }
        None
    }
}
