use std::cell::Cell;

use bitflags::bitflags;

use oxc_allocator::{Allocator, Box, CloneIn, CloneInSemanticIds, Dummy, Vec};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{ContentEq, GetSpan, Span};
use oxc_syntax::node::NodeId;

/// Indicates a line or block comment.
#[ast]
#[generate_derive(CloneIn, ContentEq, ESTree)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[estree(no_rename_variants, no_ts_def)]
pub enum CommentKind {
    /// Line comment
    #[default]
    Line = 0,
    /// Single-line comment
    #[estree(rename = "Block")]
    SingleLineBlock = 1,
    /// Multi-line block comment (contains line breaks)
    #[estree(rename = "Block")]
    MultiLineBlock = 2,
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

    /// `/* #__PURE__ */` that could not be applied (not before a call/new expression)
    /// <https://github.com/oxc-project/oxc/issues/20334>
    PureNotApplied = 5,

    /// `/* #__NO_SIDE_EFFECTS__ */`
    NoSideEffects = 6,

    /// `/* #__NO_SIDE_EFFECTS__ */` that could not be applied to a function
    NoSideEffectsNotApplied = 7,

    /// Webpack magic comment
    /// e.g. `/* webpackChunkName */`
    /// <https://webpack.js.org/api/module-methods/#magic-comments>
    Webpack = 8,

    /// Vite comment
    /// e.g. `/* @vite-ignore */`
    /// <https://github.com/search?q=repo%3Avitejs%2Fvite%20vite-ignore&type=code>
    Vite = 9,

    /// Code Coverage Ignore
    /// `v8 ignore`, `c8 ignore`, `node:coverage`, `istanbul ignore`
    /// <https://github.com/oxc-project/oxc/issues/10091>
    CoverageIgnore = 10,

    /// Turbopack magic comment
    /// e.g. `/* turbopackOptional: true */`
    /// <https://nextjs.org/docs/app/guides/lazy-loading#turbopackoptional-turbopack-only>
    Turbopack = 11,

    /// File-level code coverage ignore.
    ///
    /// `v8 ignore file`, `istanbul ignore file`.
    /// Classified separately because its meaning remains valid if the next AST
    /// node is removed, unlike position-sensitive coverage annotations.
    CoverageIgnoreFile = 12,

    /// Marks the following string or no-substitution template as a property name.
    /// `/* @__KEY__ */` or `/* #__KEY__ */`
    /// <https://esbuild.github.io/api/#mangle-key>
    PropertyKey = 13,
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

    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        _: &'alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

/// A comment in source code.
#[ast]
#[generate_derive(CloneIn, ContentEq, ESTree, GetSpan)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[estree(add_fields(value = CommentValue), no_ts_def, no_parent)]
pub struct Comment {
    /// The span of the comment text, with leading and trailing delimiters.
    pub span: Span,

    /// Source boundary this comment is attached to.
    ///
    /// Leading comments use the start of the following token:
    /// ```text
    /// /* Leading */ token
    ///               ^ attached_to
    /// ```
    ///
    /// Trailing comments use the end of the preceding token:
    /// ```text
    /// token| /* Trailing */
    ///      ^ attached_to (the boundary immediately after `token`)
    /// ```
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

/// Position of a source comment relative to its semantic AST host.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum AttachedCommentPosition {
    /// Print before the host node.
    #[default]
    Before = 0,
    /// Print after the host node.
    After = 1,
    /// Print inside a childless host's delimiters.
    Inside = 2,
}

/// A source comment assigned to an AST host by the semantic attachment pass.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct AttachedComment {
    pub comment: Comment,
    pub position: AttachedCommentPosition,
    pub same_line: bool,
    /// Whether ordinary node-boundary printing owns this comment.
    pub node_owned: bool,
    /// Whether the NodeId owner is exclusive, with no source-offset fallback.
    pub node_exclusive: bool,
}

/// A compact range of comments owned by one AST host.
#[derive(Debug, Default, Clone, Copy)]
pub struct CommentAttachmentHost {
    /// Semantic identity assigned before transforms mutate the AST.
    pub node_id: NodeId,
    /// Start of this host's range in the attachment comment buffer.
    pub start: u32,
    /// Number of comments in this host's range.
    pub len: u32,
}

/// Post-parse comment ownership sidecar.
#[derive(Debug)]
pub struct CommentAttachments<'a> {
    hosts: Vec<'a, Cell<CommentAttachmentHost>>,
    comments: Vec<'a, Cell<AttachedComment>>,
    host_len: Cell<u32>,
}

/// Optional arena-owned comment attachment table.
#[derive(Debug, Default)]
pub struct CommentAttachmentsStore<'a>(pub Option<Box<'a, CommentAttachments<'a>>>);

/// Dummy type communicating [`CommentAttachmentsStore`] to `oxc_ast_tools`.
#[ast(foreign = CommentAttachmentsStore)]
#[expect(dead_code)]
struct CommentAttachmentsStoreAlias<'a>(Option<Box<'a, u8>>);

impl<'a> Dummy<'a> for CommentAttachmentsStore<'a> {
    #[inline]
    fn dummy(_: &'a Allocator) -> Self {
        Self::default()
    }
}

impl<'a> CommentAttachments<'a> {
    #[inline]
    pub fn new_in(allocator: &'a Allocator, capacity: usize) -> Self {
        Self {
            hosts: Vec::from_iter_in(
                std::iter::repeat_with(Cell::default).take(capacity),
                &allocator,
            ),
            comments: Vec::from_iter_in(
                std::iter::repeat_with(Cell::default).take(capacity),
                &allocator,
            ),
            host_len: Cell::new(0),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.host_len.get() == 0
    }

    #[inline]
    pub fn host_len(&self) -> usize {
        self.host_len.get() as usize
    }

    #[inline]
    pub fn host(&self, index: usize) -> CommentAttachmentHost {
        self.hosts[index].get()
    }

    #[inline]
    pub fn comment(&self, index: usize) -> AttachedComment {
        self.comments[index].get()
    }

    #[inline]
    pub fn clear(&self) {
        self.host_len.set(0);
    }

    #[inline]
    pub fn set_comment(&self, index: usize, comment: AttachedComment) {
        self.comments[index].set(comment);
    }

    #[inline]
    /// # Panics
    ///
    /// Panics if the attachment pass produces more hosts than source comments.
    pub fn push_host(&self, host: CommentAttachmentHost) {
        let index = self.host_len();
        self.hosts[index].set(host);
        self.host_len.set(u32::try_from(index + 1).unwrap());
    }
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
            CommentKind::SingleLineBlock | CommentKind::MultiLineBlock => {
                Span::new(self.span.start + 2, self.span.end - 2)
            }
        }
    }

    /// Returns `true` if this is a line comment.
    #[inline]
    pub fn is_line(self) -> bool {
        self.kind == CommentKind::Line
    }

    /// Returns `true` if this is a block comment (either single-line or multi-line).
    #[inline]
    pub fn is_block(self) -> bool {
        matches!(self.kind, CommentKind::SingleLineBlock | CommentKind::MultiLineBlock)
    }

    /// Returns `true` if this is a multi-line block comment.
    #[inline]
    pub fn is_multiline_block(self) -> bool {
        self.kind == CommentKind::MultiLineBlock
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

    /// Is a leading `/* @__KEY__ */` or `/* #__KEY__ */` annotation.
    #[inline]
    pub fn is_property_key_annotation(self) -> bool {
        self.content == CommentContent::PropertyKey && self.is_leading()
    }

    /// Is webpack magic comment.
    #[inline]
    pub fn is_webpack(self) -> bool {
        self.content == CommentContent::Webpack
    }

    /// Is turbopack magic comment.
    #[inline]
    pub fn is_turbopack(self) -> bool {
        self.content == CommentContent::Turbopack
    }

    /// Is vite special comment.
    #[inline]
    pub fn is_vite(self) -> bool {
        self.content == CommentContent::Vite
    }

    /// Is coverage ignore comment.
    #[inline]
    pub fn is_coverage_ignore(self) -> bool {
        matches!(self.content, CommentContent::CoverageIgnore | CommentContent::CoverageIgnoreFile)
            && self.is_leading()
    }

    /// Is a file-level coverage ignore comment.
    #[inline]
    pub fn is_coverage_ignore_file(self) -> bool {
        self.content == CommentContent::CoverageIgnoreFile && self.is_leading()
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
