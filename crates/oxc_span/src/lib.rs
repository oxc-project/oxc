//! Source positions and related helper functions.
//!
//! <https://doc.rust-lang.org/beta/nightly-rustc/rustc_span>

mod cmp;
mod edit_distance;
#[cfg(feature = "serialize")]
mod serialize;
mod source_type;
mod span;

pub use cmp::ContentEq;
pub use edit_distance::{best_match, min_edit_distance};
use oxc_str::{CompactStr, Ident, Str};
pub use source_type::{
    FileExtension, Language, LanguageVariant, ModuleKind, SourceType, UnknownExtension,
    VALID_EXTENSIONS,
};
pub use span::{GetSpan, GetSpanMut, SPAN, Span};

// Only here to make it available in `generated/assert_layouts` module
#[cfg(debug_assertions)]
use span::I32Dummy;

mod generated {
    #[cfg(debug_assertions)]
    mod assert_layouts;
    mod derive_dummy;
    #[cfg(feature = "serialize")]
    mod derive_estree;
}

#[doc(hidden)]
pub mod __internal {
    // Used by `format_compact_str!` macro defined in `oxc_str`
    pub use compact_str::format_compact;
    // Used by `format_str!` and `format_ident!` macros defined in `oxc_str`
    pub use oxc_allocator::StringBuilder as ArenaStringBuilder;
}

// Additional trait implementations for types re-exported from oxc_str
use std::ops::Index;

impl ContentEq for Str<'_> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for Ident<'_> {
    #[inline]
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Index<Span> for CompactStr {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self.as_str()[index.start as usize..index.end as usize]
    }
}
