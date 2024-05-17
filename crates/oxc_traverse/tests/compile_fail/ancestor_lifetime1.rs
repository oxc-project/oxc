use oxc_ast::ast::IdentifierReference;
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

struct Trans<'a, 'b> {
    ancestor: Option<&'b Ancestor<'a>>,
}

impl<'a, 'b> Traverse<'a> for Trans<'a, 'b> {
    fn enter_identifier_reference(
        &mut self,
        _node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.ancestor = Some(ctx.parent());
    }
}

fn main() {}
