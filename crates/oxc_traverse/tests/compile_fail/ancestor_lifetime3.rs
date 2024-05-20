use oxc_allocator::Vec;
use oxc_ast::ast::{IdentifierReference, Statement};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    program_body: Option<&'b Vec<'a, Statement<'a>>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Ancestor::ProgramDirectives(program) = ctx.parent() {
            self.program_body = Some(program.body());
        }
    }
}

fn main() {}
