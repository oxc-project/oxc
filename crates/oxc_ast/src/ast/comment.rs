use oxc_allocator::CloneIn;
use oxc_ast_macros::ast;
use oxc_span::{cmp::ContentEq, hash::ContentHash, Span};

#[ast]
#[generate_derive(CloneIn, ContentEq, ContentHash)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentKind {
    #[default]
    Line = 0,
    Block = 1,
}

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

    pub fn is_line(self) -> bool {
        self.kind == CommentKind::Line
    }

    pub fn is_block(self) -> bool {
        self.kind == CommentKind::Block
    }

    pub fn is_leading(self) -> bool {
        self.position == CommentPosition::Leading
    }

    pub fn is_trailing(self) -> bool {
        self.position == CommentPosition::Trailing
    }

    pub fn real_span(&self) -> Span {
        Span::new(self.real_span_start(), self.real_span_end())
    }

    pub fn real_span_end(&self) -> u32 {
        match self.kind {
            CommentKind::Line => self.span.end,
            // length of `*/`
            CommentKind::Block => self.span.end + 2,
        }
    }

    pub fn real_span_start(&self) -> u32 {
        self.span.start - 2
    }

    pub fn is_jsdoc(&self, source_text: &str) -> bool {
        self.is_leading() && self.is_block() && self.span.source_text(source_text).starts_with('*')
    }
}
