//! String types for oxc.
//!
//! This crate provides [`Str`], [`Ident`], and [`CompactStr`] types for efficient string handling.

mod compact_str;
mod ident;
mod str;

pub use compact_str::{CompactStr, MAX_INLINE_LEN};
pub use ident::{ArenaIdentHashMap, Ident, IdentHashMap, IdentHashSet};
pub use str::Str;

#[doc(hidden)]
pub mod __internal {
    // Used by `format_compact_str!` macro defined in `compact_str.rs`
    pub use compact_str::format_compact;
    // Used by `format_str!` and `format_ident!` macros
    pub use oxc_allocator::StringBuilder as ArenaStringBuilder;
    // Used by `static_ident!` macro
    pub use crate::ident::new_const_ident;
}
