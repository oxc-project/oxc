use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast;

use crate::{hir, hir_builder::HirBuilder};

pub struct LowerToHIR<'a> {
    hir: HirBuilder<'a>,
}

impl<'a> LowerToHIR<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { hir: HirBuilder::new(allocator) }
    }

    #[must_use]
    pub fn build(mut self, program: &ast::Program<'a>) -> hir::Program<'a> {
        self.lower_program(program)
    }

    #[must_use]
    pub fn lower_vec<T, R, F>(&mut self, items: &Vec<'a, T>, cb: F) -> Vec<'a, R>
    where
        F: Fn(&mut Self, &T) -> R,
    {
        let mut vec = self.hir.new_vec_with_capacity(items.len());
        for item in items {
            vec.push(cb(self, item));
        }
        vec
    }

    fn lower_program(&mut self, program: &ast::Program<'a>) -> hir::Program<'a> {
        let directives = self.lower_vec(&program.directives, Self::lower_directive);
        let statements = self.lower_vec(&program.body, Self::lower_statement);
        self.hir.program(program.span, program.source_type, directives, statements)
    }

    fn lower_directive(&mut self, directive: &ast::Directive<'a>) -> hir::Directive<'a> {
        self.hir.directive(directive.span, directive.expression.clone(), directive.directive)
    }

    #[allow(clippy::unused_self)]
    fn lower_statement(&mut self, _statement: &ast::Statement<'a>) -> hir::Statement<'a> {
        unreachable!()
    }
}
