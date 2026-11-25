use std::ptr;

use oxc_ast::ast::Program;

use crate::{
    ReusableTraverseCtx, Traverse, TraverseCtx,
    generated::{ancestor::Ancestor, walk::walk_ast},
};

/// Guard which resets a [`TraverseCtx`]'s ancestry stack to its initial state when dropped.
///
/// `walk_*` functions push `Ancestor`s onto the ancestry stack as they descend the AST, and pop
/// them off on the way back up. If a visitor method panics, the unwind skips the pending pops, so
/// the stack would be left containing stale `Ancestor`s. Reusing the `TraverseCtx` (the peephole
/// passes reuse their `ReusableTraverseCtx` across many traversals) could then be used to violate
/// the aliasing rules which `TraverseAncestry` exists to uphold.
///
/// This guard resets the stack back to single `Ancestor::None` when dropped, so the safety
/// invariants are upheld even if a visitor panics and the panic is caught by an embedding (e.g.
/// with `std::panic::catch_unwind`).
///
/// On the normal path (no panic), the stack is already back to its initial state when the guard
/// is dropped, so the reset is a no-op (a single length comparison).
struct AncestryResetGuard<'c, 'a>(&'c mut TraverseCtx<'a>);

impl Drop for AncestryResetGuard<'_, '_> {
    #[inline]
    fn drop(&mut self) {
        self.0.ancestry.reset();
    }
}

pub fn traverse_mut_with_ctx<'a, Tr: Traverse<'a>>(
    traverser: &mut Tr,
    program: &mut Program<'a>,
    ctx: &mut ReusableTraverseCtx<'a>,
) {
    let program = ptr::from_mut(program);

    let ctx = ctx.get_mut();

    // Check that `TraverseAncestry`'s stack is in correct state
    assert_eq!(ctx.ancestors_depth(), 1);
    assert!(matches!(ctx.parent(), Ancestor::None));

    // Ensure `TraverseAncestry`'s stack is back in its initial state when traversal ends, whether
    // by returning normally or by a panic in a visitor method unwinding out of `walk_ast`
    let guard = AncestryResetGuard(&mut *ctx);

    // SAFETY: `program` originates from a `&mut Program<'a>` and the ancestry stack is in a
    // known-good state on entry, and restored to a known-good state on exit, normal or unwinding
    // (`AncestryResetGuard`).
    unsafe { walk_ast(traverser, program, &mut *guard.0) };

    drop(guard);

    // Check that `TraverseAncestry`'s stack is in correct state
    debug_assert_eq!(ctx.ancestors_depth(), 1);
    debug_assert!(matches!(ctx.parent(), Ancestor::None));
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Expression, ExpressionStatement, Program, Statement};
    use oxc_ast::builder::AstBuilder;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::{SPAN, SourceType};

    use super::*;

    use crate::state::MinifierState;
    use crate::{CompressOptions, CompressionMode};

    /// Visitor which panics partway through traversal, simulating a bug in a minifier pass
    /// whose panic is caught by an embedding driver.
    struct PanickingVisitor;

    impl Traverse<'_> for PanickingVisitor {
        fn enter_expression_statement(
            &mut self,
            _node: &mut ExpressionStatement<'_>,
            _ctx: &mut TraverseCtx<'_>,
        ) {
            panic!("simulated bug in a minifier visitor");
        }
    }

    /// Visitor which records the ancestry state at the start of a traversal.
    #[derive(Default)]
    struct InspectingVisitor {
        program_entry_depth: usize,
        program_entry_parent_is_none: bool,
    }

    impl Traverse<'_> for InspectingVisitor {
        fn enter_program(&mut self, _node: &mut Program<'_>, ctx: &mut TraverseCtx<'_>) {
            self.program_entry_depth = ctx.ancestors_depth();
            self.program_entry_parent_is_none = matches!(ctx.parent(), Ancestor::None);
        }
    }

    /// Regression test for GH issue #30240: a visitor panic unwinding out of `walk_ast` skips the
    /// `pop_stack`s which correspond to its `push_stack`s, so `TraverseAncestry`'s stack would be
    /// left with stale `Ancestor`s. The peephole passes reuse one `ReusableTraverseCtx` for many
    /// traversals, so reusing it after a caught panic must not expose those stale `Ancestor`s.
    #[test]
    fn ancestry_stack_is_reset_after_visitor_panic() {
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);

        let stmt = Statement::new_expression_statement(
            SPAN,
            Expression::new_boolean_literal(SPAN, true, &ast),
            &ast,
        );
        let mut program = Program::new(SPAN, SourceType::mjs(), "", [], None, [], [stmt], &ast);

        // Semantic analysis assigns the node / scope IDs which `walk_*` methods read
        let semantic = SemanticBuilder::new().build(&program).semantic;
        let scoping = semantic.into_scoping();

        let state = MinifierState::new(
            SourceType::mjs(),
            CompressOptions::default(),
            CompressionMode::Full,
            &scoping,
            &allocator,
        );
        let mut ctx = ReusableTraverseCtx::new(state, scoping, &allocator);

        // 1st traversal: panic partway through, and catch the panic like an embedding could
        let mut panicking = PanickingVisitor;
        let result = catch_unwind(AssertUnwindSafe(|| {
            traverse_mut_with_ctx(&mut panicking, &mut program, &mut ctx);
        }));
        assert!(result.is_err());

        // 2nd traversal: the reused context must start with a clean ancestry stack
        // (`enter_program` runs before `walk_program` pushes anything onto the stack)
        let mut inspecting = InspectingVisitor::default();
        traverse_mut_with_ctx(&mut inspecting, &mut program, &mut ctx);
        assert_eq!(inspecting.program_entry_depth, 1);
        assert!(inspecting.program_entry_parent_is_none);

        // ...and end with a clean ancestry stack
        let ctx = ctx.get_mut();
        assert_eq!(ctx.ancestors_depth(), 1);
        assert!(matches!(ctx.parent(), Ancestor::None));
    }
}
