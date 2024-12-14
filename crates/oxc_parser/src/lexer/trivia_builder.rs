use oxc_ast::ast::{Comment, CommentKind, CommentPosition};
use oxc_span::Span;

use super::{Kind, Token};

#[derive(Debug)]
pub struct TriviaBuilder {
    // This is a set of unique comments. Duplicated
    // comments could be generated in case of rewind; they are
    // filtered out at insertion time.
    pub(crate) comments: Vec<Comment>,

    pub(crate) irregular_whitespaces: Vec<Span>,

    // states
    /// index of processed comments
    processed: usize,

    /// Saw a newline before this position
    saw_newline: bool,

    /// Previous token kind, used to indicates comments are trailing from what kind
    previous_kind: Kind,
}

impl Default for TriviaBuilder {
    fn default() -> Self {
        Self {
            comments: vec![],
            irregular_whitespaces: vec![],
            processed: 0,
            saw_newline: true,
            previous_kind: Kind::Undetermined,
        }
    }
}

impl TriviaBuilder {
    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.irregular_whitespaces.push(Span::new(start, end));
    }

    pub fn add_line_comment(&mut self, start: u32, end: u32) {
        self.add_comment(Comment::new(start, end, CommentKind::Line));
    }

    pub fn add_block_comment(&mut self, start: u32, end: u32) {
        self.add_comment(Comment::new(start, end, CommentKind::Block));
    }

    // For block comments only. This function is not called after line comments because the lexer skips
    // newline after line comments.
    pub fn handle_newline(&mut self) {
        // The last unprocessed comment is on a newline.
        let len = self.comments.len();
        if self.processed < len {
            self.comments[len - 1].followed_by_newline = true;
            if !self.saw_newline {
                self.processed = self.comments.len();
            }
        }
        self.saw_newline = true;
    }

    pub fn handle_token(&mut self, token: Token) {
        let len = self.comments.len();
        self.previous_kind = token.kind;
        if self.processed < len {
            // All unprocessed preceding comments are leading comments attached to this token start.
            for comment in &mut self.comments[self.processed..] {
                comment.position = CommentPosition::Leading;
                comment.attached_to = token.start;
            }
            self.processed = len;
        }
        self.saw_newline = false;
    }

    /// Determines if the current line comment should be treated as a trailing comment.
    ///
    /// A line comment should be treated as trailing when both of the following conditions are met:
    ///
    /// 1. It is not preceded by a newline.
    ///
    /// ```javascript
    /// let x = 5; // This should be treated as a trailing comment
    /// foo(); // This should also be treated as a trailing comment
    ///
    /// // This should not be treated as trailing (preceded by newline)
    /// let x = 5;
    /// ```
    ///
    /// 2. It does not immediately follow an `=` [`Kind::Eq`] or `(` [`Kind::LParen`]
    ///    token.
    ///
    /// ```javascript
    /// let y = // This should not be treated as trailing (follows `=`)
    ///     10;
    ///
    /// function foo( // This should not be treated as trailing (follows `(`)
    ///     param
    /// ) {}
    /// ```
    fn should_be_treated_as_trailing_comment(&self) -> bool {
        !self.saw_newline && !matches!(self.previous_kind, Kind::Eq | Kind::LParen)
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
            if self.should_be_treated_as_trailing_comment() {
                self.processed = self.comments.len() + 1; // +1 to include this comment.
            }
            self.saw_newline = true;
        }

        self.comments.push(comment);
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::{Comment, CommentKind, CommentPosition};
    use oxc_span::{SourceType, Span};

    use crate::Parser;

    fn get_comments(source_text: &str) -> Vec<Comment> {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        ret.program.comments.iter().copied().collect::<Vec<_>>()
    }

    #[test]
    fn comment_attachments() {
        let source_text = "
        /* Leading 1 */
        // Leading 2
        /* Leading 3 */ token /* Trailing 1 */ // Trailing 2
        // Leading of EOF token
        ";
        let comments = get_comments(source_text);
        let expected = [
            Comment {
                span: Span::new(9, 24),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 70,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(33, 45),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 70,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(54, 69),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 70,
                preceded_by_newline: true,
                followed_by_newline: false,
            },
            Comment {
                span: Span::new(76, 92),
                kind: CommentKind::Block,
                position: CommentPosition::Trailing,
                attached_to: 0,
                preceded_by_newline: false,
                followed_by_newline: false,
            },
            Comment {
                span: Span::new(93, 106),
                kind: CommentKind::Line,
                position: CommentPosition::Trailing,
                attached_to: 0,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(115, 138),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 147,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
        ];

        assert_eq!(comments.len(), expected.len());
        for (comment, expected) in comments.iter().copied().zip(expected) {
            assert_eq!(comment, expected, "{}", comment.content_span().source_text(source_text));
        }
    }

    #[test]
    fn comment_attachments2() {
        let source_text = "#!/usr/bin/env node
/* Leading 1 */
token /* Trailing 1 */
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(20, 35),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 36,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(42, 58),
                kind: CommentKind::Block,
                position: CommentPosition::Trailing,
                attached_to: 0,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn comment_attachments3() {
        let source_text = "
/**
 * A
 **/
/**
 * B
 **/
 token
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(1, 14),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 30,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(15, 28),
                kind: CommentKind::Block,
                position: CommentPosition::Leading,
                attached_to: 30,
                preceded_by_newline: true,
                followed_by_newline: true,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn leading_comments_after_eq() {
        let source_text = "
            const v1 = // Leading comment 1
            foo();
            function foo(param =// Leading comment 2
            new Foo()
            ) {}
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(24, 44),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 57,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(96, 116),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 129,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn leading_comments_after_left_parenthesis() {
        let source_text = "
            call(// Leading comment 1
                arguments)
            (// Leading comment 2
                arguments)
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(18, 38),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 55,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
            Comment {
                span: Span::new(79, 99),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 116,
                preceded_by_newline: false,
                followed_by_newline: true,
            },
        ];
        assert_eq!(comments, expected);
    }
}
