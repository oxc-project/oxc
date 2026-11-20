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

use oxc_formatter_core::Placeholder;

/// css-in-js `${}` interpolation marker.
///
/// The parent (JS) formatter substitutes each interpolation with a
/// backtick-delimited `` `oxfmt_css_N` `` marker before dispatching to [`format_to_ir`].
/// Backtick is invalid SCSS (the css-in-js variant),
/// so the marker is unmistakably out-of-band, not a real `@var`/`$var` or at-rule.
/// (Like Prettier, which uses `@prettier-placeholder-N-id`)
/// `oxc-css-parser` tokenizes it via the fork's `template_placeholder` option,
/// consuming the leading backtick as the sigil (so `format.rs` passes the affix without it).
///
/// The marker is this crate's input protocol, the producer's copy in `oxc_formatter`'s `embed/css.rs`
/// (which doesn't depend on this crate): a drift fails every css-in-js snapshot, the marker no token then.
pub(crate) const TEMPLATE_PLACEHOLDER: Placeholder<'static> =
    Placeholder { prefix: "`oxfmt_css_", suffix: "`" };

pub use crate::{
    comments::CssComment,
    context::CssFormatContext,
    format::{
        CssEmbeddedIn, CssRoot, ParsedCss, format, format_to_ir, format_with_session,
        parse_for_format,
    },
    options::{CssFormatOptions, CssVariant, SingleQuote, TrailingCommas},
};
