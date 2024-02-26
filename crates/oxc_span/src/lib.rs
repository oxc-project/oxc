//! Source positions and related helper functions.
//!
//! <https://doc.rust-lang.org/beta/nightly-rustc/rustc_span>

mod atom;
mod source_type;
mod span;

pub use crate::{
    atom::Atom,
    source_type::{Language, LanguageVariant, ModuleKind, SourceType, VALID_EXTENSIONS},
    span::{GetSpan, Span, SPAN},
};
pub use compact_str::CompactString;
