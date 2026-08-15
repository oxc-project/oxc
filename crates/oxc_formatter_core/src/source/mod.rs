//! Source-side access mechanics:
//! data structures over the input source that make no output decision by themselves.
//!
//! - [`SourceText`]: offset-keyed byte/slice access (WHERE)
//! - [`SpanCursor`]: span-ordered drain over a sorted item slice (WHO is next)
//!
//! Admission: the structure decides nothing about the output.
//! Unlike `spec/`, which encodes shared formatting behavior as pure functions.
//! Language differences arrive as data (offsets, the item type),
//! never as parameters encoding grammar or policy.
//! Interpretation (what counts as a comment, what a gap means, where things are placed) stays consumer-owned;
//! consumers compose these with `spec/` helpers at their print sites.

mod cursor;
mod text;

pub use cursor::SpanCursor;
pub use text::SourceText;
