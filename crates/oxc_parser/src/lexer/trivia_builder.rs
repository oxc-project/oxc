use oxc_ast::{CommentKind, Trivias};

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
        self.trivias.push((start + 2, end, CommentKind::SingleLine));
    }

    /// skip leading `/*` and trailing `*/`
    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        self.trivias.push((start + 2, end - 2, CommentKind::MultiLine));
    }
}
