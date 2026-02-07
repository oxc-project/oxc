use bitflags::bitflags;

bitflags! {
    /// Rich side effect information for bundler tree-shaking.
    ///
    /// This provides more granular information than just a boolean,
    /// allowing bundlers to make more informed decisions about code elimination.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct SideEffectInfo: u8 {
        /// The expression/statement has a side effect.
        const SIDE_EFFECT = 1;
        /// The expression accesses a global variable.
        /// This is tracked separately because global variable access may affect
        /// execution order even when marked as pure.
        const GLOBAL_VAR_ACCESS = 1 << 1;
        /// The expression has a pure annotation (`/*#__PURE__*/` or `/*@__PURE__*/`).
        const PURE_ANNOTATION = 1 << 2;
    }
}

impl SideEffectInfo {
    /// Returns `true` if the expression/statement has a side effect.
    #[inline]
    pub fn has_side_effect(self) -> bool {
        self.contains(Self::SIDE_EFFECT)
    }

    /// Returns `true` if the expression accesses a global variable.
    #[inline]
    pub fn has_global_var_access(self) -> bool {
        self.contains(Self::GLOBAL_VAR_ACCESS)
    }

    /// Returns `true` if the expression has a pure annotation.
    #[inline]
    pub fn has_pure_annotation(self) -> bool {
        self.contains(Self::PURE_ANNOTATION)
    }

    /// Create info indicating a side effect.
    #[inline]
    pub fn side_effect() -> Self {
        Self::SIDE_EFFECT
    }

    /// Create info indicating no side effects.
    #[inline]
    pub fn no_side_effect() -> Self {
        Self::empty()
    }
}

impl From<bool> for SideEffectInfo {
    #[inline]
    fn from(has_side_effect: bool) -> Self {
        if has_side_effect { Self::SIDE_EFFECT } else { Self::empty() }
    }
}
