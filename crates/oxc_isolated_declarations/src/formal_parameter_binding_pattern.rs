use oxc_allocator::ReplaceWith;
use oxc_ast::ast::BindingPattern;
use oxc_ast_visit::{VisitMut, walk_mut::walk_binding_pattern};

pub struct FormalParameterBindingPattern;

impl<'a> VisitMut<'a> for FormalParameterBindingPattern {
    fn visit_binding_pattern(&mut self, pattern: &mut BindingPattern<'a>) {
        if matches!(pattern, BindingPattern::AssignmentPattern(_)) {
            pattern.replace_with(|pattern| {
                let BindingPattern::AssignmentPattern(assignment) = pattern else { unreachable!() };
                assignment.unbox().left
            });
        }

        walk_binding_pattern(self, pattern);
    }
}

impl FormalParameterBindingPattern {
    pub fn remove_assignments_from_kind(pattern: &mut BindingPattern<'_>) {
        FormalParameterBindingPattern.visit_binding_pattern(pattern);
    }
}
