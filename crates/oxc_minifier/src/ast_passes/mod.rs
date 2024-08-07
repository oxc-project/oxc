mod collapse;
mod fold_constants;
mod remove_dead_code;
mod remove_syntax;
mod substitute_alternate_syntax;

pub use collapse::Collapse;
pub use fold_constants::FoldConstants;
pub use remove_dead_code::RemoveDeadCode;
pub use remove_syntax::RemoveSyntax;
pub use substitute_alternate_syntax::SubstituteAlternateSyntax;

use oxc_ast::ast::Program;
use oxc_traverse::{walk_program, Traverse, TraverseCtx};

pub trait CompressorPass<'a> {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>)
    where
        Self: Traverse<'a>,
        Self: Sized,
    {
        walk_program(self, program, ctx);
    }
}
