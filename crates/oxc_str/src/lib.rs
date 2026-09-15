//! String types for oxc.
//!
//! This crate provides [`Str`], [`Ident`], and [`CompactStr`] for UTF-8 strings, and
//! [`JSStr`], [`JSChar`], and [`JSStrBuilder`] for JavaScript strings, including lone surrogates.
//!
//! # Naming
//!
//! Types are named for what a value means in JavaScript, as with [`JSStr`]
//! and [`JSChar`]. Storage, encoding, and ownership stay out of public names:
//! WTF-8 is an implementation detail, and whether a name borrows or owns its
//! text is the type's concern rather than the caller's.

mod compact_str;
mod ident;
mod ident_hasher;
mod js_char;
mod js_str;
mod js_str_builder;
#[cfg(feature = "serialize")]
mod serialize;
mod str;

pub use compact_str::{CompactStr, MAX_INLINE_LEN};
pub use ident::{ArenaIdentHashMap, ArenaIdentHashSet, Ident, IdentHashMap, IdentHashSet};
pub use ident_hasher::{IdentBuildHasher, IdentHasher};
pub use js_char::JSChar;
pub use js_str::JSStr;
pub use js_str_builder::JSStrBuilder;
pub use str::{Str, Str as ArenaStr};

#[doc(hidden)]
pub mod __internal {
    // Used by `format_compact_str!` macro defined in `compact_str.rs`
    pub use compact_str::format_compact;
    // Used by `format_str!` and `format_ident!` macros
    pub use oxc_allocator::ArenaStringBuilder;
    // Used by `static_ident!` macro
    pub use crate::ident::new_const_ident;
}
