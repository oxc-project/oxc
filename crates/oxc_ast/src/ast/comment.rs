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
#[estree(via = CommentConverter, no_ts_def)]
pub struct Comment {
    /// The span of the comment text, with leading and trailing delimiters.
    pub span: Span,

    /// Start of token this leading comment is attached to.
    /// `/* Leading */ token`
    ///                ^ This start
    /// NOTE: Trailing comment attachment is not computed yet.
    #[estree(skip)]
    pub attached_to: u32,

    /// Flags for the comment, storing [`CommentKind`, `CommentPosition`, and `CommentAnnotation`]
    /// into a single field for memory efficiency. See [`Comment::kind`], [`Comment::annotation`],
    /// [`Comment::position`] to access the flags.
    pub flags: CommentFlags,
}

bitflags! {
    /// Stores CommentAnnotation, CommentPosition, and CommentKind into one for memory efficiency
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub struct CommentFlags: u16 {
        /// No flags set: no comment annotations, and implicitly assumed to be a leading line comment.
        const NONE = 0;
        /// The comment is a block comment.
        // Unless specified, comments are implicitly assumed to be line comments.
        // This saves a single bit in the flags field.
        const BLOCK = 1 << 1;
        /// The comment is a trailing comment.
        // Unless specified, comments are implicitly assumed to be leading comments.
        // This saves a single bit in the flags field.
        const TRAILING = 1 << 2;
        /// The comment has a newline before it.
        const PRECEDED_BY_NEW_LINE = 1 << 3;
        /// The comment has a newline after it.
        const FOLLOWED_BY_NEW_LINE = 1 << 4;
        /// See [`CommentAnnotation::Legal`]
        const LEGAL_ANNOTATION = 1 << 5;
        /// See [`CommentAnnotation::Jsdoc`]
        const JS_DOC_ANNOTATION = 1 << 6;
        /// See [`CommentAnnotation::Pure`]
        const PURE_ANNOTATION = 1 << 7;
        /// See [`CommentAnnotation::NoSideEffects`]
        const NO_SIDE_EFFECTS_ANNOTATION = 1 << 8;
        /// See [`CommentAnnotation::Webpack`]
        const WEBPACK_ANNOTATION = 1 << 9;
        /// See [`CommentAnnotation::Vite`]
        const VITE_ANNOTATION = 1 << 10;
        /// See [`CommentAnnotation::CoverageIgnore`]
        const COVERAGE_IGNORE_ANNOTATION = 1 << 11;
        /// Used to check if there are any annotations in the comment.
        const ANNOTATION = Self::LEGAL_ANNOTATION.bits()
            | Self::JS_DOC_ANNOTATION.bits()
            | Self::PURE_ANNOTATION.bits()
            | Self::NO_SIDE_EFFECTS_ANNOTATION.bits()
            | Self::WEBPACK_ANNOTATION.bits()
            | Self::VITE_ANNOTATION.bits()
            | Self::COVERAGE_IGNORE_ANNOTATION.bits();
    }
}

/// Dummy type to communicate the content of `CommentFlags` to `oxc_ast_tools`.
#[ast(foreign = CommentFlags)]
#[expect(dead_code)]
struct CommentFlagsAlias(u16);

impl ContentEq for CommentFlags {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'alloc> CloneIn<'alloc> for CommentFlags {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl Comment {
    /// Create a line or block comment at a given location.
    #[inline]
    pub fn new(start: u32, end: u32, kind: CommentKind) -> Self {
        let span = Span::new(start, end);
        let kind = match kind {
            CommentKind::Line => CommentFlags::NONE, // Line comments are the default
            CommentKind::Block => CommentFlags::BLOCK,
        };
        let flags = CommentFlags::TRAILING | kind;
        Self { span, attached_to: 0, flags }
    }

    /// Gets the span of the comment content.
    pub fn content_span(&self) -> Span {
        match self.kind() {
            CommentKind::Line => Span::new(self.span.start + 2, self.span.end),
            CommentKind::Block => Span::new(self.span.start + 2, self.span.end - 2),
        }
    }

    /// Returns `true` if this is a line comment.
    #[inline]
    pub fn is_line(self) -> bool {
        !self.is_block()
    }

    /// Returns `true` if this is a block comment.
    #[inline]
    pub fn is_block(self) -> bool {
        self.flags.contains(CommentFlags::BLOCK)
    }

    /// Returns `true` if this comment is before a token.
    #[inline]
    pub fn is_leading(self) -> bool {
        !self.is_trailing()
    }

    /// Returns `true` if this comment is after a token.
    #[inline]
    pub fn is_trailing(self) -> bool {
        self.flags.contains(CommentFlags::TRAILING)
    }

    /// Is comment with special meaning.
    #[inline]
    pub fn is_annotation(self) -> bool {
        self.flags.intersects(CommentFlags::ANNOTATION)
    }

    /// Returns `true` if this comment is a JSDoc comment. Implies `is_leading` and `is_block`.
    #[inline]
    pub fn is_jsdoc(self) -> bool {
        self.flags.contains(CommentFlags::JS_DOC_ANNOTATION) && self.is_leading()
    }

    /// Legal comments
    ///
    /// A "legal comment" is considered to be any statement-level comment
    /// that contains `@license` or `@preserve` or that starts with `//!` or `/*!`.
    ///
    /// <https://esbuild.github.io/api/#legal-comments>
    #[inline]
    pub fn is_legal(self) -> bool {
        self.flags.contains(CommentFlags::LEGAL_ANNOTATION) && self.is_leading()
    }

    /// Is `/* @__PURE__*/`.
    #[inline]
    pub fn is_pure(self) -> bool {
        self.flags.contains(CommentFlags::PURE_ANNOTATION)
    }

    /// Is `/* @__NO_SIDE_EFFECTS__*/`.
    #[inline]
    pub fn is_no_side_effects(self) -> bool {
        self.flags.contains(CommentFlags::NO_SIDE_EFFECTS_ANNOTATION)
    }

    /// Is webpack magic comment.
    #[inline]
    pub fn is_webpack(self) -> bool {
        self.flags.contains(CommentFlags::WEBPACK_ANNOTATION)
    }

    /// Is vite special comment.
    #[inline]
    pub fn is_vite(self) -> bool {
        self.flags.contains(CommentFlags::VITE_ANNOTATION)
    }

    /// Is coverage ignore comment.
    #[inline]
    pub fn is_coverage_ignore(self) -> bool {
        self.flags.contains(CommentFlags::COVERAGE_IGNORE_ANNOTATION) && self.is_leading()
    }

    /// Returns the [`CommentKind`] of this comment.
    #[inline]
    pub fn kind(self) -> CommentKind {
        if self.is_line() { CommentKind::Line } else { CommentKind::Block }
    }

    /// Returns the [`CommentAnnotation`] of this comment.
    #[inline]
    pub fn annotation(self) -> CommentAnnotation {
        if self.is_legal() {
            CommentAnnotation::Legal
        } else if self.is_jsdoc() {
            CommentAnnotation::Jsdoc
        } else if self.is_pure() {
            CommentAnnotation::Pure
        } else if self.is_no_side_effects() {
            CommentAnnotation::NoSideEffects
        } else if self.is_webpack() {
            CommentAnnotation::Webpack
        } else if self.is_vite() {
            CommentAnnotation::Vite
        } else if self.is_coverage_ignore() {
            CommentAnnotation::CoverageIgnore
        } else {
            CommentAnnotation::None
        }
    }

    /// Returns the [`CommentPosition`] of this comment.
    #[inline]
    pub fn position(self) -> CommentPosition {
        if self.is_trailing() { CommentPosition::Trailing } else { CommentPosition::Leading }
    }

    /// Returns `true` if this comment has a newline before it.
    #[inline]
    pub fn preceded_by_newline(self) -> bool {
        self.flags.contains(CommentFlags::PRECEDED_BY_NEW_LINE)
    }

    /// Returns `true` if this comment has a newline after it.
    #[inline]
    pub fn followed_by_newline(self) -> bool {
        self.flags.contains(CommentFlags::FOLLOWED_BY_NEW_LINE)
    }

    /// Sets the `followed_by_newline` flag.
    pub fn set_followed_by_newline(&mut self, followed_by_newline: bool) {
        if followed_by_newline {
            self.flags.insert(CommentFlags::FOLLOWED_BY_NEW_LINE);
        } else {
            self.flags.remove(CommentFlags::FOLLOWED_BY_NEW_LINE);
        }
    }

    /// Sets the `preceded_by_newline` flag.
    pub fn set_preceded_by_newline(&mut self, preceded_by_newline: bool) {
        if preceded_by_newline {
            self.flags.insert(CommentFlags::PRECEDED_BY_NEW_LINE);
        } else {
            self.flags.remove(CommentFlags::PRECEDED_BY_NEW_LINE);
        }
    }

    /// Sets the position of the comment.
    pub fn set_position(&mut self, position: CommentPosition) {
        match position {
            CommentPosition::Leading => self.flags.remove(CommentFlags::TRAILING),
            CommentPosition::Trailing => self.flags.insert(CommentFlags::TRAILING),
        }
    }

    /// Sets the annotation of the comment.
    pub fn set_annotation(&mut self, annotation: CommentAnnotation) {
        self.flags = match annotation {
            CommentAnnotation::None => self.flags & !CommentFlags::ANNOTATION,
            CommentAnnotation::Legal => self.flags | CommentFlags::LEGAL_ANNOTATION,
            CommentAnnotation::Jsdoc => self.flags | CommentFlags::JS_DOC_ANNOTATION,
            CommentAnnotation::Pure => self.flags | CommentFlags::PURE_ANNOTATION,
            CommentAnnotation::NoSideEffects => {
                self.flags | CommentFlags::NO_SIDE_EFFECTS_ANNOTATION
            }
            CommentAnnotation::Webpack => self.flags | CommentFlags::WEBPACK_ANNOTATION,
            CommentAnnotation::Vite => self.flags | CommentFlags::VITE_ANNOTATION,
            CommentAnnotation::CoverageIgnore => {
                self.flags | CommentFlags::COVERAGE_IGNORE_ANNOTATION
            }
        };
    }
}
