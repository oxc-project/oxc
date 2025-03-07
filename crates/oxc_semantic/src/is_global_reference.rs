use oxc_ast::ast::{Expression, IdentifierReference};

use crate::{ReferenceId, SymbolTable};

/// Checks whether the a identifier reference is a global value or not.
pub trait IsGlobalReference {
    fn is_global_reference(&self, _symbols: &SymbolTable) -> bool;
    fn is_global_reference_name(&self, name: &str, _symbols: &SymbolTable) -> bool;
}

impl IsGlobalReference for ReferenceId {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        symbols.references[*self].symbol_id().is_none()
    }

    fn is_global_reference_name(&self, _name: &str, _symbols: &SymbolTable) -> bool {
        panic!("This function is pointless to be called.");
    }
}

impl IsGlobalReference for IdentifierReference<'_> {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        self.reference_id
            .get()
            .is_some_and(|reference_id| reference_id.is_global_reference(symbols))
    }

    fn is_global_reference_name(&self, name: &str, symbols: &SymbolTable) -> bool {
        self.name == name && self.is_global_reference(symbols)
    }
}

impl IsGlobalReference for Expression<'_> {
    fn is_global_reference(&self, symbols: &SymbolTable) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference(symbols);
        }
        false
    }

    fn is_global_reference_name(&self, name: &str, symbols: &SymbolTable) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference_name(name, symbols);
        }
        false
    }
}
