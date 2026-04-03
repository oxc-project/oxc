//! Code generation context management
//!
//! This module provides the [`Context`] type for managing the state and configuration
//! during code generation, including JavaScript/TypeScript-specific syntax rules.

use bitflags::bitflags;

bitflags! {
    /// Code generation context flags
    ///
    /// Controls various aspects of code generation including operator precedence,
    /// language features, and syntax restrictions.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Context: u8 {
        /// Forbid the `in` operator in expressions
        ///
        /// Used in contexts where the `in` operator could be ambiguous,
        /// such as in the init clause of a for loop.
        const FORBID_IN   = 1 << 0;
        /// Forbid call expressions
        ///
        /// Used to prevent ambiguity in contexts like new expressions
        /// where parentheses could be interpreted differently.
        const FORBID_CALL = 1 << 1;
        /// Enable TypeScript-specific code generation
        ///
        /// When set, TypeScript syntax features are enabled in the output.
        const TYPESCRIPT  = 1 << 2;
        /// The type subtree is at the leading print position of a type alias RHS (`type N = …`).
        ///
        /// Propagates through union/intersection first members, array element types, and indexed
        /// access object types so bare `intrinsic` type references are parenthesized only where
        /// re-parsing would otherwise yield `TSIntrinsicKeyword` instead of `TSTypeReference`.
        const TS_TYPE_ALIAS_RHS_LEADING = 1 << 3;
    }
}

impl Context {
    /// Check if the `in` operator is forbidden in the current context
    #[inline]
    pub fn forbid_in(self) -> bool {
        self.contains(Self::FORBID_IN)
    }

    /// Check if call expressions are forbidden in the current context
    #[inline]
    pub fn forbid_call(self) -> bool {
        self.contains(Self::FORBID_CALL)
    }

    /// Create a new context with TypeScript support enabled
    #[inline]
    #[must_use]
    pub fn with_typescript(mut self) -> Self {
        self |= Self::TYPESCRIPT;
        self
    }

    /// Clears [`Self::TS_TYPE_ALIAS_RHS_LEADING`].
    #[inline]
    #[must_use]
    pub fn without_ts_type_alias_rhs_leading(self) -> Self {
        self - Self::TS_TYPE_ALIAS_RHS_LEADING
    }

    /// Conditionally set or unset the `FORBID_IN` flag
    #[inline]
    #[must_use]
    pub fn and_forbid_in(self, include: bool) -> Self {
        self.and(Self::FORBID_IN, include)
    }

    /// Conditionally set or unset the `FORBID_CALL` flag
    #[inline]
    #[must_use]
    pub fn and_forbid_call(self, include: bool) -> Self {
        self.and(Self::FORBID_CALL, include)
    }

    /// Helper method to conditionally set or unset a flag
    #[inline]
    fn and(self, flag: Self, set: bool) -> Self {
        if set { self | flag } else { self - flag }
    }
}
