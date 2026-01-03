use oxc_allocator::CloneIn;
use oxc_ast::{AstBuilder, ast::BindingPattern};
use oxc_ast_visit::{VisitMut, walk_mut::walk_binding_pattern};

pub struct FormalParameterBindingPattern<'a, 'b> {
    ast: &'b AstBuilder<'a>,
}

impl<'a> VisitMut<'a> for FormalParameterBindingPattern<'a, '_> {
    fn visit_binding_pattern(&mut self, pattern: &mut BindingPattern<'a>) {
        if let BindingPattern::AssignmentPattern(assignment) = pattern {
            *pattern = assignment.left.clone_in(self.ast.allocator);
        }
        walk_binding_pattern(self, pattern);
    }
}

impl<'a> FormalParameterBindingPattern<'a, '_> {
    pub fn remove_assignments_from_kind(ast: &AstBuilder<'a>, pattern: &mut BindingPattern<'a>) {
        let mut visitor = FormalParameterBindingPattern { ast };
        visitor.visit_binding_pattern(pattern);
    }
}
