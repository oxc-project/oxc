//! Trivias such as comments and irregular whitespaces

use std::{
    iter::FusedIterator,
    ops::{Bound, RangeBounds},
};

use oxc_span::Span;

use crate::ast::comment::*;

pub fn comments_range<R>(comments: &[Comment], range: R) -> CommentsRange<'_>
where
    R: RangeBounds<u32>,
{
    CommentsRange::new(comments, range.start_bound().cloned(), range.end_bound().cloned())
}

pub fn has_comments_between(comments: &[Comment], span: Span) -> bool {
    comments_range(comments, span.start..span.end).count() > 0
}

/// Double-ended iterator over a range of comments, by starting position.
pub struct CommentsRange<'a> {
    comments: &'a [Comment],
    range: (Bound<u32>, Bound<u32>),
    current_start: usize,
    current_end: usize,
}

impl<'a> CommentsRange<'a> {
    fn new(comments: &'a [Comment], start: Bound<u32>, end: Bound<u32>) -> Self {
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

impl<'c> DoubleEndedIterator for CommentsRange<'c> {
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
