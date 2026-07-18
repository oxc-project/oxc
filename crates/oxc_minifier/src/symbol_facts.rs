use rustc_hash::FxHashMap;

use oxc_syntax::symbol::SymbolId;

/// The strongest program-wide effect recorded for member writes to a symbol.
///
/// Effects form a monotone order. A possible prototype mutation is also an
/// ordinary write hazard, so recording it must preserve both consumers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberWriteEffect {
    #[default]
    None,
    /// A compound, logical, update, chained, or otherwise hazardous member
    /// operation can observe a sibling property write.
    Hazard,
    /// A `__proto__` or unsafe computed-key write may install a setter that
    /// makes a later property write observable.
    ///
    /// Normalize seeds this completely: fixed-point transforms cannot create
    /// a proto-write key that was not already covered. Key folding starts from
    /// a conservatively unsafe key, while forming a compound assignment changes
    /// only the operator, not the key.
    MayMutatePrototype,
}

impl MemberWriteEffect {
    pub fn is_hazardous(self) -> bool {
        self >= Self::Hazard
    }

    pub fn may_mutate_prototype(self) -> bool {
        self >= Self::MayMutatePrototype
    }
}

/// Program-wide, monotone member-write effects keyed by `SymbolId`.
///
/// Lifecycle contract:
/// - seeded by Normalize before the fixed-point loop, so membership is
///   execution-order independent;
/// - extended mid-loop only where a pass creates a stronger effect;
/// - never downgraded or cleared between peephole passes;
/// - stale stronger effects only forgo an optimization, but a missing effect is
///   unsound. Seeding must therefore complete before the loop, and creation
///   sites must record stronger effects eagerly.
#[derive(Debug, Default)]
pub struct PersistentSymbolFacts {
    effects: FxHashMap<SymbolId, MemberWriteEffect>,
}

impl PersistentSymbolFacts {
    #[inline]
    pub fn record(&mut self, symbol_id: SymbolId, effect: MemberWriteEffect) {
        let current = self.effects.entry(symbol_id).or_default();
        *current = (*current).max(effect);
    }

    #[inline]
    pub fn effect(&self, symbol_id: SymbolId) -> MemberWriteEffect {
        self.effects.get(&symbol_id).copied().unwrap_or_default()
    }
}
