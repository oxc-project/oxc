//! Source positions and related helper functions.
//!
//! <https://doc.rust-lang.org/beta/nightly-rustc/rustc_span>

#![warn(missing_docs)]

mod cmp;
mod source_type;
mod span;

pub use cmp::ContentEq;
pub use oxc_str::{
    Atom, CompactStr, Ident, MAX_INLINE_LEN as ATOM_MAX_INLINE_LEN, format_atom,
    format_compact_str, format_ident,
};
pub use source_type::{
    FileExtension, Language, LanguageVariant, ModuleKind, SourceType, UnknownExtension,
    VALID_EXTENSIONS,
};
pub use span::{GetSpan, GetSpanMut, SPAN, Span};

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
    // Used by `format_atom!` and `format_ident!` macros defined in `oxc_str`
    pub use oxc_allocator::StringBuilder as ArenaStringBuilder;
}

// Additional trait implementations for types re-exported from oxc_str
use std::ops::Index;

impl ContentEq for Atom<'_> {
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
