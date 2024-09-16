use oxc_ast::{Comment, CommentKind, CommentPosition, Trivias};
use oxc_span::Span;

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    // This is a set of unique comments. Duplicated
    // comments could be generated in case of rewind; they are
    // filtered out at insertion time.
    pub(crate) comments: Vec<Comment>,

    irregular_whitespaces: Vec<Span>,

    // states
    /// index of processed comments
    processed: usize,

    /// Saw a newline before this position
    saw_newline: bool,
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        Trivias::new(self.comments.into_boxed_slice(), self.irregular_whitespaces)
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.irregular_whitespaces.push(Span::new(start, end));
    }

    pub fn add_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `//`
        self.add_comment(Comment::new(start + 2, end, CommentKind::Line));
    }

    pub fn add_block_comment(&mut self, start: u32, end: u32) {
        // skip leading `/*` and trailing `*/`
        self.add_comment(Comment::new(start + 2, end - 2, CommentKind::Block));
    }

    // For block comments only. This function is not called after line comments because the lexer skips
    // newline after line comments.
    pub fn handle_newline(&mut self) {
        // The last unprocessed comment is on a newline.
        let len = self.comments.len();
        if self.processed < len {
            self.comments[len - 1].followed_by_newline = true;
        }
        self.saw_newline = true;
    }

    pub fn handle_token(&mut self, token_start: u32) {
        let len = self.comments.len();
        if self.processed < len {
            // All unprocess preceding comments are leading comments attached to this token start.
            for comment in &mut self.comments[self.processed..] {
                comment.position = CommentPosition::Leading;
                comment.attached_to = token_start;
            }
            self.processed = len;
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
        // This newly added comment may be preceded by a newline.
        comment.preceded_by_newline = self.saw_newline;
        if comment.is_line() {
            // A line comment is always followed by a newline. This is never set in `handle_newline`.
            comment.followed_by_newline = true;
            // A line comment is trailing when it is no preceded by a newline.
            if !self.saw_newline {
                self.processed = self.comments.len() + 1; // +1 to include this comment.
            }
            self.saw_newline = true;
        }

        self.comments.push(comment);
    }
}

#[cfg(test)]
mod test {
    use crate::Parser;
    use oxc_allocator::Allocator;
    use oxc_ast::{Comment, CommentKind, CommentPosition};
    use oxc_span::{SourceType, Span};

    #[test]
    fn comment_attachments() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source_text = "
        /* Leading 1 */
        // Leading 2
        /* Leading 3 */ token /* Trailing 1 */ // Trailing 2
        // Leading of EOF token
        ";
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let comments = ret.trivias.comments().copied().collect::<Vec<_>>();
        let expected = [
            Comment {
                span: Span::new(11, 22),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 70,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(35, 45),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 70,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(56, 67),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 70,
                preceded_by_newline: true,
                followed_by_newline: false,
            },
            Comment {
                span: Span::new(78, 90),
                kind: CommentKind::Block,
                position: CommentPosition::Trailing,
                attached_to: 0,
                preceded_by_newline: false,
                followed_by_newline: false,
            },
            Comment {
                span: Span::new(95, 106),
                kind: CommentKind::Line,
                position: CommentPosition::Trailing,
                attached_to: 0,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(117, 138),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 147,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
        ];

        assert_eq!(comments.len(), expected.len());
        for (comment, expected) in comments.iter().copied().zip(expected) {
            assert_eq!(comment, expected, "{}", comment.real_span().source_text(source_text));
        }
    }

    #[test]
    fn comment_attachments2() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source_text = "#!/usr/bin/env node
/* Leading 1 */
token
        ";
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let comments = ret.trivias.comments().copied().collect::<Vec<_>>();
        let expected = vec![Comment {
            span: Span::new(22, 33),
            kind: CommentKind::Block,
            position: CommentPosition::Leading,
            attached_to: 36,
            preceded_by_newline: false, // hashbang comment always end in newline
            followed_by_newline: true,
        }];
        assert_eq!(comments, expected);
    }
}
