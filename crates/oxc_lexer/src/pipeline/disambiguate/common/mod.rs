//! Building blocks shared by the disambiguation questions.
//!
//! - [`tokens`]: the view of the lex that every question reads.
//! - [`bits`]: bit scans over the token bitmaps.
//! - [`text`]: raw source text on the forward scans.
//! - [`walk`]: moving backwards through the token stream: stepping to the previous token,
//!   inspecting it, and jumping over bracketed groups.

pub(super) mod bits;
pub(super) mod text;
mod tokens;
mod walk;

pub(crate) use tokens::{Prev, Tokens};
pub(crate) use walk::{Brackets, prev_sig};

pub(super) use walk::{kind_at, lt_in_range, word_is_any, word_len};
