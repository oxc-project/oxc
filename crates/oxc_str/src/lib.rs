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
//!
//! # Consuming `JSStr`
//!
//! A consumer that needs `str` decides what a lone surrogate means for its
//! check, and three policies cover the cases in practice. Checks over names,
//! paths, and specifiers decline when [`JSStr::as_str`] returns `None`: a
//! fixed name set or a well-known path never contains a lone surrogate, so
//! declining changes nothing. Comparisons with fixed names use `JSStr`'s
//! `PartialEq<&str>` directly and need no conversion. Only a value matched
//! against user-configured patterns or shown to a person goes lossy:
//! [`JSStr::to_str_lossy`] for matching, and the `Debug` form for
//! diagnostics, which escapes the surrogate instead of replacing it.
//!
//! Declining is only safe where skipping the work is conservative, as for an
//! optional optimization or a lookup that cannot match. Code that decides
//! whether a program is valid, whether a rewrite is safe, or what code to emit
//! must handle the value instead of skipping it.

mod compact_str;
mod ident;
mod ident_hasher;
mod js_char;
mod js_str;
mod js_str_builder;
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
