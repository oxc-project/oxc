/// What the minifier has proved about calls to a locally declared function.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FunctionSummary {
    /// No reusable call-site proof is available. For summary consumers, this
    /// is equivalent to having no persistent metadata entry for the symbol.
    #[default]
    Unknown,
    /// Calling the function has no side effects, but its result is unknown.
    SideEffectFree,
    /// Calling the function has no side effects and returns `undefined`.
    SideEffectFreeReturnsUndefined,
}

impl FunctionSummary {
    pub fn is_side_effect_free(self) -> bool {
        matches!(self, Self::SideEffectFree | Self::SideEffectFreeReturnsUndefined)
    }

    pub fn returns_undefined(self) -> bool {
        self == Self::SideEffectFreeReturnsUndefined
    }
}

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

/// Metadata that remains valid across peephole iterations for one symbol.
///
/// Function summaries may be replaced when a later pass proves more about the
/// declaration. Member-write effects are monotone: they are seeded before the
/// fixed-point loop, strengthened when a transform creates a hazard, and never
/// downgraded. A stale stronger effect only forgoes an optimization, but a
/// missing effect is unsound. Seeding must therefore complete before the loop,
/// and creation sites must record stronger effects eagerly.
#[derive(Debug, Default)]
pub struct PersistentSymbolMetadata {
    function_summary: FunctionSummary,
    member_write_effect: MemberWriteEffect,
}

impl PersistentSymbolMetadata {
    #[inline]
    pub fn set_function_summary(&mut self, summary: FunctionSummary) {
        self.function_summary = summary;
    }

    #[inline]
    pub fn function_summary(&self) -> FunctionSummary {
        self.function_summary
    }

    #[inline]
    pub fn record_member_write_effect(&mut self, effect: MemberWriteEffect) {
        self.member_write_effect = self.member_write_effect.max(effect);
    }

    #[inline]
    pub fn member_write_effect(&self) -> MemberWriteEffect {
        self.member_write_effect
    }
}
