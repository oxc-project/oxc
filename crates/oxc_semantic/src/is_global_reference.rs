use oxc_ast::ast::{Expression, IdentifierReference};

use crate::{ReferenceId, Scoping};

/// Checks whether the a identifier reference is a global value or not.
pub trait IsGlobalReference {
    fn is_global_reference(&self, scoping: &Scoping) -> bool;
    fn is_global_reference_name(&self, name: &str, scoping: &Scoping) -> bool;
}

impl IsGlobalReference for ReferenceId {
    fn is_global_reference(&self, scoping: &Scoping) -> bool {
        scoping.references[*self].symbol_id().is_none()
    }

    fn is_global_reference_name(&self, _name: &str, _scoping: &Scoping) -> bool {
        panic!("This function is pointless to be called.");
    }
}

impl IsGlobalReference for IdentifierReference<'_> {
    fn is_global_reference(&self, scoping: &Scoping) -> bool {
        self.reference_id
            .get()
            .is_some_and(|reference_id| reference_id.is_global_reference(scoping))
    }

    fn is_global_reference_name(&self, name: &str, scoping: &Scoping) -> bool {
        self.name == name && self.is_global_reference(scoping)
    }
}

impl IsGlobalReference for Expression<'_> {
    fn is_global_reference(&self, scoping: &Scoping) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference(scoping);
        }
        false
    }

    fn is_global_reference_name(&self, name: &str, scoping: &Scoping) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference_name(name, scoping);
        }
        false
    }
}
