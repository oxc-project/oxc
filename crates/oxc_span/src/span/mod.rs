use std::ops::{Index, IndexMut, Range};

use miette::{LabeledSpan, SourceOffset, SourceSpan};

mod types;
use oxc_allocator::{Allocator, CloneIn};
pub use types::Span;

/// An Empty span useful for creating AST nodes.
pub const SPAN: Span = Span::new(0, 0);

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

    /// Create a new empty [`Span`] that starts and ends at an offset position.
    ///
    /// # Examples
    /// ```
    /// use oxc_span::Span;
    ///
    /// let fifth = Span::empty(5);
    /// assert!(fifth.is_empty());
    /// assert_eq!(fifth, Span::sized(5, 0));
    /// assert_eq!(fifth, Span::new(5, 5));
    /// ```
    pub fn empty(at: u32) -> Self {
        Self { start: at, end: at }
    }

    /// Create a new [`Span`] starting at `start` and covering `size` bytes.
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// let span = Span::sized(2, 4);
    /// assert_eq!(span.size(), 4);
    /// assert_eq!(span, Span::new(2, 6));
    /// ```
    pub const fn sized(start: u32, size: u32) -> Self {
        Self::new(start, start + size)
    }

    /// Get the number of bytes covered by the [`Span`].
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// assert_eq!(Span::new(1, 1).size(), 0);
    /// assert_eq!(Span::new(0, 5).size(), 5);
    /// assert_eq!(Span::new(5, 10).size(), 5);
    /// ```
    pub const fn size(&self) -> u32 {
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
    pub const fn is_empty(&self) -> bool {
        debug_assert!(self.start <= self.end);
        self.start == self.end
    }

    /// Returns `true` if `self` is not a real span.
    /// i.e. `SPAN` which is used for generated nodes which are not in source code.
    ///
    /// # Example
    /// ```
    /// use oxc_span::{Span, SPAN};
    ///
    /// assert!(SPAN.is_unspanned());
    /// assert!(!Span::new(0, 5).is_unspanned());
    /// assert!(!Span::new(5, 5).is_unspanned());
    /// ```
    pub const fn is_unspanned(&self) -> bool {
        self.start == SPAN.start && self.end == SPAN.end
    }

    /// Check if this [`Span`] contains another [`Span`].
    ///
    /// [`Span`]s that start & end at the same position as this [`Span`] are
    /// considered contained.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oxc_span::Span;
    /// let span = Span::new(5, 10);
    ///
    /// assert!(span.contains_inclusive(span)); // always true for itself
    /// assert!(span.contains_inclusive(Span::new(5, 5)));
    /// assert!(span.contains_inclusive(Span::new(6, 10)));
    /// assert!(span.contains_inclusive(Span::empty(5)));
    ///
    /// assert!(!span.contains_inclusive(Span::new(4, 10)));
    /// assert!(!span.contains_inclusive(Span::empty(0)));
    /// ```
    #[inline]
    pub const fn contains_inclusive(self, span: Span) -> bool {
        self.start <= span.start && span.end <= self.end
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

    /// Create a [`Span`] that is grown by `offset` on either side.
    ///
    /// This is equivalent to `span.expand_left(offset).expand_right(offset)`.
    /// See [`expand_left`] and [`expand_right`] for more info.
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    ///
    /// let span = Span::new(3, 5);
    /// assert_eq!(span.expand(1), Span::new(2, 6));
    /// // start and end cannot be expanded past `0` and `u32::MAX`, respectively
    /// assert_eq!(span.expand(5), Span::new(0, 10));
    /// ```
    ///
    /// [`expand_left`]: Span::expand_left
    /// [`expand_right`]: Span::expand_right
    #[must_use]
    pub fn expand(self, offset: u32) -> Self {
        Self::new(self.start.saturating_sub(offset), self.end.saturating_add(offset))
    }

    /// Create a [`Span`] that has its start and end positions shrunk by
    /// `offset` amount.
    ///
    /// It is a logical error to shrink the start of the [`Span`] past its end
    /// position. This will panic in debug builds.
    ///
    /// This is equivalent to `span.shrink_left(offset).shrink_right(offset)`.
    /// See [`shrink_left`] and [`shrink_right`] for more info.
    ///
    /// # Example
    /// ```
    /// use oxc_span::Span;
    /// let span = Span::new(5, 10);
    /// assert_eq!(span.shrink(2), Span::new(7, 8));
    /// ```
    ///
    /// [`shrink_left`]: Span::shrink_left
    /// [`shrink_right`]: Span::shrink_right
    #[must_use]
    pub fn shrink(self, offset: u32) -> Self {
        let start = self.start.saturating_add(offset);
        let end = self.end.saturating_sub(offset);
        debug_assert!(start <= end, "Cannot shrink span past zero length");
        Self::new(start, end)
    }

    /// Create a [`Span`] that has its start position moved to the left by
    /// `offset` bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(5, 10);
    /// assert_eq!(a.expand_left(5), Span::new(0, 10));
    /// ```
    ///
    /// ## Bounds
    ///
    /// The leftmost bound of the span is clamped to 0. It is safe to call this
    /// method with a value larger than the start position.
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(0, 5);
    /// assert_eq!(a.expand_left(5), Span::new(0, 5));
    /// ```
    #[must_use]
    pub const fn expand_left(self, offset: u32) -> Self {
        Self::new(self.start.saturating_sub(offset), self.end)
    }

    /// Create a [`Span`] that has its start position moved to the right by
    /// `offset` bytes.
    ///
    /// It is a logical error to shrink the start of the [`Span`] past its end
    /// position.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(5, 10);
    /// let shrunk = a.shrink_left(5);
    /// assert_eq!(shrunk, Span::new(10, 10));
    ///
    /// // Shrinking past the end of the span is a logical error that will panic
    /// // in debug builds.
    /// std::panic::catch_unwind(|| {
    ///    shrunk.shrink_left(5);
    /// });
    /// ```
    ///
    #[must_use]
    pub const fn shrink_left(self, offset: u32) -> Self {
        let start = self.start.saturating_add(offset);
        debug_assert!(start <= self.end);
        Self::new(self.start.saturating_add(offset), self.end)
    }

    /// Create a [`Span`] that has its end position moved to the right by
    /// `offset` bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(5, 10);
    /// assert_eq!(a.expand_right(5), Span::new(5, 15));
    /// ```
    ///
    /// ## Bounds
    ///
    /// The rightmost bound of the span is clamped to `u32::MAX`. It is safe to
    /// call this method with a value larger than the end position.
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(0, u32::MAX);
    /// assert_eq!(a.expand_right(5), Span::new(0, u32::MAX));
    /// ```
    #[must_use]
    pub const fn expand_right(self, offset: u32) -> Self {
        Self::new(self.start, self.end.saturating_add(offset))
    }

    /// Create a [`Span`] that has its end position moved to the left by
    /// `offset` bytes.
    ///
    /// It is a logical error to shrink the end of the [`Span`] past its start
    /// position.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(5, 10);
    /// let shrunk = a.shrink_right(5);
    /// assert_eq!(shrunk, Span::new(5, 5));
    ///
    /// // Shrinking past the start of the span is a logical error that will panic
    /// // in debug builds.
    /// std::panic::catch_unwind(|| {
    ///    shrunk.shrink_right(5);
    /// });
    /// ```
    #[must_use]
    pub const fn shrink_right(self, offset: u32) -> Self {
        let end = self.end.saturating_sub(offset);
        debug_assert!(self.start <= end);
        Self::new(self.start, end)
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

    /// Create a [`LabeledSpan`] covering this [`Span`] with the given label.
    ///
    /// Use [`Span::primary_label`] if this is the primary span for the diagnostic.
    #[must_use]
    pub fn label<S: Into<String>>(self, label: S) -> LabeledSpan {
        LabeledSpan::new_with_span(Some(label.into()), self)
    }

    /// Creates a primary [`LabeledSpan`] covering this [`Span`] with the given label.
    #[must_use]
    pub fn primary_label<S: Into<String>>(self, label: S) -> LabeledSpan {
        LabeledSpan::new_primary_with_span(Some(label.into()), self)
    }
}

impl Index<Span> for str {
    type Output = str;

    #[inline]
    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start as usize..index.end as usize]
    }
}

impl IndexMut<Span> for str {
    #[inline]
    fn index_mut(&mut self, index: Span) -> &mut Self::Output {
        &mut self[index.start as usize..index.end as usize]
    }
}

impl From<Range<u32>> for Span {
    #[inline]
    fn from(range: Range<u32>) -> Self {
        Self::new(range.start, range.end)
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
    /// Get the [`Span`] for an AST node
    fn span(&self) -> Span;
}

/// Get mutable ref to span for an AST node
pub trait GetSpanMut {
    /// Get a mutable reference to an AST node's [`Span`].
    fn span_mut(&mut self) -> &mut Span;
}

impl GetSpan for Span {
    #[inline]
    fn span(&self) -> Span {
        *self
    }
}

impl GetSpanMut for Span {
    #[inline]
    fn span_mut(&mut self) -> &mut Span {
        self
    }
}

impl<'a> CloneIn<'a> for Span {
    type Cloned = Self;

    #[inline]
    fn clone_in(&self, _: &'a Allocator) -> Self {
        *self
    }
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
        Span::new(0, 5).hash(&mut second);
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

    #[test]
    fn test_contains() {
        let span = Span::new(5, 10);

        assert!(span.contains_inclusive(span));
        assert!(span.contains_inclusive(Span::new(5, 5)));
        assert!(span.contains_inclusive(Span::new(10, 10)));
        assert!(span.contains_inclusive(Span::new(6, 9)));

        assert!(!span.contains_inclusive(Span::new(0, 0)));
        assert!(!span.contains_inclusive(Span::new(4, 10)));
        assert!(!span.contains_inclusive(Span::new(5, 11)));
        assert!(!span.contains_inclusive(Span::new(4, 11)));
    }

    #[test]
    fn test_expand() {
        let span = Span::new(3, 5);
        assert_eq!(span.expand(0), Span::new(3, 5));
        assert_eq!(span.expand(1), Span::new(2, 6));
        // start and end cannot be expanded past `0` and `u32::MAX`, respectively
        assert_eq!(span.expand(5), Span::new(0, 10));
    }

    #[test]
    fn test_shrink() {
        let span = Span::new(4, 8);
        assert_eq!(span.shrink(0), Span::new(4, 8));
        assert_eq!(span.shrink(1), Span::new(5, 7));
        // can be equal
        assert_eq!(span.shrink(2), Span::new(6, 6));
    }

    #[test]
    #[should_panic(expected = "Cannot shrink span past zero length")]
    fn test_shrink_past_start() {
        let span = Span::new(5, 10);
        let _ = span.shrink(5);
    }
}
