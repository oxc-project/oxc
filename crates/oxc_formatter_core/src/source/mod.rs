//! Source-side access mechanics:
//! data structures over the input source that make no output decision by themselves.
//!
//! - [`SourceText`]: offset-keyed byte/slice access (WHERE)
//! - [`SpanCursor`]: span-ordered drain over a sorted item slice (WHO is next)
//! - [`rebase_labels`] / [`restore_crlf_labels`]: diagnostic labels moved from a trimmed / newline-normalized copy back onto the source
//!
//! Admission: the structure decides nothing about the output.
//! Unlike `spec/`, which encodes shared formatting behavior as pure functions.
//! Language differences arrive as data (offsets, the item type),
//! never as parameters encoding grammar or policy.
//! Interpretation (what counts as a comment, what a gap means, where things are placed) stays consumer-owned;
//! consumers compose these with `spec/` helpers at their print sites.

mod cursor;
mod labels;
mod text;

pub use cursor::SpanCursor;
pub use labels::{rebase_labels, restore_crlf_labels};
pub use text::SourceText;
