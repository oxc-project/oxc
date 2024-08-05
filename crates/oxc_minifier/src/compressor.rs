use oxc_allocator::Allocator;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder};

use crate::{
    ast_passes::{
        Collapse, FoldConstants, RemoveDeadCode, RemoveSyntax, SubstituteAlternateSyntax,
    },
    CompressOptions,
};

pub struct Compressor<'a> {
    ast: AstBuilder<'a>,
    options: CompressOptions,
}

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator, options: CompressOptions) -> Self {
        let ast = AstBuilder::new(allocator);
        Self { ast, options }
    }

    pub fn build(self, program: &mut Program<'a>) {
        // TODO: inline variables
        self.remove_syntax(program);
        self.fold_constants(program);
        self.remove_dead_code(program);
        // TODO: StatementFusion
        self.substitute_alternate_syntax(program);
        self.collapse(program);
    }

    fn remove_syntax(&self, program: &mut Program<'a>) {
        if self.options.remove_syntax {
            RemoveSyntax::new(self.ast, self.options).build(program);
        }
    }

    fn fold_constants(&self, program: &mut Program<'a>) {
        if self.options.fold_constants {
            FoldConstants::new(self.ast).with_evaluate(self.options.evaluate).build(program);
        }
    }

    fn substitute_alternate_syntax(&self, program: &mut Program<'a>) {
        if self.options.substitute_alternate_syntax {
            SubstituteAlternateSyntax::new(self.ast, self.options).build(program);
        }
    }

    fn remove_dead_code(&self, program: &mut Program<'a>) {
        if self.options.remove_dead_code {
            RemoveDeadCode::new(self.ast).build(program);
        }
    }

    fn collapse(&self, program: &mut Program<'a>) {
        if self.options.collapse {
            Collapse::new(self.ast, self.options).build(program);
        }
    }
}
