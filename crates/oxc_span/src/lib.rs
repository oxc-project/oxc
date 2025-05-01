//! Source positions and related helper functions.
//!
//! <https://doc.rust-lang.org/beta/nightly-rustc/rustc_span>

#![warn(missing_docs)]

mod atom;
mod cmp;
mod compact_str;
mod source_type;
mod span;

pub use crate::{
    atom::Atom,
    cmp::ContentEq,
    compact_str::{CompactStr, MAX_INLINE_LEN as ATOM_MAX_INLINE_LEN},
    source_type::{
        Language, LanguageVariant, ModuleKind, SourceType, UnknownExtension, VALID_EXTENSIONS,
    },
    span::{GetSpan, GetSpanMut, SPAN, Span},
};

mod generated {
    #[cfg(debug_assertions)]
    pub mod assert_layouts;
    mod derive_dummy;
    #[cfg(feature = "serialize")]
    pub mod derive_estree;
}

#[doc(hidden)]
pub mod __internal {
    // Used by `format_compact_str!` macro defined in `compact_str.rs`
    pub use compact_str::format_compact;
    // Used by `format_atom!` macro defined in `atom.rs`
    pub use oxc_allocator::String as ArenaString;
}
