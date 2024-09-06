//! Source positions and related helper functions.
//!
//! <https://doc.rust-lang.org/beta/nightly-rustc/rustc_span>

mod atom;
mod source_type;
mod span;

pub mod cmp;
pub mod hash;

pub use crate::{
    atom::{Atom, CompactStr, MAX_INLINE_LEN as ATOM_MAX_INLINE_LEN},
    source_type::{
        Language, LanguageVariant, ModuleKind, SourceType, UnknownExtension, VALID_EXTENSIONS,
    },
    span::{GetSpan, GetSpanMut, Span, SPAN},
};
