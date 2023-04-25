use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast;

use crate::hir;

pub struct LowerToHIR<'a> {
    allocator: &'a Allocator,
}

impl<'a> LowerToHIR<'a> {
    #[must_use]
    pub fn build(self, program: ast::Program<'a>) -> hir::Program<'a> {
        self.lower_program(program)
    }

    fn lower_program(&self, program: ast::Program<'a>) -> hir::Program<'a> {
        hir::Program {
            span: program.span,
            source_type: program.source_type,
            directives: self.lower_directives(program.directives),
            // body: program.body,
        }
    }

    fn lower_directives(
        &self,
        directives: Vec<'a, ast::Directive<'a>>,
    ) -> Vec<'a, hir::Directive<'a>> {
        let directives =
            directives.into_iter().map(|d| hir::Directive { span: d.span, directive: d.directive });
        Vec::from_iter_in(directives, self.allocator)
    }

    // manually type out everything ...
}
