//! Trivia (called that because it's trivial) represent the parts of the source text that are largely insignificant for normal understanding of the code.
//! For example; whitespace, comments, and even conflict markers.

use std::collections::BTreeMap;

use crate::Span;

#[derive(Debug, Default)]
pub struct Trivias {
    /// Keyed by span.start
    comments: BTreeMap<u32, Comment>,

    configuration_comments: BTreeMap<u32, Comment>,
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct Comment {
    kind: CommentKind,
    end: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum CommentKind {
    ConfigurationSingleLine,
    SingleLine,
    MultiLine,
}

impl Comment {
    #[must_use]
    pub fn new(end: u32, kind: CommentKind) -> Self {
        Self { kind, end }
    }
}

impl Trivias {
    #[must_use]
    pub fn has_comments_between(&self, span: Span) -> bool {
        self.comments.range(span.start..span.end).count() > 0
            || self.configuration_comments.range(span.start..span.end).count() > 0
    }

    pub fn add_comment(&mut self, span: Span, kind: CommentKind) {
        let comment = Comment::new(span.end, kind);
        match kind {
            CommentKind::ConfigurationSingleLine => {
                self.configuration_comments.insert(span.start, comment)
            }
            _ => self.comments.insert(span.start, comment),
        }
    }
}
