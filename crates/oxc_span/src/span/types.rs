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
/// [`Span`]'s implementation of [`Hash`] is a no-op so that AST nodes can be
/// compared by hash. This makes them unsuitable for use as keys in a hash map.
///
/// ```
/// use std::hash::{Hash, Hasher, DefaultHasher};
/// use oxc_span::Span;
///
/// let mut first = DefaultHasher::new();
/// let mut second = DefaultHasher::new();
///
/// Span::new(0, 5).hash(&mut first);
/// Span::new(10, 20).hash(&mut second);
///
/// assert_eq!(first.finish(), second.finish());
/// ```
#[ast]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[non_exhaustive] // Disallow struct expression constructor `Span {}`
pub struct Span {
    pub start: u32,
    pub end: u32,
}
