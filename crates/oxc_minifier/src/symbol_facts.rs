use bitflags::bitflags;
use rustc_hash::FxHashMap;

use oxc_syntax::symbol::SymbolId;

bitflags! {
    /// A set of monotone facts held about a single symbol. See
    /// [`PersistentSymbolFacts`] for the lifecycle every bit must obey. Do not
    /// add a bit without a consumer.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct SymbolFact: u8 {
        /// The symbol has a hazardous member-write operation anywhere in the
        /// program: compound / logical-assignment / update / chained-delete ops,
        /// which READ the property before writing (so a sibling plain write's
        /// value is observable); bases of chained member writes (`a` in
        /// `a.b.c = 1`, whose intermediate object must not be dropped); and
        /// writes through `__proto__` or non-literal computed keys, which may
        /// install setters. The DEFAULT-mode drop of write-only property
        /// assignments (`remove_unused_member_assignment`) must skip such symbols.
        const MEMBER_WRITE_HAZARD = 1 << 0;

        /// The symbol has a potential `__proto__` write anywhere in the program:
        /// an explicit `.__proto__` static-key write, or a computed-key write
        /// whose key is unsafe per `member_key_is_safe` (not provably a string
        /// other than `"__proto__"`). Either can install a setter that makes a
        /// later property write on the symbol side-effectful. This deliberately
        /// reuses the hazard scan's key class: literal `o[null]`/`o[true]` keys
        /// can never coerce to `"__proto__"`, but distinguishing them would only
        /// drop more in shapes that don't occur in real code, at the cost of a
        /// second key classifier.
        ///
        /// A purely syntactic fact: consumed by the shared drop predicate
        /// (`is_member_assign_to_unused_binding`, guarding both the default and
        /// opt-in write-only property drops), which pairs it with a query-time
        /// reference count to exempt a sole-reference proto write.
        ///
        /// Seeded entirely by `Normalize`; the fixed-point loop cannot create a
        /// proto write Normalize did not see. A computed non-literal key is seeded
        /// before any folding, so a key later folded to the literal `"__proto__"`
        /// was already covered, and forming a compound assignment only changes the
        /// operator, never the key.
        const PROTO_WRITTEN = 1 << 1;
    }
}

/// Program-wide, monotone facts about symbols, keyed by `SymbolId`.
///
/// Lifecycle contract (also the fit criterion for any future [`SymbolFact`] bit):
/// - seeded by `Normalize` before the fixed-point loop, so membership is
///   execution-order independent (a per-pass set populated during the same
///   traversal that drops writes would be traversal-order dependent);
/// - extended mid-loop only where a pass CREATES a new fact instance (e.g.
///   forming a new compound assignment in `mark_assignment_target_as_read`);
/// - monotone: bits are only ever set, never cleared —
///   `PeepholeOptimizations::enter_program` deliberately leaves it alone;
/// - staleness is sound: a SET bit whose hazard has since been optimized away
///   only FORGOES an optimization, never enables a wrong one. A fact belongs
///   here only if that holds for it. (The reverse direction — a MISSING bit —
///   is unsound to act on, which is exactly why seeding must complete before
///   the loop and why creation sites must insert eagerly; monotonicity does
///   not cover it.)
#[derive(Debug, Default)]
pub struct PersistentSymbolFacts {
    facts: FxHashMap<SymbolId, SymbolFact>,
}

impl PersistentSymbolFacts {
    /// Record `fact` for `symbol_id`, unioned with any facts already held.
    #[inline]
    pub fn insert(&mut self, symbol_id: SymbolId, fact: SymbolFact) {
        self.facts.entry(symbol_id).or_default().insert(fact);
    }

    /// Whether `symbol_id` carries `fact`.
    #[inline]
    pub fn has(&self, symbol_id: SymbolId, fact: SymbolFact) -> bool {
        self.facts.get(&symbol_id).is_some_and(|f| f.contains(fact))
    }
}
