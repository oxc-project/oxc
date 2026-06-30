//! CSS/SCSS/Less formatter built on top of `oxc_formatter_core`.
//!
//! Parses with [oxc-css-parser](https://docs.rs/oxc-css-parser) (raffia fork) and prints Prettier-compatible output.
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

/// css-in-js `${}` interpolation marker, opening affix.
///
/// The parent (JS) formatter substitutes each interpolation with a
/// backtick-delimited `` `PLACEHOLDER-N` `` marker before dispatching to [`format_to_ir`].
/// Backtick is invalid SCSS (the css-in-js variant),
/// so the marker is unmistakably out-of-band, not a real `@var`/`$var` or at-rule.
/// (Like Prettier, which uses `@prettier-placeholder-N-id`)
/// `oxc-css-parser` tokenizes it via the fork's `template_placeholder` option,
/// consuming the leading backtick as the sigil (so `format.rs` passes the affix without it).
///
/// The producer-side constant lives in `oxc_formatter`'s `embed/css.rs`
/// (which doesn't depend on this crate); orchestrator-side consumers (oxfmt) should use these.
pub const TEMPLATE_PLACEHOLDER_PREFIX: &str = "`PLACEHOLDER-";
/// Closing affix of the css-in-js marker (the closing backtick).
pub const TEMPLATE_PLACEHOLDER_SUFFIX: &str = "`";

pub use crate::{
    context::CssFormatContext,
    format::{TailwindSorter, format, format_to_ir},
    options::{CssFormatOptions, CssVariant, SingleQuote, TrailingCommas},
};
