use oxc_ast::{Comment, CommentKind, Trivias, TriviasMap};
use oxc_span::Span;

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    // Duplicated comments can be added from rewind, use `BTreeMap` to ensure uniqueness
    comments: Vec<(u32, Comment)>,
    irregular_whitespaces: Vec<Span>,
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        let comments = TriviasMap::from_iter(self.comments);
        Trivias::new(comments, self.irregular_whitespaces)
    }

    pub fn add_single_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `//`
        self.add_comment(start + 2, Comment::new(end, CommentKind::SingleLine));
    }

    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `/*` and trailing `*/`
        self.add_comment(start + 2, Comment::new(end - 2, CommentKind::MultiLine));
    }

    fn add_comment(&mut self, start: u32, comment: Comment) {
        // The comments array is an ordered vec, only add the comment if its not added before,
        // to avoid situations where the parser needs to rewind and reinsert the comment.
        if let Some(comment) = self.comments.last_mut() {
            if start <= comment.0 {
                return;
            }
        }
        self.comments.push((start, comment));
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.irregular_whitespaces.push(Span::new(start, end));
    }
}
