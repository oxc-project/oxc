use oxc_ast::ast::IdentifierReference;

pub trait IsGlobalReference: Sized {
    /// Whether the reference is a global reference.
    ///
    /// - None means it is unknown.
    /// - Some(true) means it is a global reference.
    /// - Some(false) means it is not a global reference.
    fn is_global_reference(&self, reference: &IdentifierReference<'_>) -> Option<bool>;
}

pub struct WithoutGlobalReferenceInformation;

impl IsGlobalReference for WithoutGlobalReferenceInformation {
    fn is_global_reference(&self, _reference: &IdentifierReference<'_>) -> Option<bool> {
        None
    }
}
