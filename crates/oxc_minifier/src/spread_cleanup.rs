//! Removal-only cleanup for dead object-copy spreads of getter-free literals.
//!
//! `({ ...Proto })` in discarded position is kept whenever the spread argument
//! is an identifier, because copying a binding's own enumerable properties runs
//! that object's getters. When the binding provably still holds a getter-free
//! object literal, there are no getters to run and the copy is dead code — the
//! pattern from rolldown/rolldown#8582, which keeps whole modules alive.
//!
//! Proving it needs a whole-symbol fact ("nothing else can have installed an
//! accessor"), but the minifier's symbol data is per-pass scratch that is only
//! coherent at a pass boundary. So the fact is never computed mid-traversal:
//!
//! ```text
//! ordinary peephole pass
//!   ├─ changed → discard this pass's marks and continue
//!   └─ quiet   → census against the settled references
//!                  ↓
//!               publish candidates, run one cleanup pass
//!                  ↓
//!               removals reopen ordinary DCE
//! ```
//!
//! During traversal each object literal records the `ReferenceId` of every
//! direct `...Identifier` argument it copies — at the spread node, so the cost
//! is one bit per spread site rather than work per identifier. A pass that
//! changed anything throws its marks away; only a QUIET pass gets to consume
//! them, and a quiet pass by definition ended with the AST it started with, so
//! its marks, its `SymbolValue` entries, its liveness data and the post-flush
//! `Scoping` all describe the same program.
//!
//! The candidate test is then a positive census — *every* live reference of the
//! symbol is one of those marks — rather than a list of read positions deemed
//! harmless. Anything that is not literally an object-copy spread disqualifies
//! the symbol: a call argument, a member write, an array spread, a reference
//! this pass never saw. That is what keeps the fact honest as the AST changes
//! underneath it, and it needs no sanction list to maintain.

use oxc_allocator::{Allocator, BitSet, Vec as ArenaVec};
#[cfg(debug_assertions)]
use oxc_ast_visit::Visit;

use oxc_ast::ast::*;
use oxc_semantic::Scoping;
use oxc_syntax::{reference::ReferenceId, scope::ScopeFlags, symbol::SymbolId};

use crate::{TraverseCtx, generated::ancestor::Ancestor, symbol_value::FreshValueKind};

/// Per-run state for the quiet-pass object-spread cleanup.
pub struct SpreadCleanup<'a> {
    /// References the current pass saw as the direct `...Identifier` argument
    /// of an object literal, and that passed the mark-time proof.
    ///
    /// Reset at the start of every pass. Marks from a pass that changed the AST
    /// are never consumed: consumption only follows a quiet pass, whose own
    /// marks are complete and current.
    ///
    /// Sized once from the initial `references_len()`. A reference minted
    /// during the loop is past capacity and so cannot be marked, which makes
    /// the census treat it as an unknown use and reject its symbol — the
    /// conservative direction, and the same capacity guard
    /// `PassChanges::removed_references` relies on.
    ///
    /// FORWARD OBLIGATION. Every soundness argument here reduces to one
    /// invariant: a transform that moves a marked identifier into an escaping
    /// position necessarily records a pass change, so the pass is not quiet and
    /// these marks are discarded unread. Any transform that changes which node
    /// a `ReferenceId` occupies must therefore record a change. That is a
    /// convention, not a mechanism — the one audited exception is
    /// `dead_drop_mutates_ast` (`minimize_statements.rs`), which suppresses the
    /// flag for identity drops of initializer-free `var` declarations to keep
    /// the fixed-point loop from oscillating. Those declarations have no
    /// initializer and so cannot relocate a spread.
    marks: BitSet<'a>,

    /// The subset of [`Self::marks`] whose enclosing object literal is itself
    /// discarded, so removing the copy is something the following cleanup pass
    /// can actually do.
    ///
    /// Purely an economy: a candidate whose copies all sit in live positions
    /// (`const Q = { ...P };` with `Q` used) is real but useless, and
    /// publishing it costs a full extra traversal that removes nothing. Only
    /// [`Self::marks`] takes part in the census, so narrowing this set can lose
    /// an optimization but never make one unsound.
    removable_marks: BitSet<'a>,

    /// Symbols the last quiet-pass boundary proved to hold a getter-free object
    /// literal whose every live reference is a dead object-copy spread.
    ///
    /// Published by [`settle_candidates`], read by the single pass that follows
    /// it, then dropped. Never carried across a boundary: the fact is about one
    /// settled AST, and re-deriving it is what makes the construction
    /// idempotent.
    candidates: BitSet<'a>,

    /// Whether [`Self::candidates`] holds anything, so the per-spread test on
    /// the removal path is a branch rather than a scan.
    has_candidates: bool,
}

impl<'a> SpreadCleanup<'a> {
    pub fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        Self {
            marks: BitSet::new_in(scoping.references_len(), allocator),
            removable_marks: BitSet::new_in(scoping.references_len(), allocator),
            candidates: BitSet::new_in(scoping.symbols_len(), allocator),
            has_candidates: false,
        }
    }

    /// Discard the previous pass's marks. Called at the start of every pass.
    pub fn begin_pass(&mut self) {
        self.marks.clear();
        self.removable_marks.clear();
    }

    /// Drop the published candidates and report whether there were any, i.e.
    /// whether the pass that just finished was a cleanup pass.
    pub fn take_candidates(&mut self) -> bool {
        if !self.has_candidates {
            return false;
        }
        self.candidates.clear();
        self.has_candidates = false;
        true
    }
}

/// Record one `...Identifier` object-copy use.
///
/// Called from `PeepholeOptimizations::exit_spread_element`, so the cost is
/// bounded by the number of spread sites rather than by the properties of every
/// object literal. Array and call spreads reach the same hook, hence the parent
/// test: only an object literal's own `properties` list is a copy.
///
/// The generated walk fires this exactly once per node per pass, in traversal
/// order — `minimize_statements` and its helpers re-process already-walked
/// statements with plain functions and never re-enter the traversal dispatch.
///
/// A spread that fails the mark-time proof is simply left unmarked, which makes
/// the census reject its symbol.
pub fn mark_object_copy_spread<'a>(spread: &SpreadElement<'a>, ctx: &mut TraverseCtx<'a>) {
    // The mark-time order proof and `PeepholeOptimizations::summary_order_proven`
    // read the same signal — "a `SymbolValue` entry exists right now" — as
    // evidence that the declarator was traversed earlier this pass. That
    // consumer needs the flag because it runs from positions reached during
    // statement-list re-processing; marking is a traversal hook and cannot be,
    // which is exactly what this asserts. Move marking to `exit_expression` and
    // the assertion is what tells you the proof no longer holds.
    debug_assert!(
        !ctx.state.reprocessing_statements,
        "spread marks require traversal-order recording; statement-list \
         re-processing invalidates the declarator-precedes-use proof",
    );
    if !matches!(ctx.parent(), Ancestor::ObjectExpressionProperties(_)) {
        return;
    }
    let Expression::Identifier(ident) = &spread.argument else { return };
    let Some(reference_id) = ident.reference_id.get() else { return };
    let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
    if !mark_time_proof_holds(symbol_id, ctx) {
        return;
    }
    let index = reference_id.index();
    if index < ctx.state.spread_cleanup.marks.capacity() {
        ctx.state.spread_cleanup.marks.set_bit(index);
        if enclosing_literal_is_discarded(ctx) {
            ctx.state.spread_cleanup.removable_marks.set_bit(index);
        }
    }
}

/// Whether the object literal holding the spread being exited is itself in a
/// discarded position, i.e. one `remove_unused_expression` reaches.
///
/// The ancestor above `ObjectExpressionProperties` is the literal's own parent.
/// An expression statement discards it outright; a sequence element discards
/// every operand but the last, and treating the last one as discarded too is
/// harmless here — this only decides whether publishing a candidate is worth a
/// pass, never whether removing it is legal.
fn enclosing_literal_is_discarded(ctx: &TraverseCtx<'_>) -> bool {
    matches!(
        ctx.ancestors().nth(1),
        Some(
            Ancestor::ExpressionStatementExpression(_) | Ancestor::SequenceExpressionExpressions(_)
        )
    )
}

/// Whether this spread may stand as evidence about the symbol's value.
///
/// Both halves are checked here rather than at the boundary because here the
/// traversal cursor IS the spread's own position — at the boundary there is no
/// cursor to ask.
///
/// - Declaration before use: `SymbolState::values` is per-pass scratch written
///   by `exit_variable_declarator`, so an entry existing at this spread proves
///   the declarator precedes it in this pass's traversal.
/// - Same function: an entry orders the traversal, not execution. A nested
///   function can run before a hoisted `var`'s initializer, when the binding
///   holds `undefined` rather than the classified literal.
///
/// Both are stricter than the object case alone needs — spreading `undefined`
/// is a no-op, and a `const` read before its declarator is a TDZ
/// `ReferenceError` inside the documented `No TDZ Violation` assumption — but
/// they are what lets the census be a plain membership test, with no reasoning
/// about which reads observe which value.
fn mark_time_proof_holds(symbol_id: SymbolId, ctx: &TraverseCtx<'_>) -> bool {
    if ctx.state.symbols.value(symbol_id).is_none() {
        return false;
    }
    let scoping = ctx.scoping();
    let symbol_scope_id = scoping.symbol_scope_id(symbol_id);
    !scoping
        .scope_ancestors(ctx.current_scope_id())
        .take_while(|&scope_id| scope_id != symbol_scope_id)
        .any(|scope_id| scoping.scope_flags(scope_id).is_function())
}

/// Publish the symbols whose dead object-copy spreads the next pass may remove,
/// and report whether there are any.
///
/// Called only at a quiet-pass boundary, after `finish_pass` has flushed the
/// pass's changes, so `Scoping` lists exactly the references the settled AST
/// contains.
///
/// A symbol qualifies when all of the following hold:
///
/// - its per-pass value entry classifies it `FreshValueKind::Object`, which
///   `fresh_value_kind` grants only to a literal with no getter, no setter and
///   no `__proto__:` key, and which `init_value` withholds entirely from a
///   symbol with more than one value declaration;
/// - it has no resolved write, so nothing replaces the classified value;
/// - its declaring scope contains no direct `eval`, which could reach the
///   binding without a resolved reference;
/// - liveness data exists and does not mark it implicitly observable (a module
///   export, a script global, an Annex B alias, a `using` resource). Absent
///   liveness means observability is unknown, which is not a candidate;
/// - the census: every live resolved reference is one of this pass's marks. An
///   ordinary read, a call argument, a member write, an array spread, or a
///   reference minted after the last flush is not a mark, and any one of them
///   disqualifies the symbol.
pub fn settle_candidates(
    #[cfg_attr(not(debug_assertions), expect(unused_variables))] program: &Program<'_>,
    ctx: &mut TraverseCtx<'_>,
) -> bool {
    // Arena-allocated so a boundary costs no system allocation; the marked
    // symbols must be read out before `candidates` can be written to.
    let mut marked_symbols = ArenaVec::new_in(&*ctx);
    for index in ctx.state.spread_cleanup.marks.ones() {
        if let Some(symbol_id) =
            ctx.scoping().get_reference(ReferenceId::from_usize(index)).symbol_id()
        {
            marked_symbols.push(symbol_id);
        }
    }
    let mut found_candidates = false;
    for symbol_id in marked_symbols {
        if !ctx.state.spread_cleanup.candidates.contains(symbol_id.index())
            && is_candidate(symbol_id, ctx)
        {
            ctx.state.spread_cleanup.candidates.set_bit(symbol_id.index());
            found_candidates = true;
        }
    }
    ctx.state.spread_cleanup.has_candidates = found_candidates;
    #[cfg(debug_assertions)]
    if found_candidates {
        debug_assert_census_complete(program, ctx);
    }
    found_candidates
}

/// Debug-only guard on the census's core premise: `Scoping` lists EVERY live
/// reference of a candidate symbol.
///
/// The dangerous direction is a live reference to a candidate that the census
/// never saw — an escape channel hidden behind an id the resolved-reference
/// list omits, or a node whose `ReferenceId` is aliased by another. Either
/// would let a copy be removed whose value was mutated behind the census's
/// back. This is the same shape as `debug_assert_no_over_prune`, and costs
/// nothing in release.
///
/// Two mechanisms could produce it, and both hold today: the minifier's only
/// `clone_in` (`substitute_alternate_syntax.rs`) resets `reference_id` and
/// mints fresh ones, and `remove_unused_object_expr`'s literal rebuilding always
/// exits through `ctx.replace_expression`, which records a change and so cannot
/// happen on a quiet pass. Neither is enforced by anything but this walk.
#[cfg(debug_assertions)]
fn debug_assert_census_complete(program: &Program<'_>, ctx: &TraverseCtx<'_>) {
    struct CensusCheck<'b, 'c> {
        ctx: &'b TraverseCtx<'c>,
    }
    impl<'a> Visit<'a> for CensusCheck<'_, '_> {
        fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
            let Some(reference_id) = it.reference_id.get() else { return };
            let Some(symbol_id) = self.ctx.scoping().get_reference(reference_id).symbol_id() else {
                return;
            };
            if !self.ctx.state.spread_cleanup.candidates.contains(symbol_id.index()) {
                return;
            }
            assert!(
                self.ctx.state.spread_cleanup.marks.contains(reference_id.index()),
                "spread census incompleteness: reference {} of candidate symbol {} is live in \
                 the program but was never marked as an object copy — `Scoping` did not list \
                 it, or two nodes share the id",
                reference_id.index(),
                symbol_id.index(),
            );
        }
    }
    CensusCheck { ctx }.visit_program(program);
}

fn is_candidate(symbol_id: SymbolId, ctx: &TraverseCtx<'_>) -> bool {
    let Some(value) = ctx.state.symbols.value(symbol_id) else { return false };
    if value.kind != FreshValueKind::Object || value.references.has_writes() {
        return false;
    }
    let scoping = ctx.scoping();
    if scoping.scope_flags(scoping.symbol_scope_id(symbol_id)).contains(ScopeFlags::DirectEval) {
        return false;
    }
    let Some(liveness) = ctx.state.symbols.liveness() else { return false };
    if liveness.is_implicitly_observable(symbol_id) {
        return false;
    }
    let reference_ids = scoping.get_resolved_reference_ids(symbol_id);
    reference_ids
        .iter()
        .all(|reference_id| ctx.state.spread_cleanup.marks.contains(reference_id.index()))
        && reference_ids.iter().any(|reference_id| {
            ctx.state.spread_cleanup.removable_marks.contains(reference_id.index())
        })
}

/// Whether this object property is a spread the last quiet-pass boundary proved
/// dead, and may therefore be dropped from a discarded object literal.
///
/// Candidacy survives everything the cleanup pass does to the AST. A transform
/// can delete references, which only shrinks the census; it can move a spread
/// wholesale, which leaves it a spread; and none converts a spread use into a
/// use of another shape. A transform that ever does — one that rewrites
/// `({ ...P })` into something reading `P` in another position — must clear the
/// candidate set, or this fact stops being true mid-pass.
pub fn is_dead_object_copy_spread<'a>(
    property: &ObjectPropertyKind<'a>,
    ctx: &TraverseCtx<'a>,
) -> bool {
    let ObjectPropertyKind::SpreadProperty(spread) = property else { return false };
    is_dead_object_copy_spread_argument(&spread.argument, ctx)
}

/// [`is_dead_object_copy_spread`] against a spread argument already
/// destructured out of its property.
pub fn is_dead_object_copy_spread_argument<'a>(
    argument: &Expression<'a>,
    ctx: &TraverseCtx<'a>,
) -> bool {
    if !ctx.state.spread_cleanup.has_candidates {
        return false;
    }
    let Expression::Identifier(ident) = argument else { return false };
    let Some(reference_id) = ident.reference_id.get() else { return false };
    ctx.scoping()
        .get_reference(reference_id)
        .symbol_id()
        .is_some_and(|symbol_id| ctx.state.spread_cleanup.candidates.contains(symbol_id.index()))
}
