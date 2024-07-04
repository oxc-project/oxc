//! Trivias such as comments and irregular whitespaces

use std::{
    iter::FusedIterator,
    ops::{Bound, Deref, RangeBounds},
    sync::Arc,
};

use oxc_span::Span;

/// Single or multiline comment
#[derive(Debug, Clone, Copy)]
pub struct Comment {
    pub kind: CommentKind,
    pub end: u32,
}

impl Comment {
    pub fn new(end: u32, kind: CommentKind) -> Self {
        Self { kind, end }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    SingleLine,
    MultiLine,
}

impl CommentKind {
    pub fn is_single_line(self) -> bool {
        matches!(self, Self::SingleLine)
    }

    pub fn is_multi_line(self) -> bool {
        matches!(self, Self::MultiLine)
    }
}

/// Sorted set of unique trivia comments, in ascending order by starting position.
pub type SortedComments = Box<[(u32, Comment)]>;

#[derive(Debug, Clone, Default)]
pub struct Trivias(Arc<TriviasImpl>);

#[derive(Debug, Default)]
pub struct TriviasImpl {
    /// Unique comments, ordered by increasing span-start.
    comments: SortedComments,

    irregular_whitespaces: Vec<Span>,
}

impl Deref for Trivias {
    type Target = TriviasImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Trivias {
    pub fn new(comments: SortedComments, irregular_whitespaces: Vec<Span>) -> Trivias {
        Self(Arc::new(TriviasImpl { comments, irregular_whitespaces }))
    }

    pub fn comments(&self) -> impl Iterator<Item = (CommentKind, Span)> + '_ {
        self.comments.iter().map(|(start, comment)| (comment.kind, Span::new(*start, comment.end)))
    }

    pub fn comments_range<R>(&self, range: R) -> CommentsRange<'_>
    where
        R: RangeBounds<u32>,
    {
        CommentsRange::new(&self.comments, range.start_bound().cloned(), range.end_bound().cloned())
    }

    pub fn has_comments_between(&self, span: Span) -> bool {
        self.comments_range(span.start..span.end).count() > 0
    }

    pub fn irregular_whitespaces(&self) -> &Vec<Span> {
        &self.irregular_whitespaces
    }
}

/// Double-ended iterator over a range of comments, by starting position.
pub struct CommentsRange<'a> {
    comments: &'a [(u32, Comment)],
    range: (Bound<u32>, Bound<u32>),
    current_start: usize,
    current_end: usize,
}

impl<'a> CommentsRange<'a> {
    fn new(comments: &'a [(u32, Comment)], start: Bound<u32>, end: Bound<u32>) -> Self {
        // Directly skip all comments that are already known to start
        // outside the requested range.
        let partition_start = {
            let range_start = match start {
                Bound::Unbounded => 0,
                Bound::Included(x) => x,
                Bound::Excluded(x) => x.saturating_add(1),
            };
            comments.partition_point(|(start, _)| *start < range_start)
        };
        let partition_end = {
            let range_end = match end {
                Bound::Unbounded => u32::MAX,
                Bound::Included(x) => x,
                Bound::Excluded(x) => x.saturating_sub(1),
            };
            comments.partition_point(|(start, _)| *start <= range_end)
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
    type Item = (&'c u32, &'c Comment);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_start < self.current_end {
            for (start, comment) in &self.comments[self.current_start..self.current_end] {
                self.current_start = self.current_start.saturating_add(1);
                if self.range.contains(start) {
                    return Some((start, comment));
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
            for (start, comment) in self.comments[self.current_start..self.current_end].iter().rev()
            {
                self.current_end = self.current_end.saturating_sub(1);
                if self.range.contains(start) {
                    return Some((start, comment));
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
        let comments: SortedComments = vec![
            (0, Comment { end: 4, kind: CommentKind::SingleLine }),
            (5, Comment { end: 9, kind: CommentKind::SingleLine }),
            (10, Comment { end: 13, kind: CommentKind::SingleLine }),
            (14, Comment { end: 17, kind: CommentKind::SingleLine }),
            (18, Comment { end: 23, kind: CommentKind::SingleLine }),
        ]
        .into_boxed_slice();
        let full_len = comments.len();
        let trivias = Trivias::new(comments, vec![]);
        assert_eq!(trivias.comments_range(..).count(), full_len);
        assert_eq!(trivias.comments_range(1..).count(), full_len.saturating_sub(1));
        assert_eq!(trivias.comments_range(..18).count(), full_len.saturating_sub(1));
        assert_eq!(trivias.comments_range(..=18).count(), full_len);
    }
}
