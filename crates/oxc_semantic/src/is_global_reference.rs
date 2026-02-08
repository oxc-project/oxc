use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_span::Ident;

use crate::{ReferenceId, Scoping};

/// Checks whether an identifier reference is a global value or not.
///
/// # Limitation: `with` statements
///
/// This check does not correctly handle references inside `with` statements.
/// Inside a `with` block, unresolved references may resolve to properties of
/// the `with` object at runtime, but this function will incorrectly return
/// `true` for such references.
///
/// ```js
/// const foo = { Object: class { /* ... */ } }
/// with (foo) { console.log(new Object()) }
/// //                           ^^^^^^ This `Object` is NOT a global reference,
/// //                                  but `is_global_reference` returns `true`.
/// ```
///
/// This is acceptable because:
/// 1. `with` statements are forbidden in strict mode
/// 2. Bundlers like Rolldown don't support `with` statements
/// 3. The minifier bails out when `with` statements are present
///
/// See: <https://github.com/oxc-project/oxc/issues/8365>
pub trait IsGlobalReference {
    fn is_global_reference(&self, scoping: &Scoping) -> bool;
    fn is_global_reference_name(&self, name: Ident<'_>, scoping: &Scoping) -> bool;
}

impl IsGlobalReference for ReferenceId {
    fn is_global_reference(&self, scoping: &Scoping) -> bool {
        scoping.references[*self].symbol_id().is_none()
    }

    fn is_global_reference_name(&self, _name: Ident<'_>, _scoping: &Scoping) -> bool {
        panic!("This function is pointless to be called.");
    }
}

impl IsGlobalReference for IdentifierReference<'_> {
    fn is_global_reference(&self, scoping: &Scoping) -> bool {
        self.reference_id
            .get()
            .is_some_and(|reference_id| reference_id.is_global_reference(scoping))
    }

    fn is_global_reference_name(&self, name: Ident<'_>, scoping: &Scoping) -> bool {
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

    fn is_global_reference_name(&self, name: Ident<'_>, scoping: &Scoping) -> bool {
        if let Expression::Identifier(ident) = self {
            return ident.is_global_reference_name(name, scoping);
        }
        false
    }
}
