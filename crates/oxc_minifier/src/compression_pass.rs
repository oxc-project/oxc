//! Completion of Normalize and peephole compression passes.
//!
//! AST mutations accumulate removed semantic references and dropped direct
//! `eval` calls on [`crate::state::PassChanges`]. Both Normalize and each
//! peephole pass cross the same completion boundary: prune those references,
//! refresh direct-eval scope flags, then derive function reachability from the
//! settled semantic state.

#[cfg(debug_assertions)]
use oxc_ast_visit::Visit;
use oxc_ast_visit::{VisitJs, walk_js::walk_call_expression};
use rustc_hash::FxHashSet;

use oxc_allocator::{BitSet, GetAllocator};
use oxc_ast::ast::*;
use oxc_semantic::Scoping;
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use crate::{
    ReusableTraverseCtx, TraverseCtx, minifier_traverse::traverse_mut_with_ctx,
    peephole::PeepholeOptimizations, symbol_liveness, traverse_context::as_direct_eval_call,
};

/// The fixed-point decision produced by one completed peephole pass.
#[must_use]
pub struct PassOutcome {
    pub(crate) needs_another_pass: bool,
}

/// Refresh `ScopeFlags::DirectEval` from live direct-eval call sites.
///
/// `direct_eval_scopes` lists scopes that still contain a direct `eval(...)` call.
/// Clears `DirectEval` from every scope, then re-propagates from each scope in the
/// set up to the root.
///
/// Skipping this leaves `DirectEval` set on scopes whose only eval call was just
/// DCE'd, which keeps unused-declaration removal conservative until reparse.
fn refresh_direct_eval_flags(scoping: &mut Scoping, direct_eval_scopes: &FxHashSet<ScopeId>) {
    // Semantic propagates `DirectEval` to the root, so an empty live set plus a
    // clean root means no scope has the flag — nothing to clear or set.
    if direct_eval_scopes.is_empty() && !scoping.root_scope_flags().contains_direct_eval() {
        return;
    }

    for index in 0..scoping.scopes_len() {
        scoping.scope_flags_mut(ScopeId::from_usize(index)).remove(ScopeFlags::DirectEval);
    }

    for &scope_id in direct_eval_scopes {
        let mut ancestor = Some(scope_id);
        while let Some(scope_id) = ancestor {
            let flags = scoping.scope_flags_mut(scope_id);
            // An earlier iteration already flagged this chain; stop walking up.
            if flags.contains_direct_eval() {
                break;
            }
            flags.insert(ScopeFlags::DirectEval);
            ancestor = scoping.scope_parent_id(scope_id);
        }
    }
}

/// Debug-only guard for the incremental scoping refresh: every reference
/// marked in [`crate::state::PassChanges::removed_references`] must really be
/// gone from the live program. Pruning a still-live reference is the unsafe
/// direction that produces incorrect output.
///
/// Walks the live program once per changed pass in debug builds only, so the
/// entire unit-test and `cargo coverage -- minifier` corpus doubles as an
/// over-prune detector at zero release cost.
#[cfg(debug_assertions)]
fn debug_assert_no_over_prune(program: &Program<'_>, removed_references: &BitSet<'_>) {
    struct OverPruneCheck<'b, 'c> {
        removed_references: &'b BitSet<'c>,
    }
    impl<'a> Visit<'a> for OverPruneCheck<'_, '_> {
        fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
            let Some(reference_id) = it.reference_id.get() else { return };
            let idx = reference_id.index();
            // `contains` is false past capacity — the capacity guard
            // (see `PassChanges::removed_references`).
            assert!(
                !self.removed_references.contains(idx),
                "incremental scoping over-prune: reference {idx} is marked removed but still \
                 appears in the live program",
            );
        }
    }
    OverPruneCheck { removed_references }.visit_program(program);
}

/// Debug-only converse of [`debug_assert_no_over_prune`], run once by the
/// `Compressor` driver after the fixed-point loop: every reference that
/// existed when the loop began and is still in a symbol's resolved-references
/// list must appear in the live program. A violation means a site discarded a
/// subtree without routing it through a `drop_*` / `replace_*` helper (the leak
/// direction: stale references silently block optimizations), or the caller
/// passed a `scoping` already inconsistent with `program` (see the precondition
/// on `Compressor::build_with_scoping`).
///
/// References minted during the loop (`idx >= initial_references_len`) are
/// exempt: the capacity guard deliberately leaves a same-pass mint-then-drop
/// unmarked (see `PassChanges::removed_references`).
///
/// Together with the over-prune guard this closes both failure directions of
/// the drop-helper convention across the whole unit-test and conformance
/// corpus, at zero release cost.
#[cfg(debug_assertions)]
pub fn debug_assert_no_under_prune(
    program: &Program<'_>,
    ctx: &TraverseCtx<'_>,
    initial_references_len: usize,
) {
    struct LiveRefCollector<'b, 'c> {
        live: &'b mut BitSet<'c>,
    }
    impl<'a> Visit<'a> for LiveRefCollector<'_, '_> {
        fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
            if let Some(reference_id) = it.reference_id.get() {
                let idx = reference_id.index();
                if idx < self.live.capacity() {
                    self.live.set_bit(idx);
                }
            }
        }
    }
    let mut live = BitSet::new_in(initial_references_len, ctx.allocator());
    LiveRefCollector { live: &mut live }.visit_program(program);
    for reference_ids in ctx.scoping().resolved_references() {
        for reference_id in reference_ids {
            let idx = reference_id.index();
            assert!(
                idx >= initial_references_len || live.has_bit(idx),
                "incremental scoping under-prune: reference {idx} is still in a symbol's \
                 resolved-references list but its node is gone from the program — a drop site \
                 bypassed the `drop_*` / `replace_*` helpers, or the caller passed a `scoping` \
                 inconsistent with `program`",
            );
        }
    }
}

/// Debug-only guard for the gated direct-eval refresh: every live direct
/// `eval(...)` call must already have `ScopeFlags::DirectEval` on its
/// reference's recorded scope and every ancestor (the exact postcondition of
/// [`refresh_direct_eval_flags`]). The gate
/// ([`crate::state::PassChanges::direct_eval_dropped`]) only re-derives flags
/// when an eval call is *dropped*, so it is sound only while no pass *forms* a
/// new direct eval call (e.g. by moving `eval` into callee position). Stale-SET
/// flags are merely conservative and not checked; only the missing direction
/// is unsafe.
///
/// Locally-bound `eval` callees are exempt: `remove_sequence_expression`
/// deliberately forms them (`var eval; (0, eval)()` -> `var eval; eval()` —
/// `should_keep_indirect_access` only protects the *global* `eval`), banking on
/// a local binding named `eval` not holding the real `eval`. Under that same
/// assumption the missing flag is inert; a later refresh may still set it (the
/// name-based collector), which is the allowed conservative direction.
///
/// Allocation-free by design: asserts inline per call during the walk so the
/// allocation-tracking task (debug assertions on) sees no sys-allocs. The walk
/// itself is skipped when the program has no unresolved `eval` at all: the
/// check only fires on unresolved (global) callees, and every live unresolved
/// reference's name is a key in `root_unresolved_references` (populated at
/// build, appended on in-loop mints, deliberately never pruned in-loop) — a
/// conservative superset that can never skip a checkable call.
#[cfg(debug_assertions)]
fn debug_assert_no_stale_direct_eval(program: &Program<'_>, scoping: &Scoping) {
    struct DirectEvalFlagCheck<'s> {
        scoping: &'s Scoping,
    }
    impl<'a> VisitJs<'a> for DirectEvalFlagCheck<'_> {
        fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
            if let Some(ident) = as_direct_eval_call(it)
                && let Some(reference_id) = ident.reference_id.get()
            {
                let reference = self.scoping.get_reference(reference_id);
                // No symbol = unresolved = the global `eval` (see above).
                if reference.symbol_id().is_none() {
                    // Same scope derivation as `LiveDirectEvalCollector` —
                    // producer, consumer, and this check must agree.
                    for scope_id in self.scoping.scope_ancestors(reference.scope_id()) {
                        assert!(
                            self.scoping.scope_flags(scope_id).contains_direct_eval(),
                            "stale direct-eval flags: scope {scope_id:?} is missing \
                             `ScopeFlags::DirectEval` for a live direct `eval(...)` call — a \
                             pass formed a new direct eval call without dropping one — see \
                             `PassChanges::direct_eval_dropped`",
                        );
                    }
                }
            }
            walk_call_expression(self, it);
        }
    }
    if !scoping.root_unresolved_references().contains_key("eval") {
        return;
    }
    DirectEvalFlagCheck { scoping }.visit_program(program);
}

/// Consume the [`crate::state::PassChanges`] accumulator: batch-prune removed
/// resolved references from scoping, refresh direct-eval flags if an
/// `eval(...)` call was dropped, and re-initialize the accumulator.
///
/// Pass completion calls this after Normalize (so the fixed-point loop starts
/// against already-pruned scoping and Normalize's drops cost no extra
/// peephole pass) and after every peephole pass — quiet ones included, where
/// every step below is a cheap no-op.
fn flush_pass_changes(program: &Program<'_>, ctx: &mut TraverseCtx<'_>) -> bool {
    let had_removed_references = !ctx.state.pass_changes.removed_references.is_empty();
    let liveness_inputs_changed = ctx.state.pass_changes.direct_eval_dropped
        || (had_removed_references && symbol_liveness::dead_references_affect_analysis(ctx));

    // (1) Resolved references — direct consumption, no walk.
    //     Change data is built by `replace_*` / `drop_*` helpers as subtrees
    //     are removed and is consumed here in one batch.
    if had_removed_references {
        // Debug-only guard: every reference we are about to prune must really
        // be gone from the live program (see the helper).
        #[cfg(debug_assertions)]
        debug_assert_no_over_prune(program, &ctx.state.pass_changes.removed_references);

        // Disjoint-field borrows: `state.pass_changes` and `scoping` don't overlap.
        ctx.scoping
            .scoping_mut()
            .retain_resolved_references_excluding(&ctx.state.pass_changes.removed_references);
    }

    // (2) Direct-eval — gated full walk only when an eval was dropped.
    if ctx.state.pass_changes.direct_eval_dropped {
        let scoping = ctx.scoping();
        let mut live = LiveDirectEvalCollector::new(scoping);
        live.visit_program(program);
        let scopes = live.scopes;
        refresh_direct_eval_flags(ctx.scoping_mut(), &scopes);
    }
    // Debug-only converse of the gate: no pass may have FORMED a new direct
    // eval call without dropping one (see the helper).
    #[cfg(debug_assertions)]
    debug_assert_no_stale_direct_eval(program, ctx.scoping());

    // (3) Reset the accumulator for the next pass. `references_len` only grows
    //     (helpers mint, never delete, references), so the bitset is
    //     re-allocated only when refs were minted this pass; otherwise a
    //     memset reuses the warm allocation (a bump arena never reclaims the
    //     old one).
    let refs_len = ctx.scoping().references_len();
    if ctx.state.pass_changes.removed_references.capacity() == refs_len {
        if had_removed_references {
            ctx.state.pass_changes.removed_references.clear();
        }
    } else {
        ctx.state.pass_changes.removed_references = BitSet::new_in(refs_len, ctx.allocator());
    }
    ctx.state.pass_changes.direct_eval_dropped = false;
    liveness_inputs_changed
}

/// Complete semantic bookkeeping by flushing accumulated changes into
/// scoping, then deriving function reachability from those settled references.
/// Keeping the pair together makes the ordering structural.
fn finish_pass<'a>(
    program: &Program<'a>,
    ctx: &mut TraverseCtx<'a>,
    force_liveness_analysis: bool,
) -> bool {
    let liveness_inputs_changed = flush_pass_changes(program, ctx);
    symbol_liveness::analyze(program, ctx, force_liveness_analysis || liveness_inputs_changed)
}

/// Finish Normalize's semantic journal before the unconditional first
/// peephole pass. Normalize's shape changes do not drive fixed-point
/// convergence, so consume the revisit request and ignore newly dead functions:
/// pass one runs regardless and consumes both forms of progress.
pub fn finish_normalize_pass<'a>(program: &Program<'a>, ctx: &mut TraverseCtx<'a>) {
    let _ = ctx.state.take_revisit_requested();
    finish_pass(program, ctx, /* force_liveness_analysis */ true);
    debug_assert_pass_changes_clean(ctx);
}

/// Run and finish one ordinary peephole pass as a single transaction.
///
/// The returned outcome combines AST/fact progress with newly published dead
/// functions. A revisit request alone does not force liveness recomputation;
/// reference removal and dropped direct eval continue to gate that analysis.
pub fn run_peephole_pass<'a>(
    program: &mut Program<'a>,
    ctx: &mut ReusableTraverseCtx<'a>,
) -> PassOutcome {
    debug_assert_pass_changes_clean(ctx.get_mut());
    ctx.state_mut().symbols.reset_values();
    traverse_mut_with_ctx(&mut PeepholeOptimizations, program, ctx);

    let ctx = ctx.get_mut();
    let revisit_requested = ctx.state.take_revisit_requested();
    let newly_dead = finish_pass(program, ctx, /* force_liveness_analysis */ false);
    debug_assert!(
        !newly_dead || revisit_requested,
        "ordinary liveness progress must follow a recorded pass change"
    );
    debug_assert_pass_changes_clean(ctx);

    PassOutcome { needs_another_pass: revisit_requested || newly_dead }
}

#[inline]
fn debug_assert_pass_changes_clean(ctx: &TraverseCtx<'_>) {
    debug_assert!(ctx.state.pass_changes_are_clean());
}

/// Walks the live program to find scopes containing direct `eval(...)` calls.
/// Used by [`flush_pass_changes`] only when at least one direct eval call was
/// dropped this pass (gated via
/// [`crate::state::PassChanges::direct_eval_dropped`]).
struct LiveDirectEvalCollector<'s> {
    scoping: &'s Scoping,
    scopes: FxHashSet<ScopeId>,
}

impl<'s> LiveDirectEvalCollector<'s> {
    fn new(scoping: &'s Scoping) -> Self {
        Self { scoping, scopes: FxHashSet::default() }
    }
}

impl<'a> VisitJs<'a> for LiveDirectEvalCollector<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(ident) = as_direct_eval_call(it)
            && let Some(reference_id) = ident.reference_id.get()
        {
            let scope_id = self.scoping.get_reference(reference_id).scope_id();
            self.scopes.insert(scope_id);
        }
        // Recurse — `eval` may be nested in another call's arguments, e.g. `foo(eval('x'))`.
        walk_call_expression(self, it);
    }
}
