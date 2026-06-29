//! CSS/SCSS/Less formatter built on top of `oxc_formatter_core`.
//!
//! Parses with [raffia](https://docs.rs/raffia) and prints Prettier-compatible output.
//!
//! ```ignore
//! use oxc_allocator::Allocator;
//! use oxc_formatter_css::{CssFormatOptions, format};
//!
//! let allocator = Allocator::new();
//! let formatted =
//!     format(&allocator, "a { color: red }", CssFormatOptions::default(), None).unwrap();
//! let out = formatted.print().unwrap().into_code();
//! assert_eq!(out, "a {\n  color: red;\n}\n");
//! ```

mod comments;
mod context;
mod format;
mod options;
mod print;

/// css-in-js `${}` interpolation marker prefix.
///
/// The parent (JS) formatter substitutes each interpolation with
/// `@prettier-placeholder-N-id` before dispatching to [`format_to_ir`].
/// This is Prettier's wire format — its embed (`replacePlaceholders`)
/// expects exactly this shape, so the Prettier fallback path relies on it
/// staying in sync. The producer-side constant lives in `oxc_formatter`'s
/// `embed/css.rs` (which doesn't depend on this crate); orchestrator-side
/// consumers (oxfmt) should use these.
pub const TEMPLATE_PLACEHOLDER_PREFIX: &str = "@prettier-placeholder-";
/// See [`TEMPLATE_PLACEHOLDER_PREFIX`].
pub const TEMPLATE_PLACEHOLDER_SUFFIX: &str = "-id";

pub use crate::{
    context::CssFormatContext,
    format::{TailwindSorter, format, format_to_ir},
    options::{CssFormatOptions, CssVariant, SingleQuote, TrailingCommas},
};
