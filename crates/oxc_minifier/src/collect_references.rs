use std::marker::PhantomData;

use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_semantic::ReferenceId;
use rustc_hash::FxHashSet;

pub struct CollectReferences<'a> {
    refs: FxHashSet<ReferenceId>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Visit<'a> for CollectReferences<'a> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.refs.insert(ident.reference_id());
    }
}

impl<'a> CollectReferences<'a> {
    pub fn new() -> Self {
        Self { refs: FxHashSet::default(), _marker: PhantomData }
    }

    pub fn collect_in_expr(&mut self, expr: &Expression<'a>) -> FxHashSet<ReferenceId> {
        self.refs.clear();
        self.visit_expression(expr);
        self.refs.clone()
    }

    pub fn collect_in_assignment_target(
        &mut self,
        target: &AssignmentTarget<'a>,
    ) -> FxHashSet<ReferenceId> {
        self.refs.clear();
        self.visit_assignment_target(target);
        self.refs.clone()
    }
}
