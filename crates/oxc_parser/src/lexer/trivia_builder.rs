use oxc_ast::{CommentKind, Trivias};
use oxc_span::Span;

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    trivias: Trivias,
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        self.trivias
    }

    /// skip leading `//`
    pub fn add_single_line_comment(&mut self, start: u32, end: u32) {
        self.trivias.comments.push((start + 2, end, CommentKind::SingleLine));
    }

    /// skip leading `/*` and trailing `*/`
    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        self.trivias.comments.push((start + 2, end - 2, CommentKind::MultiLine));
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.trivias.irregular_whitespaces.push(Span::new(start, end));
    }
}
