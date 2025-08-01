use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    ops::{Index, IndexMut, Range},
};

use miette::{LabeledSpan, SourceOffset, SourceSpan};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer, ser::SerializeMap};

use oxc_allocator::{Allocator, CloneIn, Dummy};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;

#[cfg(feature = "serialize")]
use oxc_estree::ESTreeSpan;

/// An empty span.
///
/// Should be used for newly created new AST nodes.
pub const SPAN: Span = Span::new(0, 0);

/// A range in text, represented by a zero-indexed start and end offset.
///
/// It is a logical error for `end` to be less than `start`.
///
/// ```
/// # use oxc_span::Span;
/// let text = "foo bar baz";
/// let span = Span::new(4, 7);
/// assert_eq!(&text[span], "bar");
/// ```
///
/// Spans use `u32` for offsets, meaning only files up to 4GB are supported.
/// This is sufficient for "all" reasonable programs. This tradeof cuts the size
/// of `Span` in half, offering a sizeable performance improvement and memory
/// footprint reduction.
///
/// ## Creating Spans
/// Span offers several constructors, each of which is more or less convenient
/// depending on the context. In general, [`Span::new`] is sufficient for most
/// cases. If you want to create a span starting at some point of a certain
/// length, you can use [`Span::sized`].
///
/// ```
/// # use oxc_span::Span;
/// let a = Span::new(5, 10);  // Start and end offsets
/// let b = Span::sized(5, 5); // Start offset and size
/// assert_eq!(a, b);
/// ```
///
/// ## Re-Sizing Spans
/// Span offsets can be mutated directly, but it is often more convenient to use
/// one of the [`expand`] or [`shrink`] methods. Each of these create a new span
/// without modifying the original.
///
/// ```
/// # use oxc_span::Span;
/// let s = Span::new(5, 10);
/// assert_eq!(s.shrink(2), Span::new(7, 8));
/// assert_eq!(s.shrink(2), s.shrink_left(2).shrink_right(2));
///
/// assert_eq!(s.expand(5), Span::new(0, 15));
/// assert_eq!(s.expand(5), s.expand_left(5).expand_right(5));
/// ```
///
/// ## Comparison
/// [`Span`] has a normal implementation of [`PartialEq`]. If you want to compare two
/// AST nodes without considering their locations (e.g. to see if they have the
/// same content), use [`ContentEq`] instead.
///
/// ## Implementation Notes
/// See the [`text-size`](https://docs.rs/text-size) crate for details.
/// Utility methods can be copied from the `text-size` crate if they are needed.
///
/// [`expand`]: Span::expand
/// [`shrink`]: Span::shrink
/// [`ContentEq`]: crate::ContentEq
#[ast(visit)]
#[derive(Default, Clone, Copy, Eq, PartialOrd, Ord)]
#[generate_derive(ESTree)]
#[builder(skip)]
#[content_eq(skip)]
#[estree(
    no_type,
    flatten,
    no_ts_def,
    add_ts_def = "interface Span { start: number; end: number; range?: [number, number]; }"
)]
pub struct Span {
    /// The zero-based start offset of the span
    pub start: u32,
    /// The zero-based end offset of the span. This may be equal to [`start`](Span::start) if
    /// the span is empty, but should not be less than it.
    pub end: u32,
    /// Align `Span` on 8 on 64-bit platforms
    #[estree(skip)]
    _align: PointerAlign,
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
        Self { start, end, _align: PointerAlign::new() }
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
        Self::new(at, at)
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
    pub const fn size(self) -> u32 {
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
    pub const fn is_empty(self) -> bool {
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
    #[inline]
    pub const fn is_unspanned(self) -> bool {
        self.const_eq(SPAN)
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
    /// let merged_span = span1.merge(span2);
    /// assert_eq!(merged_span, Span::new(0, 8));
    /// ```
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
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

    /// Create a [`Span`] that has its start and end position moved to the left by
    /// `offset` bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(5, 10);
    /// let moved = a.move_left(5);
    /// assert_eq!(moved, Span::new(0, 5));
    ///
    /// // Moving the start over 0 is logical error that will panic in debug builds.
    /// std::panic::catch_unwind(|| {
    ///    moved.move_left(5);
    /// });
    /// ```
    #[must_use]
    pub const fn move_left(self, offset: u32) -> Self {
        let start = self.start.saturating_sub(offset);
        #[cfg(debug_assertions)]
        if start == 0 {
            debug_assert!(self.start == offset, "Cannot move span past zero length");
        }
        Self::new(start, self.end.saturating_sub(offset))
    }

    /// Create a [`Span`] that has its start and end position moved to the right by
    /// `offset` bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_span::Span;
    ///
    /// let a = Span::new(5, 10);
    /// let moved = a.move_right(5);
    /// assert_eq!(moved, Span::new(10, 15));
    ///
    /// // Moving the end over `u32::MAX` is logical error that will panic in debug builds.
    /// std::panic::catch_unwind(|| {
    ///    moved.move_right(u32::MAX);
    /// });
    /// ```
    #[must_use]
    pub const fn move_right(self, offset: u32) -> Self {
        let end = self.end.saturating_add(offset);
        #[cfg(debug_assertions)]
        if end == u32::MAX {
            debug_assert!(
                u32::MAX.saturating_sub(offset) == self.end,
                "Cannot move span past `u32::MAX` length"
            );
        }
        Self::new(self.start.saturating_add(offset), end)
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
    pub fn source_text(self, source_text: &str) -> &str {
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

    /// Creates a primary [`LabeledSpan`] covering this [`Span`].
    #[must_use]
    pub fn primary(self) -> LabeledSpan {
        LabeledSpan::new_primary_with_span(None, self)
    }

    /// Convert [`Span`] to a single `u64`.
    ///
    /// On 64-bit platforms, `Span` is aligned on 8, so equivalent to a `u64`.
    /// Compiler boils this conversion down to a no-op on 64-bit platforms.
    /// <https://godbolt.org/z/9rcMoT1fc>
    ///
    /// Do not use this on 32-bit platforms as it's likely to be less efficient.
    ///
    /// Note: `#[ast]` macro adds `#[repr(C)]` to the struct, so field order is guaranteed.
    #[expect(clippy::inline_always)] // Because this is a no-op on 64-bit platforms.
    #[inline(always)]
    const fn as_u64(self) -> u64 {
        if cfg!(target_endian = "little") {
            ((self.end as u64) << 32) | (self.start as u64)
        } else {
            ((self.start as u64) << 32) | (self.end as u64)
        }
    }

    /// Compare two [`Span`]s.
    ///
    /// Same as `PartialEq::eq`, but a const function, and takes owned `Span`s.
    //
    // `#[inline(always)]` because want to make sure this is inlined into `PartialEq::eq`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    const fn const_eq(self, other: Self) -> bool {
        if cfg!(target_pointer_width = "64") {
            self.as_u64() == other.as_u64()
        } else {
            self.start == other.start && self.end == other.end
        }
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

// On 64-bit platforms, compare `Span`s as single `u64`s, which is faster when used with `&Span` refs.
// https://godbolt.org/z/sEf9MGvsr
impl PartialEq for Span {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.const_eq(*other)
    }
}

// Skip hashing `_align` field.
// On 64-bit platforms, hash `Span` as a single `u64`, which is faster with `FxHash`.
// https://godbolt.org/z/4fbvcsTxM
impl Hash for Span {
    #[inline] // We exclusively use `FxHasher`, which produces small output hashing `u64`s and `u32`s
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        if cfg!(target_pointer_width = "64") {
            self.as_u64().hash(hasher);
        } else {
            self.start.hash(hasher);
            self.end.hash(hasher);
        }
    }
}

// Skip `_align` field in `Debug` output
#[expect(clippy::missing_fields_in_debug)]
impl Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Span").field("start", &self.start).field("end", &self.end).finish()
    }
}

/// Get the span for an AST node.
pub trait GetSpan {
    /// Get the [`Span`] for an AST node.
    fn span(&self) -> Span;
}

/// Get mutable ref to span for an AST node.
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

impl<'a> Dummy<'a> for Span {
    /// Create a dummy [`Span`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        SPAN
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Span {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.start)?;
        map.serialize_entry("end", &self.end)?;
        map.end()
    }
}

#[cfg(feature = "serialize")]
impl ESTreeSpan for Span {
    #[expect(clippy::inline_always)] // `#[inline(always)]` because it's a no-op
    #[inline(always)]
    fn range(self) -> [u32; 2] {
        [self.start, self.end]
    }
}

/// Zero-sized type which has pointer alignment (8 on 64-bit, 4 on 32-bit).
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct PointerAlign([usize; 0]);

impl PointerAlign {
    #[inline]
    const fn new() -> Self {
        Self([])
    }
}

#[cfg(test)]
mod test {
    use super::Span;

    #[test]
    fn test_size() {
        let s = Span::sized(0, 5);
        assert_eq!(s.size(), 5);
        assert!(!s.is_empty());

        let s = Span::sized(5, 0);
        assert_eq!(s.size(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        fn hash<T: Hash>(value: T) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }

        let first_hash = hash(Span::new(1, 5));
        let second_hash = hash(Span::new(1, 5));
        assert_eq!(first_hash, second_hash);

        // On 64-bit platforms, check hash is equivalent to `u64`
        #[cfg(target_pointer_width = "64")]
        {
            let u64_equivalent: u64 =
                if cfg!(target_endian = "little") { 1 + (5 << 32) } else { (1 << 32) + 5 };
            let u64_hash = hash(u64_equivalent);
            assert_eq!(first_hash, u64_hash);
        }

        // On 32-bit platforms, check `_align` field does not alter hash
        #[cfg(not(target_pointer_width = "64"))]
        {
            #[derive(Hash)]
            #[repr(C)]
            struct PlainSpan {
                start: u32,
                end: u32,
            }

            let plain_hash = hash(PlainSpan { start: 1, end: 5 });
            assert_eq!(first_hash, plain_hash);
        }
    }

    #[test]
    fn test_eq() {
        assert_eq!(Span::new(0, 0), Span::new(0, 0));
        assert_eq!(Span::new(0, 1), Span::new(0, 1));
        assert_eq!(Span::new(1, 5), Span::new(1, 5));

        assert_ne!(Span::new(0, 0), Span::new(0, 1));
        assert_ne!(Span::new(1, 5), Span::new(0, 5));
        assert_ne!(Span::new(1, 5), Span::new(2, 5));
        assert_ne!(Span::new(1, 5), Span::new(1, 4));
        assert_ne!(Span::new(1, 5), Span::new(1, 6));
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

    #[test]
    fn test_move_left() {
        let span = Span::new(5, 10);
        assert_eq!(span.move_left(1), Span::new(4, 9));
        assert_eq!(span.move_left(2), Span::new(3, 8));
        assert_eq!(span.move_left(5), Span::new(0, 5));
    }

    #[test]
    #[should_panic(expected = "Cannot move span past zero length")]
    fn test_move_past_start() {
        let span = Span::new(5, 10);
        let _ = span.move_left(6);
    }

    #[test]
    fn test_move_right() {
        let span: Span = Span::new(5, 10);
        assert_eq!(span.move_right(1), Span::new(6, 11));
        assert_eq!(span.move_right(2), Span::new(7, 12));
        assert_eq!(
            span.move_right(u32::MAX.saturating_sub(10)),
            Span::new(u32::MAX.saturating_sub(5), u32::MAX)
        );
    }

    #[test]
    #[should_panic(expected = "Cannot move span past `u32::MAX` length")]
    fn test_move_past_end() {
        let span = Span::new(u32::MAX.saturating_sub(2), u32::MAX.saturating_sub(1));
        let _ = span.move_right(2);
    }
}

#[cfg(test)]
mod doctests {
    use super::Span;

    /// Tests from [`Span`] docs, since rustdoc test runner is disabled
    #[test]
    fn doctest() {
        // 1
        let text = "foo bar baz";
        let span = Span::new(4, 7);
        assert_eq!(&text[span], "bar");

        // 2
        let a = Span::new(5, 10); // Start and end offsets
        let b = Span::sized(5, 5); // Start offset and size
        assert_eq!(a, b);

        // 3
        let s = Span::new(5, 10);
        assert_eq!(s.shrink(2), Span::new(7, 8));
        assert_eq!(s.shrink(2), s.shrink_left(2).shrink_right(2));

        assert_eq!(s.expand(5), Span::new(0, 15));
        assert_eq!(s.expand(5), s.expand_left(5).expand_right(5));
    }
}

#[cfg(test)]
mod size_asserts {
    use std::mem::{align_of, size_of};

    use super::Span;

    const _: () = assert!(size_of::<Span>() == 8);

    #[cfg(target_pointer_width = "64")]
    const _: () = assert!(align_of::<Span>() == 8);

    #[cfg(not(target_pointer_width = "64"))]
    const _: () = assert!(align_of::<Span>() == 4);
}
