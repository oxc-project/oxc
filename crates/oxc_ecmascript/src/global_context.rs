use oxc_allocator::Address;
use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_syntax::reference::ReferenceId;

use crate::constant_evaluation::{ConstantValue, ValueType};

pub trait GlobalContext<'a>: Sized {
    /// Whether the reference is a global reference.
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool;

    /// Look up a memoized [`ValueType`] for the expression node at `addr`.
    ///
    /// Default is no-op (no cache). Implemented by the minifier to make `value_type`
    /// O(1) amortized on long binary/logical/conditional chains (otherwise re-walking
    /// the left subtree at every node during the post-order peephole is O(n²)).
    /// Non-minifier contexts keep the uncached behaviour unchanged.
    fn cached_value_type(&self, _addr: Address) -> Option<ValueType> {
        None
    }

    /// Store a memoized [`ValueType`] for the expression node at `addr`. No-op by default.
    fn cache_value_type(&self, _addr: Address, _ty: ValueType) {}

    fn is_global_expr(&self, name: &str, expr: &Expression<'a>) -> bool {
        expr.get_identifier_reference()
            .filter(|ident| ident.name == name)
            .is_some_and(|ident| self.is_global_reference(ident))
    }

    fn get_constant_value_for_reference_id(
        &self,
        _reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        None
    }

    /// The [`ValueType`] of the constant a reference resolves to, if known.
    ///
    /// Cheaper than [`Self::get_constant_value_for_reference_id`] when only the
    /// type discriminant is needed, since it avoids cloning the value
    /// (relevant for `BigInt` / owned `String`).
    fn value_type_for_reference_id(&self, _reference_id: ReferenceId) -> Option<ValueType> {
        None
    }
}

pub struct WithoutGlobalReferenceInformation;

impl<'a> GlobalContext<'a> for WithoutGlobalReferenceInformation {
    fn is_global_reference(&self, _reference: &IdentifierReference<'a>) -> bool {
        false
    }
}
