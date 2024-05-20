use oxc_ast::ast::IdentifierReference;
use oxc_traverse::{ancestor::ProgramWithoutDirectives, Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    program: Option<&'b ProgramWithoutDirectives<'a>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Ancestor::ProgramDirectives(program) = ctx.parent() {
            self.program = Some(program);
        }
    }
}

fn main() {}
