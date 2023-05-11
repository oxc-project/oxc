use std::hash::{Hash, Hasher};

use compact_str::CompactString;
use miette::{SourceOffset, SourceSpan};
#[cfg(feature = "serde")]
use serde::Serialize;

/// Type alias for [`CompactString`]
pub type Atom = CompactString;

/// Newtype for working with text ranges
///
/// See the [`text-size`](https://docs.rs/text-size) crate for details.
/// Utility methods can be copied from the `text-size` crate if they are needed.
/// NOTE: `u32` is sufficient for "all" reasonable programs. Larger than u32 is a 4GB JS file.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
        Self::new(SourceOffset::from(val.start as usize), SourceOffset::from(val.size() as usize))
    }
}

/// Get the span for an AST node
pub trait GetSpan {
    fn span(&self) -> Span;
}
