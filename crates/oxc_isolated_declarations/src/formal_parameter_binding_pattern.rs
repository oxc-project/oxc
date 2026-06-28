use oxc_allocator::{Allocator, CloneIn, GetAllocator};
use oxc_ast::ast::BindingPattern;
use oxc_ast_visit::{VisitMut, walk_mut::walk_binding_pattern};

use crate::IsolatedDeclarations;

pub struct FormalParameterBindingPattern<'a> {
    allocator: &'a Allocator,
}

impl<'a> VisitMut<'a> for FormalParameterBindingPattern<'a> {
    fn visit_binding_pattern(&mut self, pattern: &mut BindingPattern<'a>) {
        if let BindingPattern::AssignmentPattern(assignment) = pattern {
            *pattern = assignment.left.clone_in(self.allocator);
        }
        walk_binding_pattern(self, pattern);
    }
}

impl<'a> FormalParameterBindingPattern<'a> {
    pub fn remove_assignments_from_kind(
        transformer: &IsolatedDeclarations<'a>,
        pattern: &mut BindingPattern<'a>,
    ) {
        let mut visitor = FormalParameterBindingPattern { allocator: transformer.allocator() };
        visitor.visit_binding_pattern(pattern);
    }
}
