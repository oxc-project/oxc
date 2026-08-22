//! WTF-8 encoding for oxc — allows storing lone surrogates.
//! Ported from SWC's `hstr::wtf8` (MIT, originally rust-wtf8 / Rust std).
//!
//! WTF-8 is a superset of UTF-8 that also encodes lone surrogates (U+D800..U+DFFF)
//! as 3-byte sequences (ED A0 80 .. ED BF BF). This lets JavaScript strings
//! containing lone surrogates be stored without loss.

#![allow(clippy::many_single_char_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::inline_always)]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod wtf8;

pub use wtf8::{CodePoint, Wtf8, Wtf8Buf};
