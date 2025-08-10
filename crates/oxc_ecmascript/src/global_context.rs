use oxc_ast::ast::IdentifierReference;
use oxc_syntax::reference::ReferenceId;

use crate::constant_evaluation::ConstantValue;

pub trait GlobalContext<'a>: Sized {
    /// Whether the reference is a global reference.
    ///
    /// - None means it is unknown.
    /// - Some(true) means it is a global reference.
    /// - Some(false) means it is not a global reference.
    fn is_global_reference(&self, reference: &IdentifierReference<'a>) -> Option<bool>;

    fn get_constant_value_for_reference_id(
        &self,
        _reference_id: ReferenceId,
    ) -> Option<ConstantValue<'a>> {
        None
    }
}

pub struct WithoutGlobalReferenceInformation;

impl<'a> GlobalContext<'a> for WithoutGlobalReferenceInformation {
    fn is_global_reference(&self, _reference: &IdentifierReference<'a>) -> Option<bool> {
        None
    }
}
