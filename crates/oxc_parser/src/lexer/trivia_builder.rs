use memchr::memchr_iter;
use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::{Comment, CommentContent, CommentKind, CommentPosition};
use oxc_span::Span;

use super::{Kind, Token};

#[derive(Debug)]
pub struct TriviaBuilder<'a> {
    // This is a set of unique comments. Duplicated
    // comments could be generated in case of rewind; they are
    // filtered out at insertion time.
    pub(crate) comments: ArenaVec<'a, Comment>,

    pub(crate) irregular_whitespaces: Vec<Span>,

    // states
    /// index of processed comments
    processed: usize,

    /// Start offset of the most recent regular line break outside comments,
    /// or `u32::MAX` if none seen since the previous token.
    /// Initialized to `0`: the start of the file counts as a preceding line break.
    last_newline_start: u32,

    /// Previous token kind, used to indicates comments are trailing from what kind
    previous_kind: Kind,

    /// Mirror of `self.processed < self.comments.len()`, kept as a bool so the
    /// per-token hot path tests a single byte.
    has_pending: bool,

    /// Whether the pending comments can still be finalized as trailing comments of the
    /// previous token: the run starts on the token's line, and no earlier line break has
    /// already declined the sweep (stay-leading comment, or a declined line comment).
    sweepable: bool,

    /// Index of the pure comment in `comments` vec, or `None` if no pure comment for the current token.
    pub(super) pure_comment: Option<usize>,

    pub(super) has_no_side_effects_comment: bool,

    /// Whether to classify comment contents into annotations.
    /// Set from [`ParseOptions::parse_comment_annotations`](crate::ParseOptions::parse_comment_annotations).
    pub(crate) parse_annotations: bool,
}

impl<'a> TriviaBuilder<'a> {
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            comments: ArenaVec::new_in(&allocator),
            irregular_whitespaces: vec![],
            processed: 0,
            last_newline_start: 0,
            previous_kind: Kind::Undetermined,
            has_pending: false,
            sweepable: false,
            pure_comment: None,
            has_no_side_effects_comment: false,
            parse_annotations: true,
        }
    }

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
        // The irregular whitespaces array is ordered; only add if not added before, to avoid
        // duplicates when the parser looks ahead (e.g. `peek_token`) and rewinds, then re-lexes the
        // same whitespace. Same approach as `add_comment`.
        if let Some(last) = self.irregular_whitespaces.last()
            && start <= last.start
        {
            return;
        }
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
    // newline after line comments. Line breaks inside multi-line comments, and irregular line
    // terminators, deliberately do not update this state (same as before this was a watermark).
    #[inline]
    pub fn handle_newline(&mut self, start: u32) {
        self.last_newline_start = start;
    }

    #[inline]
    pub fn handle_token(&mut self, token: Token) {
        // Cold path: resolve pending comments before this token. For files with no comments
        // (or once all comments are consumed) `has_pending` is false, so this branch is skipped.
        if self.has_pending {
            self.finish_pending(token.start());
        }
        self.previous_kind = token.kind();
        self.last_newline_start = u32::MAX;
    }

    /// Was a line break seen at or after `pos` (and since the previous token)?
    fn newline_since(&self, pos: u32) -> bool {
        self.last_newline_start != u32::MAX && self.last_newline_start >= pos
    }

    /// Apply the effects of any line break seen after the last pending comment — what
    /// `handle_newline` used to do eagerly at each line break: set its `followed_by_newline`
    /// flag, and either finalize the whole pending run as trailing comments of the previous
    /// token, or permanently decline the sweep for this run.
    fn resolve_pending(&mut self) {
        let len = self.comments.len();
        let end = self.comments[len - 1].span.end;
        if self.newline_since(end) {
            let comment = &mut self.comments[len - 1];
            comment.set_followed_by_newline(true);
            if self.sweepable && !Self::should_stay_leading(comment) {
                self.processed = len;
                self.has_pending = false;
            } else {
                self.sweepable = false;
            }
        }
    }

    #[cold]
    fn finish_pending(&mut self, attached_to: u32) {
        self.resolve_pending();
        if self.has_pending {
            self.attach_pending_leading_comments(attached_to, self.comments.len());
        }
    }

    #[cold]
    fn attach_pending_leading_comments(&mut self, attached_to: u32, len: usize) {
        for comment in &mut self.comments[self.processed..] {
            comment.position = CommentPosition::Leading;
            comment.attached_to = attached_to;
        }
        self.processed = len;
        self.has_pending = false;
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
    /// 2. It does not immediately follow an `=` [`Kind::Eq`], `(` [`Kind::LParen`]
    ///    or `:` [`Kind::Colon`] token.
    ///
    /// ```javascript
    /// let y = // This should not be treated as trailing (follows `=`)
    ///     10;
    ///
    /// function foo( // This should not be treated as trailing (follows `(`)
    ///     param
    /// ) {}
    ///
    /// let z = cond ? a : // This should not be treated as trailing (follows `:`)
    ///     b;
    /// ```
    ///
    /// Treating a comment after `:` as trailing drops it (it anchors to the
    /// previous token rather than the following operand), which breaks codegen
    /// idempotency once a transform emits `? consequent : // comment\nalternate`.
    fn should_be_treated_as_trailing_comment(&self) -> bool {
        self.last_newline_start == u32::MAX
            && !matches!(self.previous_kind, Kind::Eq | Kind::LParen | Kind::Colon)
    }

    fn should_stay_leading(comment: &Comment) -> bool {
        // Match esbuild's model where legal comments are preserved before the following token/statement.
        // Annotation comments (`@__PURE__`, `@__NO_SIDE_EFFECTS__`) semantically mark the *next*
        // token, so they must also stay leading even when no newline precedes them — otherwise
        // codegen's minified output (which smashes statements together) breaks idempotency:
        // pass 1 emits the verbatim annotation as leading, pass 2 re-parses it as trailing of the
        // previous `}`/`;` and loses the `attached_to`, falling back to the canonical literal.
        matches!(
            comment.content,
            CommentContent::Legal
                | CommentContent::JsdocLegal
                | CommentContent::Pure
                | CommentContent::PureNotApplied
                | CommentContent::NoSideEffects
        )
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
        if self.parse_annotations {
            Self::parse_annotation(&mut comment, source_text);
        }
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

        // Apply line-break effects on the previous pending comment first
        // (what `handle_newline` used to do eagerly at each line break).
        if self.has_pending {
            self.resolve_pending();
        }

        // This newly added comment may be preceded by a newline: a line break since the
        // previous comment, or since the previous token if there is no previous comment.
        // (Line comments record their own terminating line break below, and line breaks
        // inside multi-line comments deliberately don't count, as before.)
        let preceded = match self.comments.last() {
            Some(last) => self.newline_since(last.span.end),
            None => self.newline_since(0),
        };
        comment.set_preceded_by_newline(preceded);

        let mut line_declined = false;
        if comment.is_line() {
            // A line comment is always followed by its terminating line break.
            comment.set_followed_by_newline(true);
            if self.should_be_treated_as_trailing_comment() && !Self::should_stay_leading(&comment)
            {
                self.processed = self.comments.len() + 1; // +1 to include this comment.
            } else {
                line_declined = true;
            }
            // The line comment consumed its terminating line break.
            self.last_newline_start = comment.span.end;
        }

        // Set annotation flags here (not in `parse_annotation`) so the index is correct
        // even when the dedup check above skips a duplicate from parser lookahead/rewind.
        self.set_annotation_flags(&comment, self.comments.len());
        self.comments.push(comment);

        // Maintain the pending-run state.
        if self.processed < self.comments.len() {
            if !self.has_pending {
                // This comment starts a new pending run: it can only be swept as
                // trailing if it is on the same line as the previous token.
                self.sweepable = !preceded;
                self.has_pending = true;
            }
            if line_declined {
                // A declined line comment permanently blocks the sweep for this run.
                self.sweepable = false;
            }
        } else {
            self.has_pending = false;
        }
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
        assert!(ret.diagnostics.is_empty());
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
    fn legal_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();/**
 * @license MIT
 **/
function bar() {}";
        let comments = get_comments(source_text);
        let function_start = u32::try_from(source_text.find("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, function_start);
        assert!(comments[0].is_legal());
        assert!(comments[0].followed_by_newline());
    }

    #[test]
    fn legal_line_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();//! @license MIT\nfunction bar() {}";
        let comments = get_comments(source_text);
        let function_start = u32::try_from(source_text.find("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, function_start);
        assert!(comments[0].is_legal());
        assert!(comments[0].followed_by_newline());
    }

    // Annotation comments mark the *next* token, so they must stay leading even
    // when they sit directly after a previous statement with no preceding newline
    // (which is what codegen produces in `minify` mode). Without this, pass 2 of
    // an idempotency test would re-classify the annotation as trailing of the
    // previous token, drop its `attached_to`, and the codegen would fall back to
    // the canonical literal — diverging from pass 1's verbatim output.
    #[test]
    fn no_side_effects_block_comment_after_code_is_attached_to_next_token() {
        let source_text = "function foo() {}/* #__NO_SIDE_EFFECTS__ */\nfunction bar() {}";
        let comments = get_comments(source_text);
        let bar_start = u32::try_from(source_text.rfind("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, bar_start);
        assert!(comments[0].is_no_side_effects());
    }

    #[test]
    fn no_side_effects_line_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();// @__NO_SIDE_EFFECTS__\nfunction bar() {}";
        let comments = get_comments(source_text);
        let function_start = u32::try_from(source_text.find("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, function_start);
        assert!(comments[0].is_no_side_effects());
        assert!(comments[0].followed_by_newline());
    }

    #[test]
    fn pure_block_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();/* @__PURE__ */new Bar()";
        let comments = get_comments(source_text);
        let new_start = u32::try_from(source_text.find("new").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, new_start);
        assert!(comments[0].is_pure());
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
    fn leading_comments_after_colon() {
        // A line comment right after a conditional `:` anchors to the following
        // alternate (leading), not the previous token — otherwise it is dropped.
        let source_text = "v = cond ? a : // Leading comment\nb;";
        let comments = get_comments(source_text);
        let expected = vec![Comment {
            span: Span::new(15, 33),
            kind: CommentKind::Line,
            position: CommentPosition::Leading,
            attached_to: 34,
            newlines: CommentNewlines::Trailing,
            content: CommentContent::None,
        }];
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
    fn pure_comment_applied_on_member_chain() {
        // Rollup/esbuild treat PURE as applying to the innermost call/new even when
        // member access wraps it; member-access side effects are a separate concern.
        let cases = [
            "/*#__PURE__*/ test().a.b.c;",
            "/*#__PURE__*/ new Foo().a;",
            "/*#__PURE__*/ test()[0].b;",
            "class C { #bar; m() { /*#__PURE__*/ this.foo().#bar; } }",
            // Chain expressions with member root
            "/*#__PURE__*/ foo()?.a.b;",
            "/*#__PURE__*/ foo?.().a.b;",
            "/*#__PURE__*/ foo?.()[0];",
        ];
        for source_text in cases {
            let comments = get_comments(source_text);
            assert_eq!(comments[0].content, CommentContent::Pure, "{source_text}");
        }
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
