/// Describes what kind of automatic fix is available for a diagnostic.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FixAvailability {
    /// No fix is available for this diagnostic.
    #[default]
    None,
    /// A safe fix is available. Safe fixes are guaranteed to not change
    /// the meaning of the code.
    SafeFix,
    /// A dangerous fix is available. Dangerous fixes may change the meaning
    /// of the code or may not always be correct.
    DangerousFix,
    /// A suggestion is available. Suggestions are recommendations that may
    /// not be automatically applicable.
    Suggestion,
    /// A dangerous suggestion is available.
    DangerousSuggestion,
}

impl FixAvailability {
    /// Returns `true` if a fix is available (any kind).
    #[inline]
    pub const fn is_some(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns `true` if no fix is available.
    #[inline]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns `true` if this is a safe fix (not dangerous).
    #[inline]
    pub const fn is_safe_fix(self) -> bool {
        matches!(self, Self::SafeFix)
    }

    /// Returns `true` if this is a dangerous fix.
    #[inline]
    pub const fn is_dangerous_fix(self) -> bool {
        matches!(self, Self::DangerousFix)
    }

    /// Returns `true` if this is a suggestion (safe or dangerous).
    #[inline]
    pub const fn is_suggestion(self) -> bool {
        matches!(self, Self::Suggestion | Self::DangerousSuggestion)
    }
}
