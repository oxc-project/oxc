use oxc_ast::ast::IdentifierReference;

pub trait IsGlobalReference {
    fn is_global_reference(&self, reference: &IdentifierReference<'_>) -> bool;
}
