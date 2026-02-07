use std::ptr;

use oxc_ast::ast::Program;

use crate::{
    generated::{ancestor::Ancestor, traverse::MinifierTraverse, walk::walk_ast},
    traverse_context::ReusableMinifierTraverseCtx,
};

pub fn traverse_mut_with_ctx<'a, Tr: MinifierTraverse<'a>>(
    traverser: &mut Tr,
    program: &mut Program<'a>,
    ctx: &mut ReusableMinifierTraverseCtx<'a>,
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
