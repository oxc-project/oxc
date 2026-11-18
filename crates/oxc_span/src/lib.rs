//! Source positions and related helper functions.
//!
//! <https://doc.rust-lang.org/beta/nightly-rustc/rustc_span>

mod cmp;
mod edit_distance;
mod labeled_span;
#[cfg(feature = "serialize")]
mod serialize;
mod source_type;
mod span;

pub use cmp::ContentEq;
pub use edit_distance::{best_match, min_edit_distance};
pub use labeled_span::LabeledSpan;
pub use source_type::{
    FileExtension, Language, LanguageVariant, ModuleKind, SourceType, UnknownExtension,
    VALID_EXTENSIONS,
};
pub use span::{GetSpan, GetSpanMut, SPAN, Span};

// Only here to make it available in `generated/assert_layouts` module
#[cfg(debug_assertions)]
use span::I32Dummy;

mod generated {
    #[cfg(debug_assertions)]
    mod assert_layouts;
    mod derive_dummy;
    #[cfg(feature = "serialize")]
    mod derive_estree;
}
