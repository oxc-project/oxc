use bitflags::bitflags;
use rustc_hash::FxHashMap;

use oxc_syntax::symbol::SymbolId;

bitflags! {
    /// A set of monotone facts held about a single symbol. See
    /// [`PersistentSymbolFacts`] for the lifecycle every bit must obey.
    ///
    /// Planned follow-up bits (do not add one without a consumer): `PROTO_WRITTEN`,
    /// migrating `MinifierState::proto_write_symbols` — recording it here (seeded
    /// by `Normalize`) would make it execution-order independent and fix that
    /// field's order-dependence TODO.
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
    }
}

/// Program-wide, monotone facts about symbols, keyed by `SymbolId`.
///
/// Lifecycle contract (also the fit criterion for any future [`SymbolFact`] bit):
/// - seeded by `Normalize` before the fixed-point loop, so membership is
///   execution-order independent (unlike the per-pass `proto_write_symbols` used
///   by the `property_write_side_effects: false` opt-in path);
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
