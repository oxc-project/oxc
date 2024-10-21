#![warn(missing_docs)]
use oxc_allocator::CloneIn;
use oxc_ast_macros::ast;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Span};

/// Indicates a line or block comment.
#[ast]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    /// Line comment
    #[default]
    Line = 0,
    /// Block comment
    Block = 1,
}

/// Information about a comment's position relative to a token.
#[ast]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentPosition {
    /// Comments prior to a token until another token or trailing comment.
    ///
    /// e.g.
    ///
    /// ```
    /// /* leading */ token;
    /// /* leading */
    /// // leading
    /// token;
    /// ```
    #[default]
    Leading = 0,

    /// Comments tailing a token until a newline.
    /// e.g. `token /* trailing */ // trailing`
    Trailing = 1,
}

/// A comment in source code.
#[ast]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Comment {
    /// The span of the comment text (without leading/trailing delimiters).
    pub span: Span,

    /// Line or block comment
    pub kind: CommentKind,

    /// Leading or trailing comment
    pub position: CommentPosition,

    /// Start of token this leading comment is attached to.
    /// `/* Leading */ token`
    ///                ^ This start
    /// NOTE: Trailing comment attachment is not computed yet.
    pub attached_to: u32,

    /// Whether this comment has a preceding newline.
    /// Used to avoid becoming a trailing comment in codegen.
    pub preceded_by_newline: bool,

    /// Whether this comment has a tailing newline.
    pub followed_by_newline: bool,
}

impl Comment {
    /// Create a line or block comment at a given location.
    #[inline]
    pub fn new(start: u32, end: u32, kind: CommentKind) -> Self {
        let span = Span::new(start, end);
        Self {
            span,
            kind,
            position: CommentPosition::Trailing,
            attached_to: 0,
            preceded_by_newline: false,
            followed_by_newline: false,
        }
    }

    /// Returns `true` if this is a line comment.
    pub fn is_line(self) -> bool {
        self.kind == CommentKind::Line
    }

    /// Returns `true` if this is a block comment.
    pub fn is_block(self) -> bool {
        self.kind == CommentKind::Block
    }

    /// Returns `true` if this comment is before a token.
    pub fn is_leading(self) -> bool {
        self.position == CommentPosition::Leading
    }

    /// Returns `true` if this comment is after a token.
    pub fn is_trailing(self) -> bool {
        self.position == CommentPosition::Trailing
    }

    #[allow(missing_docs)]
    pub fn real_span(&self) -> Span {
        Span::new(self.real_span_start(), self.real_span_end())
    }

    #[allow(missing_docs)]
    pub fn real_span_end(&self) -> u32 {
        match self.kind {
            CommentKind::Line => self.span.end,
            // length of `*/`
            CommentKind::Block => self.span.end + 2,
        }
    }

    #[allow(missing_docs)]
    pub fn real_span_start(&self) -> u32 {
        self.span.start - 2
    }

    /// Returns `true` if this comment is a JSDoc comment. Implies `is_leading`
    /// and `is_block`.
    pub fn is_jsdoc(&self, source_text: &str) -> bool {
        self.is_leading() && self.is_block() && self.span.source_text(source_text).starts_with('*')
    }
}
