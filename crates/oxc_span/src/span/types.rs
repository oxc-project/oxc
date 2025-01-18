use oxc_ast_macros::ast;
use oxc_estree::ESTree;

use super::PointerAlign;

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
/// same content), use [`ContentEq`](crate::cmp::ContentEq) instead.
///
/// ## Implementation Notes
/// See the [`text-size`](https://docs.rs/text-size) crate for details.
/// Utility methods can be copied from the `text-size` crate if they are needed.
///
/// [`expand`]: Span::expand
/// [`shrink`]: Span::shrink
#[ast(visit)]
#[derive(Default, Clone, Copy, Eq, PartialOrd, Ord)]
#[generate_derive(ESTree)]
#[estree(no_type, always_flatten)]
pub struct Span {
    /// The zero-based start offset of the span
    pub start: u32,
    /// The zero-based end offset of the span. This may be equal to [`start`](Span::start) if
    /// the span is empty, but should not be less than it.
    pub end: u32,
    /// Align `Span` on 8 on 64-bit platforms
    #[estree(skip)]
    pub(super) _align: PointerAlign,
}

#[cfg(test)]
mod test {
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
