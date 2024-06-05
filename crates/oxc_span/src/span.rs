// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::hash::{Hash, Hasher};

use miette::{LabeledSpan, SourceOffset, SourceSpan};

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "serialize")]
use tsify::Tsify;

/// An Empty span useful for creating AST nodes.
pub const SPAN: Span = Span::new(0, 0);

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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(Serialize, Tsify))]
#[non_exhaustive] // disallow struct expression constructor `Span {}`
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// Create a new [`Span`] from a start and end position.
    ///
    /// # Invariants
    /// The `start` position must be less than or equal to `end`. Note that this
    /// invariant is only checked in debug builds to avoid a performance
    /// penalty.
    ///
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Get the number of characters covered by the [`Span`].
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// assert_eq!(Span::new(1, 1).size(), 0);
    /// assert_eq!(Span::new(0, 5).size(), 5);
    /// assert_eq!(Span::new(5, 10).size(), 5);
    /// ```
    pub fn size(&self) -> u32 {
        debug_assert!(self.start <= self.end);
        self.end - self.start
    }

    /// Returns `true` if `self` covers a range of zero length.
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// assert!(Span::new(0, 0).is_empty());
    /// assert!(Span::new(5, 5).is_empty());
    /// assert!(!Span::new(0, 5).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.start <= self.end);
        self.start == self.end
    }

    /// Create a [`Span`] covering the maximum range of two [`Span`]s.
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// let span1 = Span::new(0, 5);
    /// let span2 = Span::new(3, 8);
    /// let merged_span = span1.merge(&span2);
    /// assert_eq!(merged_span, Span::new(0, 8));
    /// ```
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self::new(self.start.min(other.start), self.end.max(other.end))
    }

    /// Get a snippet of text from a source string that the [`Span`] covers.
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// let source = "function add (a, b) { return a + b; }";
    /// let name_span = Span::new(9, 12);
    /// let name = name_span.source_text(source);
    /// assert_eq!(name_span.size(), name.len() as u32);
    /// ```
    pub fn source_text<'a>(&self, source_text: &'a str) -> &'a str {
        &source_text[self.start as usize..self.end as usize]
    }
}

impl Hash for Span {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        // hash to nothing so all ast spans can be comparable with hash
    }
}

impl From<Span> for SourceSpan {
    fn from(val: Span) -> Self {
        Self::new(SourceOffset::from(val.start as usize), val.size() as usize)
    }
}

impl From<Span> for LabeledSpan {
    fn from(val: Span) -> Self {
        LabeledSpan::underline(val)
    }
}

/// Get the span for an AST node
pub trait GetSpan {
    fn span(&self) -> Span;
}

#[cfg(test)]
mod test {
    use super::Span;

    #[test]
    fn test_hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut first = DefaultHasher::new();
        let mut second = DefaultHasher::new();
        Span::new(0, 5).hash(&mut first);
        Span::new(10, 20).hash(&mut second);
        assert_eq!(first.finish(), second.finish());
    }
    #[test]
    fn test_eq() {
        assert_eq!(Span::new(0, 0), Span::new(0, 0));
        assert_eq!(Span::new(0, 1), Span::new(0, 1));
        assert_ne!(Span::new(0, 0), Span::new(0, 1));
    }

    #[test]
    fn test_ordering_less() {
        assert!(Span::new(0, 0) < Span::new(0, 1));
        assert!(Span::new(0, 3) < Span::new(2, 5));
    }

    #[test]
    fn test_ordering_greater() {
        assert!(Span::new(0, 1) > Span::new(0, 0));
        assert!(Span::new(2, 5) > Span::new(0, 3));
    }
}
