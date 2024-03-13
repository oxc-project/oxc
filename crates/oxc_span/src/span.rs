use std::hash::{Hash, Hasher};

use miette::{SourceOffset, SourceSpan};
use oxc_macros::SerAttrs;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "wasm")]
use tsify::Tsify;

/// An Empty span useful for creating AST nodes.
pub const SPAN: Span = Span::new(0, 0);

/// Newtype for working with text ranges
///
/// See the [`text-size`](https://docs.rs/text-size) crate for details.
/// Utility methods can be copied from the `text-size` crate if they are needed.
/// NOTE: `u32` is sufficient for "all" reasonable programs. Larger than u32 is a 4GB JS file.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[non_exhaustive] // disallow struct expression constructor `Span {}`
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn size(&self) -> u32 {
        debug_assert!(self.start <= self.end);
        self.end - self.start
    }

    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }

    pub fn source_text<'a>(&self, source_text: &'a str) -> &'a str {
        &source_text[self.start as usize..self.end as usize]
    }
}

impl Hash for Span {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // hash to nothing so all ast spans can be comparible with hash
    }
}

impl From<Span> for SourceSpan {
    fn from(val: Span) -> Self {
        Self::new(SourceOffset::from(val.start as usize), val.size() as usize)
    }
}

/// Get the span for an AST node
pub trait GetSpan {
    fn span(&self) -> Span;
}
