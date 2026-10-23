use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_syntax::reference::ReferenceId;

use crate::{ConstantValue, ValueType};

pub trait GlobalContext<'a>: Sized {
    /// Whether the reference is a global reference.
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> bool;

    fn is_global_expr(&self, name: &str, expr: &Expression<'a>) -> bool {
        expr.get_identifier_reference()
            .filter(|ident| ident.name == name)
            .is_some_and(|ident| self.is_global_reference(ident))
    }

    /// The constant value that a reference resolves to, if known.
    ///
    /// A returned [`ConstantValue::String`] borrows its text for `'a`, and this trait provides
    /// no allocator. A context that keeps its constants elsewhere can copy them into the arena
    /// with [`JSStr::from_str_in`] when it also has an allocator. Or it can implement these
    /// traits for the shorter lifetime of its own storage and lend the text: the AST is
    /// covariant in `'a`, so a longer-lived AST can be evaluated with such a context.
    ///
    /// [`JSStr::from_str_in`]: oxc_str::JSStr::from_str_in
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
    /// (relevant for `BigInt`).
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
