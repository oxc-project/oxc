use std::rc::Rc;

use oxc_ast::{CommentKind, Span, Trivias};

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    trivias: Trivias,
}

impl TriviaBuilder {
    pub fn build(self) -> Rc<Trivias> {
        Rc::new(self.trivias)
    }

    pub fn add_single_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `//`
        self.trivias.add_comment(Span::new(start + 2, end), CommentKind::SingleLine);
    }

    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `/*` and trailing `*/`
        self.trivias.add_comment(Span::new(start + 2, end - 2), CommentKind::MultiLine);
    }
}
