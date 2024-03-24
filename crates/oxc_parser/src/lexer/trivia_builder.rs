use std::collections::BTreeMap;

use oxc_ast::{Comment, CommentKind, Trivias};
use oxc_span::Span;

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    // Duplicated comments can be added from rewind, use `BTreeMap` to ensure uniqueness
    comments: BTreeMap<u32, Comment>,
    irregular_whitespaces: Vec<Span>,
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        Trivias::new(self.comments, self.irregular_whitespaces)
    }

    pub fn add_single_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `//`
        self.comments.insert(start + 2, Comment::new(end, CommentKind::SingleLine));
    }

    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `/*` and trailing `*/`
        self.comments.insert(start + 2, Comment::new(end - 2, CommentKind::MultiLine));
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.irregular_whitespaces.push(Span::new(start, end));
    }
}
