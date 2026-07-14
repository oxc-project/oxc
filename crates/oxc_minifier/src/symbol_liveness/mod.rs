//! Whole-program symbol liveness for unused-declaration removal (#13105).
//!
//! `Scoping::symbol_is_unused` is reference-count based; a reference cycle
//! keeps every member's count above zero forever, so `function c() { d() }
//! function d() { c() }` survives even though no live code can reach it.
//! This module computes reachability instead. ESM MODULES ONLY (see
//! [`collection_enabled`]): sloppy sources — scripts, CommonJS — carry
//! observability the reference count cannot express (script-globals,
//! Annex B block-function aliases) and are deliberately not analyzed yet;
//! they take a zero-cost off path. Three rules generate everything in this
//! file — hold these, and read the rest as their application to specific
//! constructs:
//!
//! 1. **A reference either EXECUTES or sits inside a REMOVABLE
//!    declaration.** Executing references are roots. References inside a
//!    candidate declaration's deferred region (its function body) are edges
//!    `(owner -> target)` that count only if the owner is live. Propagating
//!    from the roots reaches exactly the live set; a dead cycle is simply
//!    never reached — there is no cycle detection anywhere.
//! 2. **If removal cannot fully delete a site, its symbols must count as
//!    live** (force-root). Deadness is consumed per SYMBOL but removal
//!    happens per SITE — and `var`s redeclare — so one undeletable site
//!    must protect every site of the symbol.
//! 3. **The tree mutates under the analysis**, so re-collect every pass,
//!    and force-root whatever an in-pass collection provably could not
//!    have seen.
//!
//! ## Rule 1 applied: candidacy and the gate-mirror invariant
//!
//! Candidacy is FUNCTION DECLARATIONS ONLY. Declarator and class candidacy
//! were built, hardened over several review rounds, and then measured to
//! contribute exactly zero output bytes on 17 real artifacts (the 10-bundle
//! minsize corpus plus 7 modern ESM artifacts, including a real
//! rolldown-treeshaken Vue chunk) while owning the majority of the
//! analysis's hazard surface — statement-relocation movers, init-shape
//! staleness, class-removability fold-flips — so they were cut. Real-world
//! dead cycles are function-declaration cycles.
//!
//! Candidacy must be a SUBSET of what the removal sites in
//! `remove_unused_declaration.rs` delete cleanly, with zero residue: a dead
//! cycle is removed whole in one pass, and a blocked member would leave
//! references to deleted peers (ReferenceError). The definitions are shared
//! structurally so the two sides cannot drift:
//!
//! - global gates: `can_remove_unused_declarators` delegates to
//!   [`removal_enabled`] (`unused != Keep` plus the root
//!   `ScopeFlags::DirectEval` skip — the flag propagates to the root from
//!   any direct eval, subsuming per-site checks); the collection arms on
//!   [`collection_enabled`], a strict SUBSET that adds the module-only
//!   condition, so candidacy is only ever granted where removal is allowed;
//! - function declarations always drop whole — there is no shape axis for
//!   them, which is exactly why functions-only candidacy is cheap to keep
//!   sound.
//!
//! Non-candidacy is self-correcting: no region opens, so the declaration's
//! interior references record as roots — the conservative direction.
//!
//! ## Rule 2 applied: observability the reference count cannot express
//!
//! Non-candidate declarations cannot be removed BY THE ANALYSIS, but their
//! observability can exceed what references express (an export-wrapped
//! function is observed by importers). Such declarations force-root their
//! symbols at the hook.
//!
//! ## Rule 3 applied: cadence and the force-root log
//!
//! `Normalize` seeds the first set — source-level cycles are removable in
//! pass 1, before the loop's rewrites can spoil their candidate shape.
//! Every peephole pass then re-collects while it mutates, and every flush
//! ([`propagate_collected`]) publishes a fresh set, so the set a pass
//! consumes is always exactly one pass stale. Freshness is load-bearing:
//! rewrites EXPOSE cycles mid-loop (a call dropped once its callee is
//! proven pure, a folded define-replacement branch), and every cheaper
//! cadence was built and falsified — a standalone walk per pass cost
//! +15-22% wall; lazy/fixed-point-tail consumption lost output (exposed
//! cycles perish); one-shot and two-shot variants failed monitor-oxc's
//! idempotency gate on real npm packages (js_of_ocaml bundles), because
//! any fixed number of early collections stays one rewrite behind. A
//! record-time edge filter was also net-negative; junk edges are dropped
//! post-collection instead.
//!
//! In-pass collection can diverge from a settled-tree walk in one dangerous
//! direction — publishing a live-referenced symbol as dead — and the single
//! rewrite that can cause it is logged into
//! [`LivenessCollect::force_root_log`] and force-rooted at the flush: a
//! bound reference MINTED behind the traversal cursor (each minting pass
//! logs at its call site — the typeof rewrite in
//! `substitute_alternate_syntax` is the only minter today, and the debug
//! ground-truth validator flags an unlogged mint anywhere in the corpus).
//! Function-declaration sites have no other staleness axis: statement
//! movers relocate whole function declarations between slots
//! `exit_statement` visits either way, folds cannot change what a function
//! declaration is, and substitution never rewrites one. (Declarator
//! candidacy needed a relocation log, a substitution log, and a settled-
//! shape gate for exactly these axes — that machinery left with it.)
//!
//! ## The oracle
//!
//! In debug builds every flushed set is validated against an INDEPENDENT
//! re-derivation on the settled tree (`validate.rs`), so the whole test and
//! conformance corpus doubles as a differ between the in-pass collection
//! and ground truth. Read that file to CHECK this one, not to understand
//! it.
//!
//! ## Allocation discipline
//!
//! Everything sized by the program lives in the arena, like
//! `PassDirty::dead_refs`. References are filtered at record time on
//! candidate-kind `SymbolFlags` — `Function` only, so locals and parameters
//! never earn storage; only references to function declarations record at
//! all (when the wider declarator/class candidacy existed, 77-84% of edges
//! were unusable junk). Roots are deduplicated at record time by marking
//! them live immediately, and the edge "graph" is a flat list sorted by
//! source symbol, range-scanned via `partition_point` — no per-candidate
//! allocations. The in-pass collection buffers are `clear()`-reused across
//! passes, keeping system allocations untouched.

use oxc_allocator::{Allocator, BitSet, GetAllocator, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_syntax::symbol::{SymbolFlags, SymbolId};

use crate::{CompressOptions, CompressOptionsUnused, TraverseCtx, generated::ancestor::Ancestor};

// ========================================================================
// Shared predicates: the gate definitions (rules 1 and 2)
// ========================================================================

/// Symbol kinds a liveness candidate can have: function declarations only
/// (see the module doc for why declarator and class candidacy were cut). A
/// necessary (not sufficient) condition for candidacy; doubles as the
/// record-time reference filter, so references to anything else earn no
/// storage.
const CANDIDATE_KINDS: SymbolFlags = SymbolFlags::Function;

/// Whether unused-declaration removal is enabled for this program state.
/// The single definition of the global gates: `can_remove_unused_declarators`
/// (the removal sites' guard) delegates here, so candidacy and removal
/// cannot drift. Any direct eval flags the root via ancestor propagation —
/// see `refresh_direct_eval_flags`.
pub fn removal_enabled(scoping: &Scoping, options: &CompressOptions) -> bool {
    options.unused != CompressOptionsUnused::Keep
        && !scoping.root_scope_flags().contains_direct_eval()
}

/// Whether the liveness COLLECTION arms for this program: [`removal_enabled`]
/// plus ESM modules only. `source_type` is the PARSER-RESOLVED kind
/// (ambiguous `.js`/`.ts` resolves by content), so anything that is not
/// certainly a strict ES module — scripts, CommonJS,
/// ambiguous-resolved-script — takes the existing zero-cost off path: no
/// allocation, no per-node work, and the count-based removal arm behaves
/// exactly as on `main`. Sloppy sources add observability the reference
/// count cannot express (script-globals, Annex B block-function aliases);
/// enabling them is deliberate follow-up work, not a missing `!`.
///
/// A strict subset of [`removal_enabled`], which keeps the gate-mirror
/// invariant structural: candidacy (collection) is only ever granted where
/// removal is already allowed.
pub fn collection_enabled(
    scoping: &Scoping,
    source_type: SourceType,
    options: &CompressOptions,
) -> bool {
    source_type.is_module() && removal_enabled(scoping, options)
}

// ========================================================================
// Propagation: worklist over the flat edge list (rule 1)
// ========================================================================

/// Worklist propagation shared by the standalone walk and the in-pass
/// collection. The flat edge list sorted by source symbol is the adjacency
/// "map": a live symbol's targets are one `partition_point` range scan away.
/// Roots are already marked live (record-time dedup); marking non-candidate
/// targets live is harmless — only candidates are consulted for deadness.
fn propagate(
    candidates: &BitSet<'_>,
    live: &mut BitSet<'_>,
    worklist: &mut ArenaVec<'_, SymbolId>,
    edges: &mut ArenaVec<'_, (SymbolId, SymbolId)>,
) {
    edges.sort_unstable_by_key(|&(from, _)| from.index());
    while let Some(symbol_id) = worklist.pop() {
        // Only candidates have outgoing edges by construction.
        if !candidates.contains(symbol_id.index()) {
            continue;
        }
        let start = edges.partition_point(|&(from, _)| from.index() < symbol_id.index());
        for &(from, target) in &edges[start..] {
            if from != symbol_id {
                break;
            }
            if !live.contains(target.index()) {
                live.set_bit(target.index());
                worklist.push(target);
            }
        }
    }
}

// ========================================================================
// Collection: rides Normalize and every peephole pass (rule 3)
// ========================================================================

/// Per-pass in-traversal liveness collection: the peephole `Traverse` hooks
/// record candidates, roots, and edges as the pass visits each node, so no
/// standalone walk is needed to refresh the dead set. All buffers are
/// `clear()`-reused across passes.
pub struct LivenessCollect<'a> {
    /// Collection is running this pass. Set at `enter_program` from
    /// [`collection_enabled`]; stable for the whole pass, so every hook's
    /// enter/exit pair is balanced.
    active: bool,
    /// Candidates admitted this pass (per-site gates at enter time, on the
    /// pre-fold shape — folds only purify, so this under-approves relative
    /// to the settled tree: the sound direction, self-correcting next pass).
    candidates: BitSet<'a>,
    /// Marked at record time for roots, during propagation for edge targets.
    live: BitSet<'a>,
    /// Deduplicated roots; doubles as the propagation worklist.
    roots: ArenaVec<'a, SymbolId>,
    /// `(innermost enclosing candidate, referenced candidate-kind symbol)`.
    edges: ArenaVec<'a, (SymbolId, SymbolId)>,
    /// Saved `current_candidate` values; one frame per FUNCTION node,
    /// pushed at enter, popped at exit.
    frames: ArenaVec<'a, Option<SymbolId>>,
    /// Innermost candidate whose deferred region the traversal is inside.
    current_candidate: Option<SymbolId>,
    /// Scratch for the next dead set; swapped with
    /// `MinifierState::dead_symbols` at flush.
    dead_next: BitSet<'a>,
    /// Scratch for the next PINNED set; swapped with
    /// Symbols this pass's collection may have under-observed, force-rooted
    /// at the flush. One producer: a bound reference minted behind the
    /// traversal cursor (each minting pass logs at its call site —
    /// `substitute_alternate_syntax`'s typeof rewrite is the only one
    /// today, and the debug ground-truth validator flags an unlogged mint
    /// across the whole corpus). An unlogged mint would otherwise publish a
    /// live-referenced symbol as dead. Cleared every pass.
    force_root_log: ArenaVec<'a, SymbolId>,
}

impl<'a> LivenessCollect<'a> {
    /// Everything starts zero-capacity (like `MinifierState::dead_symbols`):
    /// the first ACTIVE `reset_for_pass` sizes the bitsets, so a compile with
    /// the analysis off (`unused: Keep`, root direct eval) never allocates.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            active: false,
            candidates: BitSet::new_in(0, allocator),
            live: BitSet::new_in(0, allocator),
            roots: ArenaVec::new_in(&allocator),
            edges: ArenaVec::new_in(&allocator),
            frames: ArenaVec::new_in(&allocator),
            current_candidate: None,
            dead_next: BitSet::new_in(0, allocator),
            force_root_log: ArenaVec::new_in(&allocator),
        }
    }

    fn reset_for_pass(&mut self, active: bool, symbols_len: usize, allocator: &'a Allocator) {
        self.active = active;
        self.current_candidate = None;
        self.frames.clear();
        // A pass observes rewrites made DURING it; anything already in the
        // log predates this pass's traversal, which will visit those
        // references as part of the tree. Cleared even when inactive so the
        // log stays bounded with the analysis off.
        self.force_root_log.clear();
        if !active {
            return;
        }
        self.roots.clear();
        self.edges.clear();
        // Symbols minted mid-pass are past these capacities and read as
        // live everywhere (the `PassDirty::dead_refs` convention); their
        // declarations enter the analysis next pass.
        for bits in [&mut self.candidates, &mut self.live, &mut self.dead_next] {
            if bits.capacity() == symbols_len {
                bits.clear();
            } else {
                *bits = BitSet::new_in(symbols_len, allocator);
            }
        }
    }

    /// Force-root `symbol_id` at this pass's flush; see the
    /// `force_root_log` field doc.
    /// No-op while collection is off (the flush would discard the entry
    /// anyway), so minting call sites need no gate of their own.
    pub fn log_force_root(&mut self, symbol_id: SymbolId) {
        if self.active {
            self.force_root_log.push(symbol_id);
        }
    }

    /// Mark a symbol live and enqueue it for propagation; deduplicated at
    /// record time.
    fn mark_live_root(&mut self, symbol_id: SymbolId) {
        let index = symbol_id.index();
        if index < self.live.capacity() && !self.live.has_bit(index) {
            self.live.set_bit(index);
            self.roots.push(symbol_id);
        }
    }

    /// Admit a candidacy-eligible declaration; refuses mid-pass-minted
    /// symbols (past capacity — they read as live and retry next pass).
    fn admit_candidate(&mut self, symbol_id: SymbolId) -> bool {
        let index = symbol_id.index();
        if index < self.candidates.capacity() {
            self.candidates.set_bit(index);
            true
        } else {
            false
        }
    }
}

/// `Traverse::enter_program` body for both collecting traversals —
/// `Normalize` (whose collection produces the initial dead set for pass 1)
/// and every peephole pass: reset the collection for the pass.
pub fn begin_pass(ctx: &mut TraverseCtx<'_>) {
    let allocator = ctx.allocator();
    let TraverseCtx { state, scoping, .. } = ctx;
    let enabled = collection_enabled(scoping.scoping(), state.source_type, &state.options);
    let symbols_len = scoping.scoping().symbols_len();
    state.liveness.reset_for_pass(enabled, symbols_len, allocator);
}

/// `Traverse::enter_identifier_reference` body. Runs for every
/// `IdentifierReference` (including assignment targets), so the inactive
/// path is one load and branch.
pub fn collect_identifier_reference<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &mut TraverseCtx<'a>,
) {
    let TraverseCtx { state, scoping, .. } = ctx;
    let lv = &mut state.liveness;
    if !lv.active {
        return;
    }
    let scoping = scoping.scoping();
    let Some(reference_id) = ident.reference_id.get() else { return };
    let Some(symbol_id) = scoping.get_reference(reference_id).symbol_id() else { return };
    // Kind-based edge-vs-root: a reference to a candidate-kind target inside
    // a deferred region is an edge whether or not the target is admitted
    // this pass — its candidacy bit lands via its own declaration hook in
    // the same pass, so a newly-eligible dead symbol is found at this very
    // flush. References to every other kind (locals, parameters, imports,
    // classes) earn no storage at all.
    if !scoping.symbol_flags(symbol_id).intersects(CANDIDATE_KINDS) {
        return;
    }
    match lv.current_candidate {
        None => lv.mark_live_root(symbol_id),
        Some(from) => lv.edges.push((from, symbol_id)),
    }
}

fn parent_is_export(parent: Ancestor<'_, '_>) -> bool {
    matches!(
        parent,
        Ancestor::ExportNamedDeclarationDeclaration(_)
            | Ancestor::ExportDefaultDeclarationDeclaration(_)
    )
}

/// `Traverse::enter_function` body: decide candidacy at the same position
/// the removal site evaluates (the hook fires before the function's scope is
/// entered, so `current_scope_id` is the containing scope — identical to the
/// `exit_statement` frame that runs `remove_unused_function_declaration`).
/// The only position failure a module can express is an export wrapper.
pub fn collect_enter_function<'a>(func: &Function<'a>, ctx: &mut TraverseCtx<'a>) {
    if !ctx.state.liveness.active {
        return;
    }
    let mut candidate = None;
    if func.is_declaration()
        && let Some(symbol_id) = func.id.as_ref().and_then(|id| id.symbol_id.get())
    {
        if parent_is_export(ctx.parent()) {
            ctx.state.liveness.mark_live_root(symbol_id);
        } else if ctx.state.liveness.admit_candidate(symbol_id) {
            candidate = Some(symbol_id);
        }
    }
    let lv = &mut ctx.state.liveness;
    lv.frames.push(lv.current_candidate);
    if candidate.is_some() {
        lv.current_candidate = candidate;
    }
}

/// `Traverse::exit_function` body: close the region frame the enter hook
/// opened. Functions are the only region-openers.
pub fn collect_exit_function(ctx: &mut TraverseCtx<'_>) {
    let lv = &mut ctx.state.liveness;
    if lv.active {
        debug_assert!(!lv.frames.is_empty(), "unbalanced liveness region frames");
        lv.current_candidate = lv.frames.pop().flatten();
    }
}

/// Consume the pass's collection: force-root symbols that received freshly
/// minted references (the one toward-dead divergence of in-pass collection),
/// propagate liveness, refresh `MinifierState::dead_symbols`, and report
/// whether the driver must run another pass: a NEW dead symbol appeared,
/// and its removal must be consumed with the same one-pass freshness as
/// everything else. Convergence is bounded by dead-set growth alone — dead
/// bits only ever come from the symbol table, and a quiet re-run of an
/// unchanged tree finds no new ones.
///
/// Must run after `flush_pass_dirty` so consumed reference counts are
/// post-flush fresh.
pub fn propagate_collected(ctx: &mut TraverseCtx<'_>) -> bool {
    let TraverseCtx { state, .. } = ctx;
    let crate::state::MinifierState { liveness, dead_symbols, .. } = state;
    let lv = liveness;
    if !lv.active {
        return false;
    }
    debug_assert!(lv.frames.is_empty(), "unbalanced liveness region frames at flush");

    // References minted behind the traversal cursor were never visited by
    // the collection and must stay live this flush (see the
    // `force_root_log` field doc).
    while let Some(symbol_id) = lv.force_root_log.pop() {
        lv.mark_live_root(symbol_id);
    }

    // No candidates admitted ⇒ no edges (an edge needs an enclosing
    // candidate region) and no dead set: skip the worklist drain and the
    // set scan (mirrors `compute_dead_symbols`' early-out). The swap still
    // runs so a stale dead set from the previous pass clears.
    if lv.candidates.is_empty() {
        std::mem::swap(dead_symbols, &mut lv.dead_next);
        return false;
    }

    // Candidacy is fully known post-collection, so drop the (measured
    // 77-84%) edges whose targets can never be dead before they hit the
    // sort. A removed edge's only effect was a live mark on a
    // non-candidate, which nothing consults.
    {
        let LivenessCollect { edges, candidates, .. } = lv;
        edges.retain(|&(_, target)| candidates.contains(target.index()));
    }

    propagate(&lv.candidates, &mut lv.live, &mut lv.roots, &mut lv.edges);

    for candidate in lv.candidates.ones() {
        if !lv.live.contains(candidate) {
            lv.dead_next.set_bit(candidate);
        }
    }

    // Per-bit scan; a word-level set-difference helper would make this
    // O(words) — a possible `oxc_allocator` follow-up. Until then, gate the
    // common terminal-pass case (empty dead set).
    let found_new_dead =
        !lv.dead_next.is_empty() && lv.dead_next.ones().any(|bit| !dead_symbols.contains(bit));

    std::mem::swap(dead_symbols, &mut lv.dead_next);
    found_new_dead
}
