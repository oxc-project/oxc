#![warn(missing_docs)]
use bitflags::bitflags;

use oxc_allocator::{Allocator, CloneIn};
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
    /// Singleline comment
    #[estree(rename = "Block")]
    SinglelineBlock = 1,
    /// Multiline block comment (contains line breaks)
    #[estree(rename = "Block")]
    MultilineBlock = 2,
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
pub enum CommentContent {
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

    /// A jsdoc containing legal annotation.
    /// `/** @preserve */`
    JsdocLegal = 3,

    /// `/* #__PURE__ */`
    /// <https://github.com/javascript-compiler-hints/compiler-notations-spec>
    Pure = 4,

    /// `/* #__NO_SIDE_EFFECTS__ */`
    NoSideEffects = 5,

    /// Webpack magic comment
    /// e.g. `/* webpackChunkName */`
    /// <https://webpack.js.org/api/module-methods/#magic-comments>
    Webpack = 6,

    /// Vite comment
    /// e.g. `/* @vite-ignore */`
    /// <https://github.com/search?q=repo%3Avitejs%2Fvite%20vite-ignore&type=code>
    Vite = 7,

    /// Code Coverage Ignore
    /// `v8 ignore`, `c8 ignore`, `node:coverage`, `istanbul ignore`
    /// <https://github.com/oxc-project/oxc/issues/10091>
    CoverageIgnore = 8,
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
    /// State of newlines around a comment.
    pub struct CommentNewlines: u8 {
        /// Preceded by a newline
        const Leading = 1 << 0;
        /// Followed by a newline
        const Trailing = 1 << 1;
        /// No newlines before or after
        const None = 0;
    }
}

/// Dummy type to communicate the content of `CommentFlags` to `oxc_ast_tools`.
#[ast(foreign = CommentNewlines)]
#[expect(dead_code)]
struct CommentNewlinesAlias(u8);

impl ContentEq for CommentNewlines {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'alloc> CloneIn<'alloc> for CommentNewlines {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        *self
    }
}

/// A comment in source code.
#[ast]
#[generate_derive(CloneIn, ContentEq, ESTree)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[estree(add_fields(value = CommentValue), no_ts_def, no_parent)]
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

    /// Whether this comment has newlines around it.
    /// Used to avoid becoming a trailing comment in codegen.
    #[estree(skip)]
    pub newlines: CommentNewlines,

    /// Content of the comment
    #[estree(skip)]
    pub content: CommentContent,
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
            newlines: CommentNewlines::None,
            content: CommentContent::None,
        }
    }

    /// Gets the span of the comment content.
    pub fn content_span(&self) -> Span {
        match self.kind {
            CommentKind::Line => Span::new(self.span.start + 2, self.span.end),
            CommentKind::SinglelineBlock | CommentKind::MultilineBlock => {
                Span::new(self.span.start + 2, self.span.end - 2)
            }
        }
    }

    /// Returns `true` if this is a line comment.
    #[inline]
    pub fn is_line(self) -> bool {
        self.kind == CommentKind::Line
    }

    /// Returns `true` if this is a singleline or multiline block comment.
    #[inline]
    pub fn is_block(self) -> bool {
        matches!(self.kind, CommentKind::SinglelineBlock | CommentKind::MultilineBlock)
    }

    /// Returns `true` if this is a multi-line block comment.
    #[inline]
    pub fn is_multiline_block(self) -> bool {
        self.kind == CommentKind::MultilineBlock
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

    /// Is comment without a special meaning.
    #[inline]
    pub fn is_normal(self) -> bool {
        self.content == CommentContent::None
    }

    /// Is comment with special meaning.
    #[inline]
    pub fn is_annotation(self) -> bool {
        self.content != CommentContent::None
            && self.content != CommentContent::Legal
            && self.content != CommentContent::Jsdoc
            && self.content != CommentContent::JsdocLegal
    }

    /// Returns `true` if this comment is a JSDoc comment. Implies `is_leading` and `is_block`.
    #[inline]
    pub fn is_jsdoc(self) -> bool {
        matches!(self.content, CommentContent::Jsdoc | CommentContent::JsdocLegal)
            && self.is_leading()
    }

    /// Legal comments
    ///
    /// A "legal comment" is considered to be any statement-level comment
    /// that contains `@license` or `@preserve` or that starts with `//!` or `/*!`.
    ///
    /// <https://esbuild.github.io/api/#legal-comments>
    #[inline]
    pub fn is_legal(self) -> bool {
        matches!(self.content, CommentContent::Legal | CommentContent::JsdocLegal)
            && self.is_leading()
    }

    /// Is `/* @__PURE__*/`.
    #[inline]
    pub fn is_pure(self) -> bool {
        self.content == CommentContent::Pure
    }

    /// Is `/* @__NO_SIDE_EFFECTS__*/`.
    #[inline]
    pub fn is_no_side_effects(self) -> bool {
        self.content == CommentContent::NoSideEffects
    }

    /// Is webpack magic comment.
    #[inline]
    pub fn is_webpack(self) -> bool {
        self.content == CommentContent::Webpack
    }

    /// Is vite special comment.
    #[inline]
    pub fn is_vite(self) -> bool {
        self.content == CommentContent::Vite
    }

    /// Is coverage ignore comment.
    #[inline]
    pub fn is_coverage_ignore(self) -> bool {
        self.content == CommentContent::CoverageIgnore && self.is_leading()
    }

    /// Returns `true` if this comment is preceded by a newline.
    #[inline]
    pub fn preceded_by_newline(self) -> bool {
        self.newlines.contains(CommentNewlines::Leading)
    }

    /// Returns `true` if this comment is followed by a newline.
    #[inline]
    pub fn followed_by_newline(self) -> bool {
        self.newlines.contains(CommentNewlines::Trailing)
    }

    /// Returns `true` if this comment has newlines either before or after it.
    #[inline]
    pub fn has_newlines_around(self) -> bool {
        self.newlines != CommentNewlines::None
    }

    /// Sets the state of `newlines` to include/exclude a newline before the comment.
    #[inline]
    pub fn set_preceded_by_newline(&mut self, preceded_by_newline: bool) {
        self.newlines.set(CommentNewlines::Leading, preceded_by_newline);
    }

    /// Sets the state of `newlines` to include/exclude a newline after the comment.
    #[inline]
    pub fn set_followed_by_newline(&mut self, followed_by_newline: bool) {
        self.newlines.set(CommentNewlines::Trailing, followed_by_newline);
    }
}
