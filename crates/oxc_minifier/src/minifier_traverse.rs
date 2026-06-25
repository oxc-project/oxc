use std::ptr;

use oxc_ast::ast::Program;

use crate::{
    ReusableTraverseCtx, Traverse,
    generated::{ancestor::Ancestor, walk::walk_ast},
};

pub fn traverse_mut_with_ctx<'a, Tr: Traverse<'a>>(
    traverser: &mut Tr,
    program: &mut Program<'a>,
    ctx: &mut ReusableTraverseCtx<'a>,
) {
    let program = ptr::from_mut(program);
    let ctx = ctx.get_mut();

    debug_assert!(ctx.ancestors_depth() == 1);
    debug_assert!(matches!(ctx.parent(), Ancestor::None));

    // SAFETY: `program` originates from `&mut Program<'a>` and context stack invariants are checked.
    unsafe { walk_ast(traverser, program, ctx) };

    debug_assert!(ctx.ancestors_depth() == 1);
    debug_assert!(matches!(ctx.parent(), Ancestor::None));
}
