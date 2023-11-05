//! Trivias such as comments

use std::collections::BTreeMap;

use oxc_span::Span;

/// A vec of trivias from the lexer, tupled by (span.start, span.end).
pub type Trivias = Vec<(u32, u32, CommentKind)>;

/// Trivias such as comments
///
/// Trivia (called that because it's trivial) represent the parts of the source text that are largely insignificant for normal understanding of the code.
/// For example: whitespace, comments, and even conflict markers.
#[derive(Debug, Default)]
pub struct TriviasMap {
    /// Keyed by span.start
    comments: BTreeMap<u32, Comment>,
}

impl From<Trivias> for TriviasMap {
    fn from(trivias: Trivias) -> Self {
        Self { comments: trivias.iter().map(|t| (t.0, Comment::new(t.1, t.2))).collect() }
    }
}

/// Single or multiline comment
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
    pub fn new(end: u32, kind: CommentKind) -> Self {
        Self { kind, end }
    }

    pub fn end(self) -> u32 {
        self.end
    }

    pub fn is_single_line(self) -> bool {
        matches!(self.kind, CommentKind::SingleLine)
    }

    pub fn is_multi_line(self) -> bool {
        matches!(self.kind, CommentKind::MultiLine)
    }
}

impl TriviasMap {
    pub fn comments(&self) -> &BTreeMap<u32, Comment> {
        &self.comments
    }

    pub fn has_comments_between(&self, span: Span) -> bool {
        self.comments.range(span.start..span.end).count() > 0
    }

    pub fn add_single_line_comment(&mut self, span: Span) {
        let comment = Comment::new(span.end, CommentKind::SingleLine);
        self.comments.insert(span.start, comment);
    }

    pub fn add_multi_line_comment(&mut self, span: Span) {
        let comment = Comment::new(span.end, CommentKind::MultiLine);
        self.comments.insert(span.start, comment);
    }

    pub fn comments_spans(&self) -> impl Iterator<Item = (Comment, Span)> + '_ {
        self.comments().iter().map(|(start, comment)| (*comment, Span::new(*start, comment.end)))
    }
}
