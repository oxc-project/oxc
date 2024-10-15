// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use oxc_ast_macros::ast;
#[cfg(feature = "serialize")]
use ::{serde::Serialize, tsify::Tsify};

/// Newtype for working with text ranges
///
/// See the [`text-size`](https://docs.rs/text-size) crate for details.
/// Utility methods can be copied from the `text-size` crate if they are needed.
/// NOTE: `u32` is sufficient for "all" reasonable programs. Larger than u32 is a 4GB JS file.
///
/// ## Hashing
/// [`Span`] has a normal implementation of [`Hash`]. If you want to compare two
/// AST nodes without considering their locations (e.g. to see if they have the
/// same content), use [`ContentHash`](crate::hash::ContentHash) instead.
#[ast]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[non_exhaustive] // Disallow struct expression constructor `Span {}`
pub struct Span {
    /// The zero-based start offset of the span
    pub start: u32,
    /// The zero-based end offset of the span. This may be equal to [`start`](Span::start) if
    /// the span is empty, but should not be less than it.
    pub end: u32,
}
