//! Building blocks shared by the disambiguation questions.
//!
//! - [`tokens`]: the view of the lex that every question reads.
//! - [`bits`]: bit scans over the token bitmaps.
//! - [`walk`]: moving backwards through the token stream: stepping to the previous token,
//!   inspecting it, and jumping over bracketed groups.

pub(super) mod bits;
mod closers;
mod tokens;
mod walk;

pub(crate) use closers::Closers;
pub(crate) use tokens::{Peek, Prev, Tokens};
pub(crate) use walk::{Brackets, prev_sig};

pub(super) use walk::{kind_at, word_is_any, word_len};
