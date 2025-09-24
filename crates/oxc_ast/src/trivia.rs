//! Trivia such as comments and irregular whitespaces
//!
//! This module provides utilities for working with source code comments,
//! including efficient range-based queries and iteration over comment collections.

use std::{
    iter::FusedIterator,
    ops::{Bound, RangeBounds},
};

use oxc_span::Span;

use crate::ast::comment::*;

/// Create an iterator over comments within a given range
///
/// Returns a double-ended iterator that yields comments whose starting positions
/// fall within the specified range bounds.
///
/// # Arguments
///
/// * `comments` - A slice of comments sorted by starting position
/// * `range` - The range of positions to filter comments by
///
/// # Examples
///
/// ```ignore
/// let comments_in_range = comments_range(&all_comments, 10..50);
/// for comment in comments_in_range {
///     // Process comments starting between positions 10 and 50
/// }
/// ```
pub fn comments_range<R>(comments: &[Comment], range: R) -> CommentsRange<'_>
where
    R: RangeBounds<u32>,
{
    CommentsRange::new(comments, range.start_bound().cloned(), range.end_bound().cloned())
}

/// Check if there are any comments within a given span
///
/// Returns `true` if at least one comment starts within the specified span.
///
/// # Arguments
///
/// * `comments` - A slice of comments sorted by starting position
/// * `span` - The span to check for comments
///
/// # Examples
///
/// ```ignore
/// if has_comments_between(&comments, node.span) {
///     // Handle nodes that have comments within their span
/// }
/// ```
pub fn has_comments_between(comments: &[Comment], span: Span) -> bool {
    comments_range(comments, span.start..span.end).count() > 0
}

/// Double-ended iterator over a range of comments, by starting position
///
/// This iterator efficiently filters comments based on their starting positions,
/// using binary search to skip comments outside the range. It supports both
/// forward and backward iteration.
///
/// The iterator is created using [`comments_range`] and yields references to
/// comments whose `span.start` falls within the specified range bounds.
pub struct CommentsRange<'c> {
    comments: &'c [Comment],
    range: (Bound<u32>, Bound<u32>),
    current_start: usize,
    current_end: usize,
}

impl<'c> CommentsRange<'c> {
    /// Create a new iterator over comments in the specified range
    ///
    /// Uses `partition_point` to efficiently skip comments outside the range,
    /// avoiding unnecessary iteration over comments that won't be yielded.
    fn new(comments: &'c [Comment], start: Bound<u32>, end: Bound<u32>) -> Self {
        // Directly skip all comments that are already known to start
        // outside the requested range.
        let partition_start = {
            let range_start = match start {
                Bound::Unbounded => 0,
                Bound::Included(x) => x,
                Bound::Excluded(x) => x.saturating_add(1),
            };
            comments.partition_point(|comment| comment.span.start < range_start)
        };
        let partition_end = {
            let range_end = match end {
                Bound::Unbounded => u32::MAX,
                Bound::Included(x) => x,
                Bound::Excluded(x) => x.saturating_sub(1),
            };
            comments.partition_point(|comment| comment.span.start <= range_end)
        };
        Self {
            comments,
            range: (start, end),
            current_start: partition_start,
            current_end: partition_end,
        }
    }
}

impl<'c> Iterator for CommentsRange<'c> {
    type Item = &'c Comment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_start < self.current_end {
            for comment in &self.comments[self.current_start..self.current_end] {
                self.current_start = self.current_start.saturating_add(1);
                if self.range.contains(&comment.span.start) {
                    return Some(comment);
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let max_remaining = self.current_end.saturating_sub(self.current_start);
        (0, Some(max_remaining))
    }
}

impl DoubleEndedIterator for CommentsRange<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current_start < self.current_end {
            for comment in self.comments[self.current_start..self.current_end].iter().rev() {
                self.current_end = self.current_end.saturating_sub(1);
                if self.range.contains(&comment.span.start) {
                    return Some(comment);
                }
            }
        }
        None
    }
}

impl FusedIterator for CommentsRange<'_> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comments_range() {
        let comments = vec![
            Comment::new(0, 4, CommentKind::Line),
            Comment::new(5, 9, CommentKind::Line),
            Comment::new(10, 13, CommentKind::Line),
            Comment::new(14, 17, CommentKind::Line),
            Comment::new(18, 23, CommentKind::Line),
        ]
        .into_boxed_slice();
        let full_len = comments.len();
        assert_eq!(comments_range(&comments, ..).count(), full_len);
        assert_eq!(comments_range(&comments, 1..).count(), full_len.saturating_sub(1));
        assert_eq!(comments_range(&comments, ..18).count(), full_len.saturating_sub(1));
        assert_eq!(comments_range(&comments, ..=18).count(), full_len);
    }
}
