use oxc_allocator::Allocator;
use oxc_ast::{ast::*, visit::walk_mut, AstBuilder, VisitMut};
use oxc_span::SPAN;

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
#[derive(Clone, Copy)]
pub struct RemoveDeadCode<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> RemoveDeadCode<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { ast: AstBuilder::new(allocator) }
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    pub fn remove_if(&mut self, stmt: &mut Statement<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };
        match if_stmt.test.get_boolean_value() {
            Some(true) => {
                *stmt = self.ast.move_statement(&mut if_stmt.consequent);
            }
            Some(false) => {
                *stmt = if let Some(alternate) = &mut if_stmt.alternate {
                    self.ast.move_statement(alternate)
                } else {
                    self.ast.empty_statement(SPAN)
                };
            }
            _ => {}
        }
    }
}

impl<'a> VisitMut<'a> for RemoveDeadCode<'a> {
    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.remove_if(stmt);
        walk_mut::walk_statement_mut(self, stmt);
    }
}
