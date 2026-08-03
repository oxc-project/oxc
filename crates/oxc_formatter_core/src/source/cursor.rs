//! Positional cursor over a span-sorted slice.
//!
//! Hands out unprinted items in span order:
//! consumers drain runs with [`SpanCursor::take_before`] as printing advances past source positions.
//! What the items mean (comments) and where they are placed in the output stay consumer-owned.
//! core only provides the ordering mechanics, the same split as `SourceText` (core addresses, consumers interpret).
//!
//! Language crates alias it to their item type (`type Comments<'a> = SpanCursor<'a, CssComment>`),
//! and compose it with `SourceText` + `spec::classify_gap` at their print sites:
//! the cursor supplies positions, `SourceText` slices bytes, `classify_gap` interprets the gap.

use std::cell::Cell;

use oxc_span::GetSpan;

/// Cursor over a span-sorted slice that hands out unprinted items in order.
///
/// `cursor` is a [`Cell`] so the API works through `&self`,
/// allowing simultaneous borrows alongside other context fields
/// (the `Format` trait dispatches via `&self`,
/// so a `&mut` accessor would force every drain site through `context_mut()`
/// and conflict with read-only context accesses).
pub struct SpanCursor<'a, T> {
    inner: &'a [T],
    cursor: Cell<usize>,
}

impl<'a, T: GetSpan> SpanCursor<'a, T> {
    /// `items` must be sorted by span (source order).
    pub fn new(items: &'a [T]) -> Self {
        Self { inner: items, cursor: Cell::new(0) }
    }

    /// Returns unprinted items whose `span.end <= upper_bound`,
    /// and advances the cursor past them so they won't be returned again.
    pub fn take_before(&self, upper_bound: u32) -> &'a [T] {
        let start = self.cursor.get();
        let mut end = start;
        while end < self.inner.len() && self.inner[end].span().end <= upper_bound {
            end += 1;
        }
        self.cursor.set(end);
        &self.inner[start..end]
    }

    /// Drains all remaining unprinted items and returns them.
    pub fn take_remaining(&self) -> &'a [T] {
        let start = self.cursor.get();
        self.cursor.set(self.inner.len());
        &self.inner[start..]
    }
}

impl<T: GetSpan + Copy> SpanCursor<'_, T> {
    /// Returns the next unprinted item without consuming it.
    pub fn peek(&self) -> Option<T> {
        self.inner.get(self.cursor.get()).copied()
    }

    /// Iterator over unprinted items whose `span.end <= upper_bound`.
    /// Does NOT advance the cursor;
    /// callers that want to mark these as printed must call [`Self::take_before`] instead.
    pub fn iter_before(&self, upper_bound: u32) -> impl Iterator<Item = T> {
        let start = self.cursor.get();
        self.inner[start..].iter().copied().take_while(move |item| item.span().end <= upper_bound)
    }

    /// Returns the most recently consumed item, if any.
    /// ([`Self::peek`]'s mirror on the consumed side)
    /// Lets consumers re-anchor position measurements after a drain consumed items past their own anchor.
    pub fn last_consumed(&self) -> Option<T> {
        self.cursor.get().checked_sub(1).map(|i| self.inner[i])
    }
}

#[cfg(test)]
mod tests {
    use oxc_span::Span;

    use super::SpanCursor;

    fn spans() -> Vec<Span> {
        vec![Span::new(0, 2), Span::new(4, 6), Span::new(8, 10)]
    }

    #[test]
    fn take_before_drains_in_order() {
        let items = spans();
        let cursor = SpanCursor::new(&items);
        assert_eq!(cursor.peek().map(|s| s.start), Some(0));
        assert_eq!(cursor.take_before(6).len(), 2);
        // Consumed items are never returned again.
        assert_eq!(cursor.take_before(6).len(), 0);
        assert_eq!(cursor.peek().map(|s| s.start), Some(8));
        assert_eq!(cursor.take_remaining().len(), 1);
        assert_eq!(cursor.peek(), None);
    }

    #[test]
    fn iter_before_does_not_advance() {
        let items = spans();
        let cursor = SpanCursor::new(&items);
        assert_eq!(cursor.iter_before(10).count(), 3);
        assert_eq!(cursor.iter_before(5).count(), 1);
        // Cursor unchanged: everything still pending.
        assert_eq!(cursor.take_remaining().len(), 3);
    }

    #[test]
    fn last_consumed_tracks_drains() {
        let items = spans();
        let cursor = SpanCursor::new(&items);
        assert!(cursor.last_consumed().is_none());
        cursor.take_before(2);
        assert_eq!(cursor.last_consumed().map(|s| s.end), Some(2));
        // A bound before the next item consumes nothing and keeps the last.
        cursor.take_before(3);
        assert_eq!(cursor.last_consumed().map(|s| s.end), Some(2));
    }
}
