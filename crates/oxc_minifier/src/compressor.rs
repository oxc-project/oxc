use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{Scoping, SemanticBuilder};

use crate::{
    CompressOptions, ReusableTraverseCtx,
    peephole::{Normalize, NormalizeOptions, PeepholeOptimizations},
    state::MinifierState,
};

pub struct Compressor<'a> {
    allocator: &'a Allocator,
}

impl<'a> Compressor<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { allocator }
    }

    /// Full minify: removes dead code and shrinks the output (`dce = false`).
    /// For tree-shaking only, see [`Self::dead_code_elimination`] and the
    /// `MinifierState::dce` docs.
    pub fn build(self, program: &mut Program<'a>, options: CompressOptions) {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.build_with_scoping(program, scoping, options);
    }

    /// Returns total number of iterations ran.
    ///
    /// # Precondition
    ///
    /// `scoping` must be consistent with `program` in the resolved-reference
    /// domain: every resolved `IdentifierReference` in `program` must be present
    /// in its symbol's resolved-references list, and no list may contain a
    /// `ReferenceId` whose node is absent from `program`. The compressor refreshes
    /// scoping *incrementally* — it only prunes references for nodes it drops, and
    /// no longer rebuilds liveness from scratch each pass — so a caller that
    /// mutated `program` after building `scoping` must reflect those edits in
    /// `scoping`. Stale *extra* references cause missed optimizations (output stays
    /// correct); an *added* reference that was never recorded can cause incorrect
    /// output. In-repo callers satisfy this by rebuilding a fresh `Scoping`
    /// immediately before calling this — e.g. `crates/oxc/src/compiler.rs`
    /// rebuilds scoping before compress/DCE whenever `ReplaceGlobalDefines`
    /// reports a change.
    pub fn build_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) -> u8 {
        let max_iterations = options.max_iterations;
        let state = MinifierState::new(
            program.source_type,
            options,
            /* dce */ false,
            &scoping,
            self.allocator,
        );
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options = NormalizeOptions {
            convert_while_to_fors: true,
            convert_const_to_let: true,
            remove_unnecessary_use_strict: true,
        };
        Normalize::new(normalize_options).build(program, &mut ctx);
        Self::run_in_loop(max_iterations, program, &mut ctx)
    }

    /// Tree-shaking only: removes dead and unused code, but does not shrink the
    /// output like [`Self::build`] (`dce = true`). Rolldown runs this on its
    /// own. See the `MinifierState::dce` docs.
    pub fn dead_code_elimination(self, program: &mut Program<'a>, options: CompressOptions) -> u8 {
        let scoping = SemanticBuilder::new().build(program).semantic.into_scoping();
        self.dead_code_elimination_with_scoping(program, scoping, options)
    }

    /// Returns total number of iterations ran.
    ///
    /// # Precondition
    ///
    /// Same incoming-`scoping` consistency requirement as
    /// [`Self::build_with_scoping`] — see its docs.
    pub fn dead_code_elimination_with_scoping(
        self,
        program: &mut Program<'a>,
        scoping: Scoping,
        options: CompressOptions,
    ) -> u8 {
        let max_iterations = options.max_iterations;
        let state = MinifierState::new(
            program.source_type,
            options,
            /* dce */ true,
            &scoping,
            self.allocator,
        );
        let mut ctx = ReusableTraverseCtx::new(state, scoping, self.allocator);
        let normalize_options = NormalizeOptions {
            convert_while_to_fors: false,
            convert_const_to_let: false,
            remove_unnecessary_use_strict: false,
        };
        Normalize::new(normalize_options).build(program, &mut ctx);
        Self::run_in_loop(max_iterations, program, &mut ctx)
    }

    /// Fixed-point iteration loop for peephole optimizations.
    fn run_in_loop(
        max_iterations: Option<u8>,
        program: &mut Program<'a>,
        ctx: &mut ReusableTraverseCtx<'a>,
    ) -> u8 {
        let mut iteration = 0u8;
        // Boundary for the under-prune debug guard: references with indices
        // beyond this are minted during the loop and legally exempt.
        #[cfg(debug_assertions)]
        let initial_references_len = ctx.get_mut().scoping().references_len();
        // Consume the drops Normalize recorded (`void x` -> `void 0`,
        // drop_console), so pass 1 already observes the pruned reference
        // counts and Normalize's drops cost no extra peephole pass. Its
        // traversal also doubled as the first liveness collection (see
        // `symbol_liveness::begin_pass`), so the same end-of-pass sequence
        // makes source-level dead cycles (`function f() { f() }`) visible
        // to pass 1 — no standalone walk anywhere. The continue signal is
        // irrelevant pre-loop: the loop below always runs at least once.
        PeepholeOptimizations::end_pass(program, ctx.get_mut());
        // Start the loop from a clean signal: Normalize's drops are flushed
        // above, so a Normalize-only mutation must not force a pointless
        // extra iteration.
        ctx.state_mut().take_mutated();
        loop {
            PeepholeOptimizations.run_once(program, ctx);
            let mutated = ctx.state_mut().take_mutated();
            // Flush and consume the liveness collection even on quiet
            // passes (every flush step is a cheap no-op then): pass N's
            // late mutations can kill a cycle that only pass N+1's —
            // otherwise quiet — collection observes. Stopping without
            // consuming it would strand the removal; deferring consumption
            // to the fixed point was measured to LOSE output (other passes
            // rewrite late-exposed dead cycles into non-candidate shapes),
            // so one-pass staleness is the freshness contract.
            let needs_liveness_pass = PeepholeOptimizations::end_pass(program, ctx.get_mut());
            // Convergence: another pass is demanded only for a NEW dead
            // symbol (bounded by the symbol table — see
            // `propagate_collected`); the iteration cap backstops
            // pathological churn.
            if !mutated && !needs_liveness_pass {
                break;
            }
            if let Some(max) = max_iterations {
                if iteration >= max {
                    break;
                }
            } else if iteration > 10 {
                debug_assert!(false, "Ran loop more than 10 times.");
                break;
            }
            iteration += 1;
        }
        #[cfg(debug_assertions)]
        PeepholeOptimizations::debug_assert_no_under_prune(
            program,
            ctx.get_mut(),
            initial_references_len,
        );
        iteration
    }
}
