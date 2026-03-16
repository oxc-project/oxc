//! String types for oxc.
//!
//! This crate provides [`Atom`], [`Ident`], and [`CompactStr`] types for efficient string handling.

mod atom;
mod compact_str;
pub mod ident;

pub use atom::Atom;
pub use compact_str::{CompactStr, MAX_INLINE_LEN};
pub use ident::{ArenaIdentHashMap, Ident, IdentHashMap, IdentHashSet};

#[doc(hidden)]
pub mod __internal {
    // Used by `format_compact_str!` macro defined in `compact_str.rs`
    pub use compact_str::format_compact;
    // Used by `format_atom!` and `format_ident!` macros
    pub use oxc_allocator::StringBuilder as ArenaStringBuilder;
}
