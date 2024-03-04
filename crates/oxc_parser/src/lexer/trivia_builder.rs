use oxc_ast::{CommentKind, Trivias};
use oxc_span::Span;

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    trivias: Trivias,
}

#[derive(Debug, Clone, Copy)]
pub struct TriviasCheckpoint {
    comments_len: usize,
    irregular_whitespaces_len: usize,
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        self.trivias
    }

    pub fn checkpoint(&self) -> TriviasCheckpoint {
        TriviasCheckpoint {
            comments_len: self.trivias.comments.len(),
            irregular_whitespaces_len: self.trivias.irregular_whitespaces.len(),
        }
    }

    pub fn rewind(&mut self, checkpoint: TriviasCheckpoint) {
        self.trivias.comments.truncate(checkpoint.comments_len);
        self.trivias.irregular_whitespaces.truncate(checkpoint.irregular_whitespaces_len);
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
