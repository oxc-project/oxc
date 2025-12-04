//! Trivia such as comments and irregular whitespaces
//!
//! This module provides utilities for working with source code comments,
//! including efficient range-based queries and iteration over comment collections.

use std::{
    cmp::Ordering,
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

/// Check if a position falls within any comment
///
/// Returns `true` if the specified position is inside any comment's span.
/// The start position is included, but the end position is excluded.
///
/// Uses binary search for efficient lookup in O(log n) time.
///
/// # Arguments
///
/// * `comments` - A slice of comments sorted by starting position
/// * `pos` - The position to check
///
/// # Examples
///
/// ```ignore
/// // Comment spans from position 10 to 20
/// assert!(is_inside_comment(&comments, 15));  // Inside comment
/// assert!(!is_inside_comment(&comments, 25)); // Outside comment
/// ```
pub fn is_inside_comment(comments: &[Comment], pos: u32) -> bool {
    get_comment_at(comments, pos).is_some()
}

/// Get the comment containing a position, if any
///
/// Returns a reference to the comment if the specified position is inside any
/// comment's span. The start position is included, but the end position is excluded.
///
/// Uses binary search for efficient lookup in O(log n) time.
///
/// # Arguments
///
/// * `comments` - A slice of comments sorted by starting position
/// * `pos` - The position to check
///
/// # Examples
///
/// ```ignore
/// // Comment spans from position 10 to 20
/// if let Some(comment) = get_comment_at(&comments, 15) {
///     // Position is inside this comment
///     println!("Comment spans {:?}", comment.span);
/// }
/// ```
pub fn get_comment_at(comments: &[Comment], pos: u32) -> Option<&Comment> {
    comments
        .binary_search_by(|c| {
            if pos < c.span.start {
                Ordering::Greater
            } else if pos >= c.span.end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
        .map(|idx| &comments[idx])
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

    #[test]
    fn test_is_inside_comment() {
        let comments = vec![
            Comment::new(0, 4, CommentKind::Line),
            Comment::new(10, 20, CommentKind::SinglelineBlock),
        ]
        .into_boxed_slice();

        assert!(is_inside_comment(&comments, 2));
        assert!(!is_inside_comment(&comments, 5));
        assert!(is_inside_comment(&comments, 15));
        assert!(!is_inside_comment(&comments, 21));
    }

    #[test]
    fn test_get_comment_at() {
        let comments = vec![
            Comment::new(0, 4, CommentKind::Line),
            Comment::new(10, 20, CommentKind::SinglelineBlock),
        ]
        .into_boxed_slice();

        // Inside first comment
        let comment = get_comment_at(&comments, 2);
        let comment = comment.expect("Expected a comment at position 2");
        assert_eq!(comment.span.start, 0);
        assert_eq!(comment.span.end, 4);

        // Between comments
        assert!(get_comment_at(&comments, 5).is_none());

        // Inside second comment
        let comment = get_comment_at(&comments, 15);
        let comment = comment.expect("Expected a comment at position 15");
        assert_eq!(comment.span.start, 10);
        assert_eq!(comment.span.end, 20);

        // After all comments
        assert!(get_comment_at(&comments, 21).is_none());

        // Boundary cases: start positions are included
        assert!(get_comment_at(&comments, 0).is_some()); // Start of first comment
        assert!(get_comment_at(&comments, 10).is_some()); // Start of second comment

        // Boundary cases: end positions are excluded
        assert!(get_comment_at(&comments, 4).is_none()); // End of first comment
        assert!(get_comment_at(&comments, 20).is_none()); // End of second comment
    }
}
