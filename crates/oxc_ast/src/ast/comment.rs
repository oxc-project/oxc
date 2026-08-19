use std::{
    cell::RefCell,
    ops::{Deref, Index, IndexMut},
    ptr::NonNull,
    slice::SliceIndex,
    sync::atomic::AtomicUsize,
};

use bitflags::bitflags;

use oxc_allocator::{
    Allocator, Box as ArenaBox, CloneIn, CloneInSemanticIds, Dummy, HashMap as ArenaHashMap, Vec,
};
use oxc_ast_macros::ast;
use oxc_estree::ESTree;
use oxc_span::{ContentEq, GetSpan, Span};
use oxc_syntax::node::NodeId;

/// Index of a source-order comment in a [`CommentStore`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommentId(u32);

impl CommentId {
    #[inline]
    pub fn from_usize(index: usize) -> Self {
        Self(u32::try_from(index).expect("comment index exceeds u32::MAX"))
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CommentId {
    type Cloned = Self;

    #[inline]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        _allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}

/// Comments attached to one AST node.
#[derive(Debug)]
pub struct NodeComments<'a> {
    pub leading: Vec<'a, CommentId>,
    pub trailing: Vec<'a, CommentId>,
    pub dangling: Vec<'a, CommentId>,
}

impl<'a> NodeComments<'a> {
    #[inline]
    fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            leading: Vec::new_in(&allocator),
            trailing: Vec::new_in(&allocator),
            dangling: Vec::new_in(&allocator),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.leading.is_empty() && self.trailing.is_empty() && self.dangling.is_empty()
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for NodeComments<'_> {
    type Cloned = NodeComments<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        NodeComments {
            leading: self.leading.clone_in_impl(with_semantic_ids, allocator),
            trailing: self.trailing.clone_in_impl(with_semantic_ids, allocator),
            dangling: self.dangling.clone_in_impl(with_semantic_ids, allocator),
        }
    }
}

/// Source-order comments plus their optional AST-node ownership.
///
/// Comments remain stored exactly once in source order. Node attachment lists contain only
/// [`CommentId`]s, so rekeying node ownership never moves or copies comment payloads.
pub struct CommentStore<'a> {
    comments: Vec<'a, Comment>,
    // Type-erased to keep `Program<'a>` covariant. `ArenaHashMap` itself is invariant in `'a`.
    attachments: NonNull<()>,
    allocator: NonNull<Allocator>,
    suppressed: NonNull<()>,
}

impl std::fmt::Debug for CommentStore<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommentStore")
            .field("comments", &self.comments)
            .field("attachments", &self.attachments_cell())
            .field("suppressed", &self.suppressed_cell())
            .finish()
    }
}

impl<'a> CommentStore<'a> {
    #[inline]
    pub fn new_in(allocator: &'a Allocator) -> Self {
        let attachments = ArenaBox::new_in(
            RefCell::new(ArenaHashMap::<NodeId, NodeComments<'a>>::new_in(allocator)),
            &allocator,
        );
        let suppressed =
            ArenaBox::new_in(RefCell::new(Vec::<CommentId>::new_in(&allocator)), &allocator);
        Self {
            comments: Vec::new_in(&allocator),
            attachments: ArenaBox::into_non_null(attachments).cast(),
            allocator: NonNull::from(allocator),
            suppressed: ArenaBox::into_non_null(suppressed).cast(),
        }
    }

    #[inline]
    pub fn from_vec(comments: Vec<'a, Comment>, allocator: &'a Allocator) -> Self {
        let attachments = ArenaBox::new_in(
            RefCell::new(ArenaHashMap::<NodeId, NodeComments<'a>>::new_in(allocator)),
            &allocator,
        );
        let suppressed =
            ArenaBox::new_in(RefCell::new(Vec::<CommentId>::new_in(&allocator)), &allocator);
        Self {
            comments,
            attachments: ArenaBox::into_non_null(attachments).cast(),
            allocator: NonNull::from(allocator),
            suppressed: ArenaBox::into_non_null(suppressed).cast(),
        }
    }

    #[inline]
    pub fn from_iter_in(iter: impl IntoIterator<Item = Comment>, allocator: &'a Allocator) -> Self {
        Self::from_vec(Vec::from_iter_in(iter, &allocator), allocator)
    }

    #[inline]
    pub fn as_slice(&self) -> &[Comment] {
        &self.comments
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Comment] {
        &mut self.comments
    }

    #[inline]
    fn allocator(&self) -> &'a Allocator {
        // SAFETY: The pointer comes from the allocator used to create this `CommentStore`, and the
        // store cannot outlive `'a` because its comment vector carries that lifetime.
        unsafe { self.allocator.as_ref() }
    }

    #[inline]
    fn attachments_cell(&self) -> &RefCell<ArenaHashMap<'a, NodeId, NodeComments<'a>>> {
        // SAFETY: `attachments` is an arena allocation of exactly this type, erased solely to keep
        // the surrounding AST lifetime covariant.
        unsafe { self.attachments.cast().as_ref() }
    }

    #[inline]
    fn attachments_cell_mut(&mut self) -> &mut RefCell<ArenaHashMap<'a, NodeId, NodeComments<'a>>> {
        // SAFETY: As above; `&mut self` guarantees exclusive access to the pointer.
        unsafe { self.attachments.cast().as_mut() }
    }

    #[inline]
    fn suppressed_cell(&self) -> &RefCell<Vec<'a, CommentId>> {
        // SAFETY: Same invariant as `attachments_cell`.
        unsafe { self.suppressed.cast().as_ref() }
    }

    #[inline]
    fn suppressed_cell_mut(&mut self) -> &mut RefCell<Vec<'a, CommentId>> {
        // SAFETY: Same invariant as `attachments_cell_mut`.
        unsafe { self.suppressed.cast().as_mut() }
    }

    #[inline]
    pub fn push(&mut self, comment: Comment) {
        self.comments.push(comment);
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut Comment> {
        self.comments.last_mut()
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Comment> {
        self.comments.get_mut(index)
    }

    #[inline]
    pub fn attachments(&self) -> std::cell::Ref<'_, ArenaHashMap<'a, NodeId, NodeComments<'a>>> {
        self.attachments_cell().borrow()
    }

    #[inline]
    pub fn attachments_mut(
        &self,
    ) -> std::cell::RefMut<'_, ArenaHashMap<'a, NodeId, NodeComments<'a>>> {
        self.attachments_cell().borrow_mut()
    }

    #[inline]
    pub fn is_suppressed(&self, comment_id: CommentId) -> bool {
        self.suppressed_cell().borrow().contains(&comment_id)
    }

    /// Drop ordinary comments owned by a removed node while leaving semantic orphans available
    /// to positional rescue.
    pub fn suppress_removed_node_comments(&self, node_comments: &NodeComments<'a>) {
        for comment_id in node_comments
            .leading
            .iter()
            .chain(&node_comments.trailing)
            .chain(&node_comments.dangling)
            .copied()
        {
            self.suppress_comment_if_ordinary(comment_id);
        }
    }

    pub fn suppress_comment_if_ordinary(&self, comment_id: CommentId) {
        let comment = self.comments[comment_id.index()];
        let mut suppressed = self.suppressed_cell().borrow_mut();
        if !comment.is_legal()
            && !comment.is_coverage_ignore_file()
            && !suppressed.contains(&comment_id)
        {
            suppressed.push(comment_id);
        }
    }

    /// Attach source-order comments to an AST node.
    pub fn attach(
        &self,
        node_id: NodeId,
        position: CommentPosition,
        comment_ids: impl IntoIterator<Item = CommentId>,
    ) {
        let mut comment_ids = comment_ids.into_iter();
        let Some(first) = comment_ids.next() else { return };
        debug_assert_ne!(node_id, NodeId::DUMMY);
        let mut attachments = self.attachments_cell().borrow_mut();
        let node_comments =
            attachments.entry(node_id).or_insert_with(|| NodeComments::new_in(self.allocator()));
        let target = match position {
            CommentPosition::Leading => &mut node_comments.leading,
            CommentPosition::Trailing => &mut node_comments.trailing,
        };
        target.push(first);
        target.extend(comment_ids);
    }

    /// Attach comments which belong inside a node rather than before or after it.
    pub fn attach_dangling(
        &self,
        node_id: NodeId,
        comment_ids: impl IntoIterator<Item = CommentId>,
    ) {
        let mut comment_ids = comment_ids.into_iter();
        let Some(first) = comment_ids.next() else { return };
        debug_assert_ne!(node_id, NodeId::DUMMY);
        let mut attachments = self.attachments_cell().borrow_mut();
        let node_comments =
            attachments.entry(node_id).or_insert_with(|| NodeComments::new_in(self.allocator()));
        node_comments.dangling.push(first);
        node_comments.dangling.extend(comment_ids);
    }

    /// Move all comment ownership from one node ID to another.
    pub fn rekey_node(&self, old_node_id: NodeId, new_node_id: NodeId) {
        if old_node_id == new_node_id {
            return;
        }
        let mut attachments = self.attachments_cell().borrow_mut();
        let Some(old) = attachments.remove(&old_node_id) else { return };
        let new = attachments
            .entry(new_node_id)
            .or_insert_with(|| NodeComments::new_in(self.allocator()));
        new.leading.extend(old.leading);
        new.trailing.extend(old.trailing);
        new.dangling.extend(old.dangling);
    }

    #[inline]
    pub fn node_comments(&self, node_id: NodeId) -> Option<std::cell::Ref<'_, NodeComments<'a>>> {
        let attachments = self.attachments_cell().borrow();
        if !attachments.contains_key(&node_id) {
            return None;
        }
        Some(std::cell::Ref::map(attachments, |attachments| attachments.get(&node_id).unwrap()))
    }

    /// Merge a complete node attachment record into the store.
    pub fn attach_node_comments(&self, node_id: NodeId, comments: NodeComments<'a>) {
        if comments.is_empty() {
            return;
        }
        let mut attachments = self.attachments_cell().borrow_mut();
        let target =
            attachments.entry(node_id).or_insert_with(|| NodeComments::new_in(self.allocator()));
        target.leading.extend(comments.leading);
        target.trailing.extend(comments.trailing);
        target.dangling.extend(comments.dangling);
    }

    #[inline]
    pub fn into_vec(self) -> Vec<'a, Comment> {
        self.comments
    }

    #[inline]
    pub fn into_arena_slice(self) -> &'a [Comment] {
        self.comments.into_arena_slice()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.comments.clear();
        self.attachments_cell_mut().get_mut().clear();
        self.suppressed_cell_mut().get_mut().clear();
    }

    /// Insert a comment at its source-order position and update attachment indexes.
    pub fn insert(&mut self, index: usize, comment: Comment) {
        self.comments.insert(index, comment);
        for node_comments in self.attachments_cell_mut().get_mut().values_mut() {
            shift_comment_ids_after_insert(&mut node_comments.leading, index);
            shift_comment_ids_after_insert(&mut node_comments.trailing, index);
            shift_comment_ids_after_insert(&mut node_comments.dangling, index);
        }
        shift_comment_ids_after_insert(self.suppressed_cell_mut().get_mut(), index);
    }

    /// Retain comments and reindex all attachment lists.
    pub fn retain(&mut self, mut keep: impl FnMut(&Comment) -> bool) {
        let retained: std::vec::Vec<bool> = self.comments.iter().map(&mut keep).collect();
        let mut remap = vec![None; self.comments.len()];
        let mut next_index = 0usize;
        for (index, &keep) in retained.iter().enumerate() {
            if keep {
                remap[index] = Some(CommentId::from_usize(next_index));
                next_index += 1;
            }
        }
        let mut index = 0usize;
        self.comments.retain(|_| {
            let keep = retained[index];
            index += 1;
            keep
        });
        let attachments = self.attachments_cell_mut().get_mut();
        for node_comments in attachments.values_mut() {
            remap_comment_ids(&mut node_comments.leading, &remap);
            remap_comment_ids(&mut node_comments.trailing, &remap);
            remap_comment_ids(&mut node_comments.dangling, &remap);
        }
        attachments.retain(|_, comments| !comments.is_empty());
        remap_comment_ids(self.suppressed_cell_mut().get_mut(), &remap);
    }
}

fn remap_comment_ids(ids: &mut Vec<'_, CommentId>, remap: &[Option<CommentId>]) {
    ids.retain_mut(|id| {
        let Some(new_id) = remap[id.index()] else { return false };
        *id = new_id;
        true
    });
}

fn shift_comment_ids_after_insert(ids: &mut Vec<'_, CommentId>, inserted: usize) {
    for id in ids {
        if id.index() >= inserted {
            *id = CommentId::from_usize(id.index() + 1);
        }
    }
}

impl Deref for CommentStore<'_> {
    type Target = [Comment];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'s, 'a> IntoIterator for &'s CommentStore<'a> {
    type Item = &'s Comment;
    type IntoIter = std::slice::Iter<'s, Comment>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.comments.iter()
    }
}

impl<'s, 'a> IntoIterator for &'s mut CommentStore<'a> {
    type Item = &'s mut Comment;
    type IntoIter = std::slice::IterMut<'s, Comment>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.comments.iter_mut()
    }
}

impl<I> Index<I> for CommentStore<'_>
where
    I: SliceIndex<[Comment]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.comments[index]
    }
}

impl<I> IndexMut<I> for CommentStore<'_>
where
    I: SliceIndex<[Comment]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.comments[index]
    }
}

impl<'a> Dummy<'a> for CommentStore<'a> {
    #[inline]
    fn dummy(allocator: &'a Allocator) -> Self {
        Self::new_in(allocator)
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for CommentStore<'_> {
    type Cloned = CommentStore<'new_alloc>;

    fn clone_in_impl(
        &self,
        with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        let comments = self.comments.clone_in_impl(with_semantic_ids, allocator);
        let suppressed =
            self.suppressed_cell().borrow().clone_in_impl(with_semantic_ids, allocator);
        let attachments = if with_semantic_ids == CloneInSemanticIds::With {
            self.attachments_cell().borrow().clone_in_impl(with_semantic_ids, allocator)
        } else {
            ArenaHashMap::new_in(allocator)
        };
        let attachments = ArenaBox::new_in(RefCell::new(attachments), &allocator);
        let suppressed = ArenaBox::new_in(RefCell::new(suppressed), &allocator);
        CommentStore {
            comments,
            attachments: ArenaBox::into_non_null(attachments).cast(),
            allocator: NonNull::from(allocator),
            suppressed: ArenaBox::into_non_null(suppressed).cast(),
        }
    }
}

/// Dummy schema declaration for [`CommentStore`], whose internals are deliberately not AST data.
#[ast(foreign = CommentStore)]
#[expect(dead_code)]
struct CommentStoreAlias<'a> {
    comments: Vec<'a, Comment>,
    attachments: AtomicUsize,
    allocator: AtomicUsize,
    suppressed: AtomicUsize,
}

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

    /// Comments attached to the end of the preceding token on the same line.
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

    /// Webpack magic comment
    /// e.g. `/* webpackChunkName */`
    /// <https://webpack.js.org/api/module-methods/#magic-comments>
    Webpack = 7,

    /// Vite comment
    /// e.g. `/* @vite-ignore */`
    /// <https://github.com/search?q=repo%3Avitejs%2Fvite%20vite-ignore&type=code>
    Vite = 8,

    /// Code Coverage Ignore
    /// `v8 ignore`, `c8 ignore`, `node:coverage`, `istanbul ignore`
    /// <https://github.com/oxc-project/oxc/issues/10091>
    CoverageIgnore = 9,

    /// Turbopack magic comment
    /// e.g. `/* turbopackOptional: true */`
    /// <https://nextjs.org/docs/app/guides/lazy-loading#turbopackoptional-turbopack-only>
    Turbopack = 10,

    /// File-level code coverage ignore.
    ///
    /// `v8 ignore file`, `istanbul ignore file`.
    /// Classified separately because its meaning remains valid if the next AST
    /// node is removed, unlike position-sensitive coverage annotations.
    CoverageIgnoreFile = 11,

    /// Marks the following string or no-substitution template as a property name.
    /// `/* @__KEY__ */` or `/* #__KEY__ */`
    /// <https://esbuild.github.io/api/#mangle-key>
    PropertyKey = 12,
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
    /// `/* Leading */ token`
    ///                ^ This start
    ///
    /// Trailing comments use the end of the preceding token:
    /// `token /* Trailing */`
    ///       ^ This end
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutations_keep_attachment_ids_in_sync() {
        let allocator = Allocator::default();
        let mut comments = CommentStore::from_iter_in(
            [
                Comment::new(1, 2, CommentKind::Line),
                Comment::new(3, 4, CommentKind::Line),
                Comment::new(5, 6, CommentKind::Line),
            ],
            &allocator,
        );
        let node_id = NodeId::new(1);
        comments.attach(
            node_id,
            CommentPosition::Leading,
            [CommentId::from_usize(0), CommentId::from_usize(2)],
        );

        comments.insert(1, Comment::new(2, 3, CommentKind::Line));
        assert_eq!(
            comments.node_comments(node_id).unwrap().leading.as_slice(),
            &[CommentId::from_usize(0), CommentId::from_usize(3)]
        );

        comments.retain(|comment| comment.span.start != 1 && comment.span.start != 3);
        assert_eq!(
            comments.node_comments(node_id).unwrap().leading.as_slice(),
            &[CommentId::from_usize(1)]
        );
    }
}
