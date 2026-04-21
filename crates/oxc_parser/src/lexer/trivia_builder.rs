use memchr::memchr_iter;
use oxc_ast::ast::{Comment, CommentContent, CommentKind, CommentPosition};
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

    /// Saw a newline before this position (since last token).
    /// Used to determine if comments are trailing comments of the previous token.
    saw_newline: bool,

    /// Saw a newline before this position (since last comment or token).
    /// Used to set `preceded_by_newline` on comments.
    saw_newline_for_comment: bool,

    /// Previous token kind, used to indicates comments are trailing from what kind
    previous_kind: Kind,

    /// Index of the pure comment in `comments` vec, or `None` if no pure comment for the current token.
    pub(super) pure_comment: Option<usize>,

    pub(super) has_no_side_effects_comment: bool,
}

impl Default for TriviaBuilder {
    fn default() -> Self {
        Self {
            comments: vec![],
            irregular_whitespaces: vec![],
            processed: 0,
            saw_newline: true,
            saw_newline_for_comment: true,
            previous_kind: Kind::Undetermined,
            pure_comment: None,
            has_no_side_effects_comment: false,
        }
    }
}

impl TriviaBuilder {
    pub fn previous_token_has_pure_comment(&self) -> Option<usize> {
        self.pure_comment
    }

    pub fn previous_token_has_no_side_effects_comment(&self) -> bool {
        self.has_no_side_effects_comment
    }

    pub fn mark_pure_comment_not_applied(&mut self, index: usize) {
        if let Some(comment) = self.comments.get_mut(index) {
            debug_assert!(comment.is_pure());
            comment.content = CommentContent::PureNotApplied;
        }
    }

    /// Mark the current token's pure comment (if any) as not applied.
    pub fn mark_current_pure_comment_not_applied(&mut self) {
        if let Some(index) = self.pure_comment {
            self.mark_pure_comment_not_applied(index);
        }
    }

    pub fn add_irregular_whitespace(&mut self, start: u32, end: u32) {
        self.irregular_whitespaces.push(Span::new(start, end));
    }

    pub fn add_line_comment(&mut self, start: u32, end: u32, source_text: &str) {
        self.add_comment(Comment::new(start, end, CommentKind::Line), source_text);
    }

    pub fn add_block_comment(
        &mut self,
        start: u32,
        end: u32,
        kind: CommentKind,
        source_text: &str,
    ) {
        self.add_comment(Comment::new(start, end, kind), source_text);
    }

    // For block comments only. This function is not called after line comments because the lexer skips
    // newline after line comments.
    pub fn handle_newline(&mut self) {
        // The last unprocessed comment is on a newline.
        let len = self.comments.len();
        if self.processed < len {
            self.comments[len - 1].set_followed_by_newline(true);
            if !self.saw_newline {
                self.processed = self.comments.len();
            }
        }
        self.saw_newline = true;
        self.saw_newline_for_comment = true;
    }

    pub fn handle_token(&mut self, token: Token) {
        let len = self.comments.len();
        self.previous_kind = token.kind();
        if self.processed < len {
            // All unprocessed preceding comments are leading comments attached to this token start.
            for comment in &mut self.comments[self.processed..] {
                comment.position = CommentPosition::Leading;
                comment.attached_to = token.start();
            }
            self.processed = len;
        }
        self.saw_newline = false;
        self.saw_newline_for_comment = false;
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

    /// Update `pure_comment` / `has_no_side_effects_comment` to point to the comment at `index`.
    fn set_annotation_flags(&mut self, comment: &Comment, index: usize) {
        if comment.is_pure() {
            self.pure_comment = Some(index);
        } else if comment.is_no_side_effects() {
            self.has_no_side_effects_comment = true;
        }
    }

    fn add_comment(&mut self, mut comment: Comment, source_text: &str) {
        Self::parse_annotation(&mut comment, source_text);
        // The comments array is an ordered vec, only add the comment if its not added before,
        // to avoid situations where the parser needs to rewind and tries to reinsert the comment.
        if let Some(last_comment) = self.comments.last()
            && comment.span.start <= last_comment.span.start
        {
            // Duplicate from parser lookahead/rewind — update annotation flags
            // to point to the existing comment.
            self.set_annotation_flags(&comment, self.comments.len() - 1);
            return;
        }

        // This newly added comment may be preceded by a newline.
        // Use `saw_newline_for_comment` which tracks newlines since the last comment or token,
        // not just since the last token.
        comment.set_preceded_by_newline(self.saw_newline_for_comment);
        if comment.is_line() {
            // A line comment is always followed by a newline. This is never set in `handle_newline`.
            comment.set_followed_by_newline(true);
            if self.should_be_treated_as_trailing_comment() {
                self.processed = self.comments.len() + 1; // +1 to include this comment.
            }
            self.saw_newline = true;
            self.saw_newline_for_comment = true;
        } else {
            // Block comments don't end with a newline, so reset saw_newline_for_comment.
            // If there's a newline after the block comment, `handle_newline` will set it back to true.
            self.saw_newline_for_comment = false;
        }

        // Set annotation flags here (not in `parse_annotation`) so the index is correct
        // even when the dedup check above skips a duplicate from parser lookahead/rewind.
        self.set_annotation_flags(&comment, self.comments.len());
        self.comments.push(comment);
    }

    /// Parse Notation
    fn parse_annotation(comment: &mut Comment, source_text: &str) {
        let s = comment.content_span().source_text(source_text);
        let bytes = s.as_bytes();

        // Early exit for empty comments
        if bytes.is_empty() {
            return;
        }

        // Check first byte for quick routing
        match bytes[0] {
            b'!' => {
                comment.content = CommentContent::Legal;
                return;
            }
            b'*' if comment.is_block() => {
                // Ignore webpack comment `/*****/`
                if !bytes.iter().all(|&c| c == b'*') {
                    if contains_license_or_preserve_comment(s) {
                        comment.content = CommentContent::JsdocLegal;
                    } else {
                        comment.content = CommentContent::Jsdoc;
                    }
                }
                return;
            }
            _ => {}
        }

        // Skip leading whitespace without allocation
        let mut start = 0;
        while start < bytes.len() && bytes[start].is_ascii_whitespace() {
            start += 1;
        }

        if start >= bytes.len() {
            return;
        }

        // Fast path: check first non-whitespace byte
        match bytes[start] {
            b'@' => {
                start += 1;
                if start >= bytes.len() {
                    return;
                }

                // Check for @vite, @license, @preserve
                if bytes[start..].starts_with(b"vite") {
                    comment.content = CommentContent::Vite;
                    return;
                }
                if bytes[start..].starts_with(b"license") || bytes[start..].starts_with(b"preserve")
                {
                    comment.content = CommentContent::Legal;
                    return;
                }

                // Continue to check for __PURE__ or __NO_SIDE_EFFECTS__ after @
            }
            b'#' => {
                start += 1;
                // Continue to check for __PURE__ or __NO_SIDE_EFFECTS__ after #
            }
            b'w' => {
                // Check for webpack comments
                if bytes[start..].starts_with(b"webpack")
                    && start + 7 < bytes.len()
                    && bytes[start + 7].is_ascii_uppercase()
                {
                    comment.content = CommentContent::Webpack;
                    return;
                }
                // Fall through to check for coverage ignore patterns
            }
            b't' => {
                // Check for turbopack comments
                if bytes[start..].starts_with(b"turbopack")
                    && start + 9 < bytes.len()
                    && bytes[start + 9].is_ascii_uppercase()
                {
                    comment.content = CommentContent::Turbopack;
                    return;
                }
                // Fall through to check for coverage ignore patterns
            }
            b'v' | b'c' | b'n' | b'i' => {
                // Check coverage ignore patterns: "v8 ignore", "c8 ignore", "node:coverage", "istanbul ignore"
                let rest = &bytes[start..];
                if rest.starts_with(b"v8 ignore")
                    || rest.starts_with(b"c8 ignore")
                    || rest.starts_with(b"node:coverage")
                    || rest.starts_with(b"istanbul ignore")
                {
                    comment.content = CommentContent::CoverageIgnore;
                    return;
                }
                // Fall through to check license/preserve
            }
            _ => {
                // Check for license/preserve comments in remaining cases
                if contains_license_or_preserve_comment(s) {
                    comment.content = CommentContent::Legal;
                }
                return;
            }
        }

        // Check for __PURE__ or __NO_SIDE_EFFECTS__ after @ or #
        if start < bytes.len() && bytes[start..].starts_with(b"__") {
            let rest = &bytes[start + 2..];
            if rest.starts_with(b"PURE__") {
                comment.content = CommentContent::Pure;
                return;
            } else if rest.starts_with(b"NO_SIDE_EFFECTS__") {
                comment.content = CommentContent::NoSideEffects;
                return;
            }
        }

        // Fallback: check for @license or @preserve anywhere in the comment
        // This handles cases like /* @foo @preserve */ where the first @ doesn't match known patterns
        if contains_license_or_preserve_comment(s) {
            comment.content = CommentContent::Legal;
        }
    }
}

#[expect(clippy::inline_always)]
#[inline(always)]
fn contains_license_or_preserve_comment(s: &str) -> bool {
    let hay = s.as_bytes();

    if hay.len() < 9 {
        return false;
    }

    let search_len = hay.len() - 8;

    for i in memchr_iter(b'@', &hay[..search_len]) {
        debug_assert!(i < search_len);
        // SAFETY: we `i` has a max val of len of bytes - 8, so accessing `i + 1` is safe
        match unsafe { hay.get_unchecked(i + 1) } {
            // spellchecker:off
            b'l'
                // SAFETY: we `i` has a max val of len of bytes - 8, so accessing `i + 7` is safe
                if unsafe { hay.get_unchecked(i + 2..i + 1 + 7) } == b"icense" =>
            {
                return true;
            }
            b'p'
                // SAFETY: we `i` has a max val of len of bytes - 8, so accessing `i + 8` is safe
                if unsafe { hay.get_unchecked(i + 2..i + 1 + 8) } == b"reserve" =>
            {
                return true;
            }
            // spellchecker:on
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::{Comment, CommentContent, CommentKind, CommentPosition, ast::CommentNewlines};
    use oxc_span::{SourceType, Span};

    use crate::Parser;

    fn get_comments(source_text: &str) -> Vec<Comment> {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.errors.is_empty());
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
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Leading,
                attached_to: 70,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(33, 45),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 70,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(54, 69),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Leading,
                attached_to: 70,
                newlines: CommentNewlines::Leading,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(76, 92),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Trailing,
                attached_to: 0,
                newlines: CommentNewlines::None,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(93, 106),
                kind: CommentKind::Line,
                position: CommentPosition::Trailing,
                attached_to: 0,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(115, 138),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 147,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
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
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Leading,
                attached_to: 36,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(42, 58),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Trailing,
                attached_to: 0,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn comment_attachments3() {
        let source_text = "
/*
 * A
 **/
/*
 * B
 **/
 token
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(1, 13),
                kind: CommentKind::MultiLineBlock,
                position: CommentPosition::Leading,
                attached_to: 28,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(14, 26),
                kind: CommentKind::MultiLineBlock,
                position: CommentPosition::Leading,
                attached_to: 28,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
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
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(96, 116),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 129,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
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
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(79, 99),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 116,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn pure_comment_not_applied() {
        let cases = [
            "/* #__PURE__ */ React.createElement;",
            "/* @__PURE__ */ someVariable;",
            "/* #__PURE__ */ 42;",
            "!/* #__PURE__ */ x;",
            // Non-expression statements
            "/* #__PURE__ */ function foo() {}",
            "/* #__PURE__ */ class Foo {}",
            "/* #__PURE__ */ var x = foo();",
            // Pure comment before `=` in variable declarator
            "const foo /* #__PURE__ */ = pureOperation();",
            // Pure comment before object literal (triggers parser lookahead/rewind for arrow detection)
            "export const X = /* @__PURE__ */ { a: 1 };",
        ];
        for source_text in cases {
            let comments = get_comments(source_text);
            assert_eq!(comments[0].content, CommentContent::PureNotApplied, "{source_text}");
        }
    }

    #[test]
    fn pure_comment_applied_after_lookahead() {
        // `export const X = /* @__PURE__ */ foo()` triggers arrow-function lookahead
        // due to the `{`-ambiguity path. The pure comment must still be correctly
        // applied to the call expression after the parser rewinds.
        let source_text = "export const X = /* @__PURE__ */ foo();";
        let comments = get_comments(source_text);
        assert_eq!(comments[0].content, CommentContent::Pure, "{source_text}");
    }

    #[test]
    fn pure_comment_not_applied_marks_correct_comment() {
        // The first pure comment is invalid (before `foo`), the second is valid (before `bar()`).
        // `mark_pure_comment_not_applied` must retag the first comment, not the second.
        let source_text = "/*#__PURE__*/ foo + /*#__PURE__*/ bar()";
        let comments = get_comments(source_text);
        assert_eq!(
            comments[0].content,
            CommentContent::PureNotApplied,
            "first comment should be PureNotApplied"
        );
        assert_eq!(comments[1].content, CommentContent::Pure, "second comment should remain Pure");
    }

    #[test]
    fn comment_parsing() {
        let data = [
            ("/*! legal */", CommentContent::Legal),
            ("/* @preserve */", CommentContent::Legal),
            ("/* @license */", CommentContent::Legal),
            ("/* foo @preserve */", CommentContent::Legal),
            ("/* foo @license */", CommentContent::Legal),
            ("/* @foo @preserve */", CommentContent::Legal),
            ("/* @foo @license */", CommentContent::Legal),
            ("/** foo @preserve */", CommentContent::JsdocLegal),
            ("/** foo @license */", CommentContent::JsdocLegal),
            ("/** jsdoc */", CommentContent::Jsdoc),
            ("/**/", CommentContent::None),
            ("/***/", CommentContent::None),
            ("/*@*/", CommentContent::None),
            ("/*@xreserve*/", CommentContent::None),
            ("/*@preserve*/", CommentContent::Legal),
            ("/*@voidzeroignoreme*/", CommentContent::None),
            ("/****/", CommentContent::None),
            ("/* @vite-ignore */", CommentContent::Vite),
            ("/* @vite-xxx */", CommentContent::Vite),
            ("/* webpackChunkName: 'my-chunk-name' */", CommentContent::Webpack),
            ("/* webpack */", CommentContent::None),
            ("/* @__PURE__ */", CommentContent::Pure),
            ("/* @__NO_SIDE_EFFECTS__ */", CommentContent::NoSideEffects),
            ("/* #__PURE__ */", CommentContent::Pure),
            ("/* #__NO_SIDE_EFFECTS__ */", CommentContent::NoSideEffects),
            ("/* turbopackOptional: true */", CommentContent::Turbopack),
        ];

        for (source_text, expected) in data {
            let comments = get_comments(source_text);
            assert_eq!(comments.len(), 1, "{source_text}");
            assert_eq!(comments[0].content, expected, "{source_text}");
        }
    }
}
