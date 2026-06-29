use oxc_allocator::{Allocator, GetAllocator, TakeIn};
use oxc_ast::ast::BindingPattern;
use oxc_ast_visit::{VisitMut, walk_mut::walk_binding_pattern};

use crate::IsolatedDeclarations;

pub struct FormalParameterBindingPattern<'a> {
    allocator: &'a Allocator,
}

impl<'a> VisitMut<'a> for FormalParameterBindingPattern<'a> {
    fn visit_binding_pattern(&mut self, pattern: &mut BindingPattern<'a>) {
        if let BindingPattern::AssignmentPattern(_) = pattern {
            // Take the `BindingPattern`, not the `AssignmentPattern` because `BindingPattern::take_in`
            // performs less allocations. Compiler removes the unreachable branch here.
            let old_pattern = pattern.take_in(&self.allocator);
            if let BindingPattern::AssignmentPattern(assignment) = old_pattern {
                *pattern = assignment.unbox().left;
            } else {
                unreachable!();
            }
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
