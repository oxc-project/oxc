//! Trivias such as comments and irregular whitespaces

use std::{
    collections::btree_map::{BTreeMap, Range},
    ops::{Deref, RangeBounds},
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

pub type TriviasMap = BTreeMap<u32, Comment>;

#[derive(Debug, Clone, Default)]
pub struct Trivias(Arc<TriviasImpl>);

#[derive(Debug, Default)]
pub struct TriviasImpl {
    /// Keyed by span.start
    comments: TriviasMap,

    irregular_whitespaces: Vec<Span>,
}

impl Deref for Trivias {
    type Target = TriviasImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Trivias {
    pub fn new(comments: TriviasMap, irregular_whitespaces: Vec<Span>) -> Trivias {
        Self(Arc::new(TriviasImpl { comments, irregular_whitespaces }))
    }

    pub fn comments(&self) -> impl Iterator<Item = (CommentKind, Span)> + '_ {
        self.comments.iter().map(|(start, comment)| (comment.kind, Span::new(*start, comment.end)))
    }

    pub fn comments_range<R>(&self, range: R) -> Range<'_, u32, Comment>
    where
        R: RangeBounds<u32>,
    {
        self.comments.range(range)
    }

    pub fn has_comments_between(&self, span: Span) -> bool {
        self.comments.range(span.start..span.end).count() > 0
    }

    pub fn irregular_whitespaces(&self) -> &Vec<Span> {
        &self.irregular_whitespaces
    }
}
