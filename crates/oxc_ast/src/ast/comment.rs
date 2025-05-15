#![warn(missing_docs)]
use oxc_allocator::CloneIn;
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{ContentEq, Span};

/// Indicates a line or block comment.
#[ast]
#[generate_derive(CloneIn, ContentEq, ESTree)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[estree(no_rename_variants, no_ts_def)]
pub enum CommentKind {
    /// Line comment
    #[default]
    Line = 0,
    /// Block comment
    Block = 1,
}

/// Information about a comment's position relative to a token.
#[ast]
#[generate_derive(CloneIn, ContentEq)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentPosition {
    /// Comments prior to a token until another token or trailing comment.
    ///
    /// e.g.
    ///
    /// ```ignore
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

/// Annotation comment that has special meaning.
#[ast]
#[generate_derive(CloneIn, ContentEq)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum CommentAnnotation {
    /// No Annotation
    #[default]
    None = 0,

    /// Legal Comment
    /// e.g. `/* @license */`, `/* @preserve */`, or starts with `//!` or `/*!`.
    /// <https://esbuild.github.io/api/#legal-comments>
    Legal = 1,

    /// `/** jsdoc */`
    /// <https://jsdoc.app>
    Jsdoc = 2,

    /// `/* #__PURE__ */`
    /// <https://github.com/javascript-compiler-hints/compiler-notations-spec>
    Pure = 3,

    /// `/* #__NO_SIDE_EFFECTS__ */`
    NoSideEffects = 4,

    /// Webpack magic comment
    /// e.g. `/* webpackChunkName */`
    /// <https://webpack.js.org/api/module-methods/#magic-comments>
    Webpack = 5,

    /// Vite comment
    /// e.g. `/* @vite-ignore */`
    /// <https://github.com/search?q=repo%3Avitejs%2Fvite%20vite-ignore&type=code>
    Vite = 6,

    /// Code Coverage Ignore
    /// `v8 ignore`, `c8 ignore`, `node:coverage`, `istanbul ignore`
    /// <https://github.com/oxc-project/oxc/issues/10091>
    CoverageIgnore = 7,
}

/// A comment in source code.
#[ast]
#[generate_derive(CloneIn, ContentEq, ESTree)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[estree(add_fields(value = CommentValue), field_order(kind, value, span), no_ts_def)]
pub struct Comment {
    /// The span of the comment text, with leading and trailing delimiters.
    pub span: Span,

    /// Start of token this leading comment is attached to.
    /// `/* Leading */ token`
    ///                ^ This start
    /// NOTE: Trailing comment attachment is not computed yet.
    #[estree(skip)]
    pub attached_to: u32,

    /// Line or block comment
    #[estree(rename = "type")]
    pub kind: CommentKind,

    /// Leading or trailing comment
    #[estree(skip)]
    pub position: CommentPosition,

    /// Whether this comment has a preceding newline.
    /// Used to avoid becoming a trailing comment in codegen.
    #[estree(skip)]
    pub preceded_by_newline: bool,

    /// Whether this comment has a tailing newline.
    #[estree(skip)]
    pub followed_by_newline: bool,

    /// Comment Annotation
    #[estree(skip)]
    pub annotation: CommentAnnotation,
}

impl Comment {
    /// Create a line or block comment at a given location.
    #[inline]
    pub fn new(start: u32, end: u32, kind: CommentKind) -> Self {
        let span = Span::new(start, end);
        Self {
            span,
            attached_to: 0,
            kind,
            position: CommentPosition::Trailing,
            preceded_by_newline: false,
            followed_by_newline: false,
            annotation: CommentAnnotation::None,
        }
    }

    /// Gets the span of the comment content.
    pub fn content_span(&self) -> Span {
        match self.kind {
            CommentKind::Line => Span::new(self.span.start + 2, self.span.end),
            CommentKind::Block => Span::new(self.span.start + 2, self.span.end - 2),
        }
    }

    /// Returns `true` if this is a line comment.
    #[inline]
    pub fn is_line(self) -> bool {
        self.kind == CommentKind::Line
    }

    /// Returns `true` if this is a block comment.
    #[inline]
    pub fn is_block(self) -> bool {
        self.kind == CommentKind::Block
    }

    /// Returns `true` if this comment is before a token.
    #[inline]
    pub fn is_leading(self) -> bool {
        self.position == CommentPosition::Leading
    }

    /// Returns `true` if this comment is after a token.
    #[inline]
    pub fn is_trailing(self) -> bool {
        self.position == CommentPosition::Trailing
    }

    /// Is comment with special meaning.
    #[inline]
    pub fn is_annotation(self) -> bool {
        self.annotation != CommentAnnotation::None
    }

    /// Returns `true` if this comment is a JSDoc comment. Implies `is_leading` and `is_block`.
    #[inline]
    pub fn is_jsdoc(self) -> bool {
        self.annotation == CommentAnnotation::Jsdoc && self.is_leading()
    }

    /// Legal comments
    ///
    /// A "legal comment" is considered to be any statement-level comment
    /// that contains `@license` or `@preserve` or that starts with `//!` or `/*!`.
    ///
    /// <https://esbuild.github.io/api/#legal-comments>
    #[inline]
    pub fn is_legal(self) -> bool {
        self.annotation == CommentAnnotation::Legal && self.is_leading()
    }

    /// Is `/* @__PURE__*/`.
    #[inline]
    pub fn is_pure(self) -> bool {
        self.annotation == CommentAnnotation::Pure
    }

    /// Is `/* @__NO_SIDE_EFFECTS__*/`.
    #[inline]
    pub fn is_no_side_effects(self) -> bool {
        self.annotation == CommentAnnotation::NoSideEffects
    }

    /// Is webpack magic comment.
    #[inline]
    pub fn is_webpack(self) -> bool {
        self.annotation == CommentAnnotation::Webpack
    }

    /// Is vite special comment.
    #[inline]
    pub fn is_vite(self) -> bool {
        self.annotation == CommentAnnotation::Vite
    }

    /// Is coverage ignore comment.
    #[inline]
    pub fn is_coverage_ignore(self) -> bool {
        self.annotation == CommentAnnotation::CoverageIgnore && self.is_leading()
    }
}
