//! String types for oxc.
//!
//! This crate provides [`Str`], [`Ident`], and [`CompactStr`] types for efficient string handling.

mod compact_str;
mod ident;
mod ident_hasher;
mod str;
mod wtf8;
mod wtf8_str;

pub use compact_str::{CompactStr, MAX_INLINE_LEN};
pub use ident::{ArenaIdentHashMap, Ident, IdentHashMap, IdentHashSet};
pub use ident_hasher::{IdentBuildHasher, IdentHasher};
pub use str::{Str, Str as ArenaStr};
pub use wtf8::{
    CodePoint, Wtf8, Wtf8Buf, Wtf8Chunk, Wtf8Chunks, Wtf8CodePoints, Wtf8CodeUnits, Wtf8Error,
};
pub use wtf8_str::Wtf8Str;

#[doc(hidden)]
pub mod __internal {
    // Used by `format_compact_str!` macro defined in `compact_str.rs`
    pub use compact_str::format_compact;
    // Used by `format_str!` and `format_ident!` macros
    pub use oxc_allocator::ArenaStringBuilder;
    // Used by `static_ident!` macro
    pub use crate::ident::new_const_ident;
}
