use oxc_ast::{Comment, CommentKind, CommentPosition, Trivias};
use oxc_span::Span;

#[derive(Debug)]
pub struct TriviaBuilder {
    // NOTE(lucab): This is a set of unique comments. Duplicated
    // comments could be generated in case of rewind; they are
    // filtered out at insertion time.
    pub(crate) comments: Vec<Comment>,

    irregular_whitespaces: Vec<Span>,

    // index of processed comments
    processed: usize,

    saw_newline: bool,
}

impl Default for TriviaBuilder {
    fn default() -> Self {
        Self { comments: vec![], irregular_whitespaces: vec![], processed: 0, saw_newline: true }
    }
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        Trivias::new(self.comments.into_boxed_slice(), self.irregular_whitespaces)
    }

    pub fn add_single_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `//`
        self.add_comment(Comment::new(start + 2, end, CommentKind::SingleLine));
    }

    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `/*` and trailing `*/`
        self.add_comment(Comment::new(start + 2, end - 2, CommentKind::MultiLine));
    }

    pub fn handle_newline(&mut self) {
        if let Some(c) = self.comments.last_mut() {
            if c.followed_by_newline.is_none() {
                c.followed_by_newline.replace(true);
            }
        }
        if !self.saw_newline {
            if let Some(comments) = self.comments.get_mut(self.processed..) {
                for comment in comments {
                    comment.position = CommentPosition::Trailing;
                }
                self.processed = self.comments.len();
            }
        }
        self.saw_newline = true;
    }

    pub fn handle_token(&mut self, attached_to: u32) {
        if let Some(c) = self.comments.last_mut() {
            if c.followed_by_newline.is_none() {
                c.followed_by_newline.replace(false);
            }
        }
        if let Some(comments) = self.comments.get_mut(self.processed..) {
            for comment in comments {
                comment.position = CommentPosition::Leading;
                comment.attached_to = attached_to;
            }
            self.processed = self.comments.len();
        }
        self.saw_newline = false;
    }

    fn add_comment(&mut self, comment: Comment) {
        // The comments array is an ordered vec, only add the comment if its not added before,
        // to avoid situations where the parser needs to rewind and tries to reinsert the comment.
        if let Some(last_comment) = self.comments.last() {
            if comment.span.start <= last_comment.span.start {
                return;
            }
        }

        let mut comment = comment;
        if comment.kind.is_single_line() && !self.saw_newline {
            comment.position = CommentPosition::Trailing;
            if let Some(comments) = self.comments.get_mut(self.processed..) {
                for comment in comments {
                    comment.position = CommentPosition::Trailing;
                }
                self.processed = self.comments.len() + 1; // +1 for the newly added comment below.
            }
            self.saw_newline = true;
        }
        if self.saw_newline {
            comment.preceded_by_newline.replace(true);
        }

        self.comments.push(comment);
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.irregular_whitespaces.push(Span::new(start, end));
    }
}
