use oxc_ast::{
    ast::BindingPatternKind, visit::walk_mut::walk_binding_pattern_kind, AstBuilder, VisitMut,
};

pub struct FormalParameterBindingPattern<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> VisitMut<'a> for FormalParameterBindingPattern<'a> {
    fn visit_binding_pattern_kind(&mut self, kind: &mut BindingPatternKind<'a>) {
        if let BindingPatternKind::AssignmentPattern(assignment) = kind {
            *kind = self.ast.copy(&assignment.left.kind);
        }

        walk_binding_pattern_kind(self, kind);
    }
}

impl<'a> FormalParameterBindingPattern<'a> {
    pub fn remove_assignments_from_kind(ast: AstBuilder<'a>, kind: &mut BindingPatternKind<'a>) {
        let mut visitor = FormalParameterBindingPattern { ast };
        visitor.visit_binding_pattern_kind(kind);
    }
}
