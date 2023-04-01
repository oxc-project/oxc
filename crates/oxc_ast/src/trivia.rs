use std::collections::BTreeMap;

use crate::Span;

/// Trivias such as comments
///
/// Trivia (called that because it's trivial) represent the parts of the source text that are largely insignificant for normal understanding of the code.
/// For example: whitespace, comments, and even conflict markers.
#[derive(Debug, Default)]
pub struct Trivias {
    /// Keyed by span.start
    comments: BTreeMap<u32, Comment>,
}

/// Single or multi line comment
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct Comment {
    kind: CommentKind,
    end: u32,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    SingleLine,
    MultiLine,
}

impl Comment {
    #[must_use]
    pub fn new(end: u32, kind: CommentKind) -> Self {
        Self { kind, end }
    }

    #[must_use]
    pub fn end(self) -> u32 {
        self.end
    }

    #[must_use]
    pub fn is_single_line(self) -> bool {
        matches!(self.kind, CommentKind::SingleLine)
    }

    #[must_use]
    pub fn is_multi_line(self) -> bool {
        matches!(self.kind, CommentKind::MultiLine)
    }
}

impl Trivias {
    #[must_use]
    pub fn comments(&self) -> &BTreeMap<u32, Comment> {
        &self.comments
    }

    #[must_use]
    pub fn has_comments_between(&self, span: Span) -> bool {
        self.comments.range(span.start..span.end).count() > 0
    }

    pub fn add_comment(&mut self, span: Span, kind: CommentKind) {
        let comment = Comment::new(span.end, kind);
        self.comments.insert(span.start, comment);
    }
}
