use std::borrow::Cow;

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use oxc_ast::{
    AttachedCommentPosition, Comment, CommentContent, CommentKind, GetNodeId,
    ast::{Expression, Program},
};
use oxc_span::GetSpan;
use oxc_syntax::{line_terminator::LineTerminatorSplitter, node::NodeId};

use crate::{Codegen, LegalComment, options::CommentOptions};

type CommentList = SmallVec<[Comment; 1]>;

#[derive(Default)]
pub struct NodeCommentStore {
    hosts: Vec<(NodeId, NodeComments)>,
    presence: Box<[u64]>,
    owners: FxHashMap<u32, NodeId>,
    exclusive: Vec<u32>,
}

#[derive(Default)]
pub struct NodeComments {
    before: CommentList,
    inside: CommentList,
    after: CommentList,
}

pub struct BoundaryComments {
    pub node: Option<NodeComments>,
    pub leading: bool,
    pub trailing: bool,
}

impl NodeCommentStore {
    fn build(program: &Program<'_>, mut retain: impl FnMut(Comment) -> bool) -> Option<Self> {
        let attachments = program.comment_attachments.0.as_deref()?;
        if attachments.is_empty() {
            return None;
        }

        let mut hosts = Vec::new();
        let mut owners = FxHashMap::default();
        let mut exclusive = Vec::new();
        for host_index in 0..attachments.host_len() {
            let host = attachments.host(host_index);
            let node_id = host.node_id;
            let start = host.start as usize;
            let end = start + host.len as usize;
            let mut comments = NodeComments::default();
            for attached_index in start..end {
                let attached = attachments.comment(attached_index);
                let comment = attached.comment;
                if !attached.node_owned || !retain(comment) {
                    continue;
                }
                owners.insert(comment.span.start, node_id);
                if attached.node_exclusive {
                    exclusive.push(comment.span.start);
                }
                match attached.position {
                    AttachedCommentPosition::Before => comments.before.push(comment),
                    AttachedCommentPosition::After => comments.after.push(comment),
                    AttachedCommentPosition::Inside => comments.inside.push(comment),
                }
            }
            if !comments.before.is_empty()
                || !comments.inside.is_empty()
                || !comments.after.is_empty()
            {
                hosts.push((node_id, comments));
            }
        }

        hosts.sort_unstable_by_key(|(node_id, _)| node_id.index());
        let mut presence = hosts.last().map_or_else(Box::default, |(node_id, _)| {
            vec![0; (node_id.index() >> 6) + 1].into_boxed_slice()
        });
        for (node_id, _) in &hosts {
            presence[node_id.index() >> 6] |= 1 << (node_id.index() & 63);
        }
        exclusive.sort_unstable();
        Some(Self { hosts, presence, owners, exclusive })
    }

    #[inline]
    fn take_all(&mut self, node_id: NodeId) -> Option<NodeComments> {
        let index = node_id.index();
        let word = self.presence.get_mut(index >> 6)?;
        let mask = 1 << (index & 63);
        if *word & mask == 0 {
            return None;
        }
        *word &= !mask;
        let host_index = self.hosts.binary_search_by_key(&index, |(id, _)| id.index()).unwrap();
        Some(std::mem::take(&mut self.hosts[host_index].1))
    }

    fn remove_comments(&mut self, removed: &[Comment]) {
        for comment in removed {
            let Some(node_id) = self.owners.remove(&comment.span.start) else { continue };
            let node_index = node_id.index();
            let Some(word) = self.presence.get_mut(node_index >> 6) else { continue };
            let mask = 1 << (node_index & 63);
            if *word & mask == 0 {
                continue;
            }
            let host_index =
                self.hosts.binary_search_by_key(&node_index, |(id, _)| id.index()).unwrap();
            let node_comments = &mut self.hosts[host_index].1;
            let mut removed_count = 0;
            for comments in
                [&mut node_comments.before, &mut node_comments.inside, &mut node_comments.after]
            {
                let before = comments.len();
                comments.retain(|candidate| candidate.span != comment.span);
                removed_count += before - comments.len();
            }
            if removed_count != 0
                && node_comments.before.is_empty()
                && node_comments.inside.is_empty()
                && node_comments.after.is_empty()
            {
                *word &= !mask;
            }
        }
    }
}

#[derive(Default)]
pub struct CommentStore {
    groups: Vec<CommentGroup>,
    anchor_presence: Box<[u64]>,
    orphan_indices: Box<[usize]>,
    remaining: usize,
}

struct CommentGroup {
    anchor: u32,
    leading: CommentList,
    trailing: CommentList,
}

impl CommentStore {
    fn build(comments: &mut Vec<Comment>) -> Self {
        let remaining = comments.len();
        if comments.windows(2).any(|comments| {
            let [left, right] = comments else { unreachable!() };
            (left.attached_to, left.span.start) > (right.attached_to, right.span.start)
        }) {
            comments.sort_unstable_by_key(|comment| (comment.attached_to, comment.span.start));
        }
        let mut groups = Vec::<CommentGroup>::new();
        let mut anchor_presence = if let Some(last) = comments.last() {
            vec![0; (last.attached_to as usize >> 6) + 1].into_boxed_slice()
        } else {
            Box::default()
        };
        for comment in comments.drain(..) {
            if groups.last().is_none_or(|group| group.anchor != comment.attached_to) {
                let anchor = comment.attached_to as usize;
                anchor_presence[anchor >> 6] |= 1 << (anchor & 63);
                groups.push(CommentGroup {
                    anchor: comment.attached_to,
                    leading: CommentList::new(),
                    trailing: CommentList::new(),
                });
            }
            let group = groups.last_mut().unwrap();
            if comment.is_leading() {
                group.leading.push(comment);
            } else {
                group.trailing.push(comment);
            }
        }
        let orphan_indices = groups
            .iter()
            .enumerate()
            .filter_map(|(index, group)| {
                group
                    .leading
                    .iter()
                    .chain(&group.trailing)
                    .any(|comment| preserve_when_orphaned(*comment))
                    .then_some(index)
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { groups, anchor_presence, orphan_indices, remaining }
    }

    #[inline]
    fn may_have_anchor(&self, anchor: u32) -> bool {
        if self.remaining == 0 {
            return false;
        }
        let anchor = anchor as usize;
        self.anchor_presence.get(anchor >> 6).is_some_and(|word| word & (1 << (anchor & 63)) != 0)
    }

    #[inline]
    fn index(&self, anchor: u32) -> Option<usize> {
        if !self.may_have_anchor(anchor) {
            return None;
        }
        self.groups.binary_search_by_key(&anchor, |group| group.anchor).ok()
    }

    #[inline]
    fn clear_if_empty(&mut self, index: usize) {
        let group = &self.groups[index];
        if group.leading.is_empty() && group.trailing.is_empty() {
            let anchor = group.anchor as usize;
            self.anchor_presence[anchor >> 6] &= !(1 << (anchor & 63));
        }
    }

    fn has_non_semantic_at(&self, anchor: u32) -> bool {
        self.index(anchor).is_some_and(|index| {
            let group = &self.groups[index];
            group
                .leading
                .iter()
                .chain(&group.trailing)
                .any(|comment| !comment.is_pure() && !comment.is_no_side_effects())
        })
    }

    #[inline]
    fn leading_at(&self, anchor: u32) -> Option<&CommentList> {
        let group = self.index(anchor).map(|index| &self.groups[index])?;
        (!group.leading.is_empty()).then_some(&group.leading)
    }

    #[inline]
    fn trailing_at(&self, anchor: u32) -> Option<&CommentList> {
        let group = self.index(anchor).map(|index| &self.groups[index])?;
        (!group.trailing.is_empty()).then_some(&group.trailing)
    }

    fn take_leading_at(&mut self, anchor: u32) -> Option<CommentList> {
        let index = self.index(anchor)?;
        if self.groups[index].leading.is_empty() {
            return None;
        }
        let comments = std::mem::take(&mut self.groups[index].leading);
        self.remaining -= comments.len();
        self.clear_if_empty(index);
        Some(comments)
    }

    fn take_trailing_at(&mut self, anchor: u32) -> Option<CommentList> {
        let index = self.index(anchor)?;
        if self.groups[index].trailing.is_empty() {
            return None;
        }
        let comments = std::mem::take(&mut self.groups[index].trailing);
        self.remaining -= comments.len();
        self.clear_if_empty(index);
        Some(comments)
    }

    fn take_matching_at(
        &mut self,
        anchor: u32,
        predicate: impl Fn(&Comment) -> bool + Copy,
    ) -> CommentList {
        let Some(index) = self.index(anchor) else { return CommentList::new() };
        let group = &mut self.groups[index];
        let mut comments = take_matching(&mut group.leading, predicate);
        comments.extend(take_matching(&mut group.trailing, predicate));
        comments.sort_unstable_by_key(|comment| comment.span.start);
        self.remaining -= comments.len();
        self.clear_if_empty(index);
        comments
    }

    fn take_matching_leading_at(
        &mut self,
        anchor: u32,
        predicate: impl Fn(&Comment) -> bool,
    ) -> CommentList {
        let Some(index) = self.index(anchor) else { return CommentList::new() };
        let comments = take_matching(&mut self.groups[index].leading, predicate);
        self.remaining -= comments.len();
        self.clear_if_empty(index);
        comments
    }

    fn nearest_matching_leading_anchor(
        &self,
        anchor: u32,
        predicate: impl Fn(&Comment) -> bool,
    ) -> Option<u32> {
        let end = self.groups.partition_point(|group| group.anchor <= anchor);
        self.groups[..end]
            .iter()
            .rev()
            .find(|group| group.leading.iter().any(&predicate))
            .map(|group| group.anchor)
    }

    fn take_matching_trailing_at(
        &mut self,
        anchor: u32,
        predicate: impl Fn(&Comment) -> bool,
    ) -> CommentList {
        let Some(index) = self.index(anchor) else { return CommentList::new() };
        let comments = take_matching(&mut self.groups[index].trailing, predicate);
        self.remaining -= comments.len();
        self.clear_if_empty(index);
        comments
    }

    fn remove_comments(&mut self, removed: &[Comment]) {
        if removed.is_empty() || self.remaining == 0 {
            return;
        }
        for comment in removed {
            let Some(index) = self.index(comment.attached_to) else { continue };
            let group = &mut self.groups[index];
            for comments in [&mut group.leading, &mut group.trailing] {
                let before = comments.len();
                comments.retain(|candidate| candidate.span != comment.span);
                self.remaining -= before - comments.len();
            }
            self.clear_if_empty(index);
        }
    }

    #[inline]
    fn bounds(&self, start: u32, end: u32, inclusive: bool) -> (usize, usize) {
        let first = self.groups.partition_point(|group| group.anchor < start);
        let last = if inclusive {
            self.groups.partition_point(|group| group.anchor <= end)
        } else {
            self.groups.partition_point(|group| group.anchor < end)
        };
        (first, last)
    }

    fn has_between(&self, start: u32, end: u32) -> bool {
        if start >= end {
            return false;
        }
        let (first, last) = self.bounds(start.saturating_add(1), end, false);
        self.groups[first..last]
            .iter()
            .any(|group| !group.leading.is_empty() || !group.trailing.is_empty())
    }

    fn first_between(
        &self,
        start: u32,
        end: u32,
        predicate: impl Fn(&Comment) -> bool,
    ) -> Option<Comment> {
        if start >= end {
            return None;
        }
        let (first, last) = self.bounds(start.saturating_add(1), end, false);
        self.groups[first..last]
            .iter()
            .flat_map(|group| group.leading.iter().chain(&group.trailing))
            .filter(|comment| predicate(comment))
            .min_by_key(|comment| comment.span.start)
            .copied()
    }

    fn take_between(
        &mut self,
        start: u32,
        end: u32,
        predicate: impl Fn(&Comment) -> bool + Copy,
    ) -> CommentList {
        if start >= end {
            return CommentList::new();
        }
        let (first, last) = self.bounds(start.saturating_add(1), end, false);
        let mut comments = CommentList::new();
        for group in &mut self.groups[first..last] {
            comments.extend(take_matching(&mut group.leading, predicate));
            comments.extend(take_matching(&mut group.trailing, predicate));
        }
        for index in first..last {
            self.clear_if_empty(index);
        }
        comments.sort_unstable_by_key(|comment| comment.span.start);
        self.remaining -= comments.len();
        comments
    }

    fn has_orphan_before(&self, end: u32) -> bool {
        let last = self.orphan_indices.partition_point(|&index| self.groups[index].anchor < end);
        self.orphan_indices[..last].iter().any(|&index| {
            let group = &self.groups[index];
            group
                .leading
                .iter()
                .chain(&group.trailing)
                .any(|comment| preserve_when_orphaned(*comment))
        })
    }

    fn take_orphans_before(&mut self, end: u32) -> CommentList {
        let last = self.orphan_indices.partition_point(|&index| self.groups[index].anchor < end);
        let mut comments = CommentList::new();
        for orphan_index in 0..last {
            let group_index = self.orphan_indices[orphan_index];
            let group = &mut self.groups[group_index];
            comments.extend(take_matching(&mut group.leading, |comment| {
                preserve_when_orphaned(*comment)
            }));
            comments.extend(take_matching(&mut group.trailing, |comment| {
                preserve_when_orphaned(*comment)
            }));
            self.clear_if_empty(group_index);
        }
        comments.sort_unstable_by_key(|comment| comment.span.start);
        self.remaining -= comments.len();
        comments
    }
}

fn take_matching(comments: &mut CommentList, predicate: impl Fn(&Comment) -> bool) -> CommentList {
    let mut taken = CommentList::new();
    comments.retain(|comment| {
        if predicate(comment) {
            taken.push(*comment);
            false
        } else {
            true
        }
    });
    taken
}

/// Whether a comment remains meaningful if its original AST anchor is removed.
fn preserve_when_orphaned(comment: Comment) -> bool {
    comment.is_legal() || comment.is_coverage_ignore_file()
}

fn is_html_comment(comment: Comment, source_text: Option<&str>) -> bool {
    comment.is_line()
        && source_text.is_some_and(|source_text| {
            let value = comment.span.source_text(source_text);
            value.starts_with("<!--") || value.starts_with("-->")
        })
}

/// A `pife`-marked arrow or function expression prints its leading comments
/// inside its own `(` wrap, so operand emission sites must not consume them.
pub fn is_pife_function(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => arrow.pife,
        Expression::FunctionExpression(function) => function.pife,
        Expression::ParenthesizedExpression(paren) => is_pife_function(&paren.expression),
        _ => false,
    }
}

/// Which annotation kind an emission site expects to recover from
/// [`Codegen::comments`].
///
/// `@__PURE__` / `#__PURE__` on a `CallExpression` or `NewExpression`, and
/// `@__NO_SIDE_EFFECTS__` / `#__NO_SIDE_EFFECTS__` on a function declaration or
/// expression, are not interchangeable: downstream tree-shakers only honor
/// each on its corresponding node kind. The filter prevents
/// [`Codegen::print_annotation_comment`] from emitting one kind where the
/// other was expected when both share an `attached_to`.
#[derive(Clone, Copy)]
pub enum AnnotationKind {
    Pure,
    NoSideEffects,
}

impl AnnotationKind {
    #[inline]
    fn matches(self, comment: &Comment) -> bool {
        match self {
            Self::Pure => comment.is_pure(),
            Self::NoSideEffects => comment.is_no_side_effects(),
        }
    }

    /// Canonical literal to emit when no verbatim source is available.
    /// `newline_after = true` is used at statement-level emission sites
    /// (function declarations, exports), `false` at inline emission sites
    /// (call / new / function expressions).
    #[inline]
    fn canonical(self, newline_after: bool) -> &'static str {
        match (self, newline_after) {
            (Self::Pure, false) => "/* @__PURE__ */ ",
            (Self::Pure, true) => "/* @__PURE__ */\n",
            (Self::NoSideEffects, false) => "/* @__NO_SIDE_EFFECTS__ */ ",
            (Self::NoSideEffects, true) => "/* @__NO_SIDE_EFFECTS__ */\n",
        }
    }
}

impl Codegen<'_> {
    #[inline]
    pub(crate) fn has_pending_comments(&self) -> bool {
        self.comments.remaining != 0
    }

    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        if self.options.comments == CommentOptions::disabled() {
            return;
        }
        let mut retained = Vec::with_capacity(comments.len());
        for comment in comments {
            if comment.is_pure() || comment.is_no_side_effects() {
                if comment.is_leading() && self.options.print_annotation_comment() {
                    retained.push(*comment);
                }
                continue;
            }

            let add = (comment.is_legal() && self.options.print_legal_comment())
                || (comment.is_jsdoc() && self.options.print_jsdoc_comment())
                || (comment.is_annotation() && self.options.print_annotation_comment())
                || (comment.is_normal() && self.options.print_normal_comment());

            if add {
                self.has_property_key_annotations |= comment.is_property_key_annotation();
                retained.push(*comment);
            }
        }
        self.comments = CommentStore::build(&mut retained);
    }

    /// Build the fast NodeId-owned comment store produced by semantic analysis.
    pub(crate) fn build_node_comments(&mut self, program: &Program<'_>) -> bool {
        let Some(store) = NodeCommentStore::build(program, |comment| {
            if comment.is_line()
                || comment.is_pure()
                || comment.is_no_side_effects()
                || comment.is_property_key_annotation()
                || preserve_when_orphaned(comment)
            {
                return false;
            }
            (comment.is_legal() && self.options.print_legal_comment())
                || (comment.is_jsdoc() && self.options.print_jsdoc_comment())
                || (comment.is_annotation() && self.options.print_annotation_comment())
                || (comment.is_normal() && self.options.print_normal_comment())
        }) else {
            return false;
        };
        self.node_comments = store;
        // Keep the source-offset store as the recovery owner for removed or
        // replaced hosts. A successful node claim removes its matching copy.
        let fallback_comments = program
            .comments
            .iter()
            .copied()
            .filter(|comment| {
                self.node_comments.exclusive.binary_search(&comment.span.start).is_err()
            })
            .collect::<Vec<_>>();
        self.build_comments(&fallback_comments);
        true
    }

    #[inline]
    pub(crate) fn take_node_comments(&mut self, node_id: NodeId) -> Option<NodeComments> {
        self.node_comments.take_all(node_id)
    }

    #[inline]
    pub(crate) fn take_boundary_comments(
        &mut self,
        node_id: NodeId,
        start: u32,
        end: u32,
    ) -> Option<BoundaryComments> {
        let node = self.node_comments.take_all(node_id);
        let leading = self.comments.may_have_anchor(start);
        let trailing = self.comments.may_have_anchor(end);
        (node.is_some() || leading || trailing).then_some(BoundaryComments {
            node,
            leading,
            trailing,
        })
    }

    #[inline]
    pub(crate) fn take_expression_boundary_comments(
        &mut self,
        expression: &Expression<'_>,
    ) -> Option<BoundaryComments> {
        let node = self.node_comments.take_all(expression.get_node_id());
        let leading = self.may_have_comments_before_expression(expression);
        let trailing = self.comments.may_have_anchor(expression.span().end);
        (node.is_some() || leading || trailing).then_some(BoundaryComments {
            node,
            leading,
            trailing,
        })
    }

    fn may_have_comments_before_expression(&self, expression: &Expression<'_>) -> bool {
        if is_pife_function(expression) || matches!(expression, Expression::ObjectExpression(_)) {
            return false;
        }
        self.comments.may_have_anchor(expression.span().start)
            || matches!(expression, Expression::ParenthesizedExpression(paren) if self.may_have_comments_before_expression(&paren.expression))
    }

    pub(crate) fn print_node_comments_before_id(&mut self, node_id: NodeId) {
        if let Some(mut comments) = self.take_node_comments(node_id) {
            self.print_node_comments_before(&mut comments);
        }
    }

    #[inline]
    pub(crate) fn print_node_comments_before(&mut self, node_comments: &mut NodeComments) {
        let comments = if node_comments.before.is_empty() {
            std::mem::take(&mut node_comments.inside)
        } else {
            let mut comments = std::mem::take(&mut node_comments.before);
            if !node_comments.inside.is_empty() {
                comments.extend(std::mem::take(&mut node_comments.inside));
                comments.sort_unstable_by_key(|comment| comment.span.start);
            }
            comments
        };
        // Until a delimiter claims `inside` explicitly, printing it at the
        // host's entry is the safe lossless fallback.
        if comments.is_empty() {
            return;
        }
        self.comments.remove_comments(&comments);
        self.print_comments_inner(&comments);
        if self.last_byte() != Some(b'\n') {
            self.consume_pending_indent_space();
        }
    }

    #[inline]
    pub(crate) fn print_node_comments_after(&mut self, node_comments: &mut NodeComments) {
        let comments = std::mem::take(&mut node_comments.after);
        if comments.is_empty() {
            return;
        }
        self.comments.remove_comments(&comments);
        let removed_newline = self.last_byte() == Some(b'\n');
        if removed_newline {
            self.code.pop_byte();
        }
        if self.last_byte() != Some(b'\n') && !self.consume_pending_indent_space() {
            self.print_soft_space();
        }
        self.print_comments_inner(&comments);
        self.clear_pending_indent_space();
        if removed_newline && self.last_byte() != Some(b'\n') {
            self.print_hard_newline();
        }
    }

    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.has_non_semantic_at(start)
    }

    pub(crate) fn has_normal_comment(&self, start: u32) -> bool {
        self.comments
            .leading_at(start)
            .is_some_and(|comments| comments.iter().any(|comment| comment.is_normal()))
    }

    /// Emit a pure / no-side-effects annotation comment for the AST node at
    /// `start`, falling back to the canonical literal when no verbatim source
    /// can be recovered.
    ///
    /// The fallback covers four cases:
    /// - no annotation comment is stashed at `start`,
    /// - the stashed comment's kind doesn't match the emission site (e.g. a
    ///   `@__NO_SIDE_EFFECTS__` slot being queried by a `CallExpression`
    ///   site that needs `@__PURE__`),
    /// - the comment is a line comment but the site can't break the line, or
    /// - source text is unavailable (e.g. the [`Codegen::print_expression`]
    ///   path that skips [`Codegen::build_comments`]).
    ///
    /// Export sites pass `self.span.start` and only recover verbatim when the
    /// annotation precedes the `export` keyword. The rarer
    /// `export /* @__NO_SIDE_EFFECTS__ */ function …` form (annotation between
    /// `export` and `function`) attaches to the inner function's span and
    /// falls back to canonical here.
    pub(crate) fn print_annotation_comment(
        &mut self,
        start: u32,
        kind: AnnotationKind,
        newline_after: bool,
    ) {
        let source_anchor = self.comments.nearest_matching_leading_anchor(start, |comment| {
            kind.matches(comment) && (newline_after || !comment.is_line())
        });
        if self.source_text.is_some()
            && let Some(source_anchor) = source_anchor
        {
            let mut comments = self.comments.take_matching_leading_at(source_anchor, |comment| {
                kind.matches(comment) && (newline_after || !comment.is_line())
            });
            // The semantic claim above remains kind-specific. Once its source
            // anchor is owned, retain compatible sibling comments at that
            // exact boundary instead of relocating them to statement fallback.
            comments.extend(self.comments.take_matching_leading_at(source_anchor, |comment| {
                newline_after || !comment.is_line()
            }));
            comments.sort_unstable_by_key(|comment| comment.span.start);
            for (index, comment) in comments.iter().enumerate() {
                if index != 0 {
                    self.print_str(" ");
                }
                self.print_comment(comment);
            }
            if newline_after {
                self.print_hard_newline();
            } else {
                self.print_str(" ");
            }
            return;
        }
        self.print_str(kind.canonical(newline_after));
    }

    pub(crate) fn print_leading_comments(&mut self, start: u32) {
        if let Some(comments) = self.comments.take_leading_at(start) {
            self.print_comments(&comments);
        }
    }

    pub(crate) fn get_comments(&mut self, start: u32) -> Option<CommentList> {
        let comments = self
            .comments
            .take_matching_at(start, |comment| !comment.is_pure() && !comment.is_no_side_effects());
        self.node_comments.remove_comments(&comments);
        (!comments.is_empty()).then_some(comments)
    }

    #[inline]
    pub(crate) fn print_comments_at(&mut self, start: u32) {
        if let Some(comments) = self.get_comments(start) {
            self.print_comments(&comments);
        }
    }

    /// Print parser-attached annotations and JSDoc at a surviving AST node.
    /// Normal comments keep using their existing syntax-specific emitters,
    /// which preserve punctuation-sensitive spacing and transformed-AST
    /// behavior. Invalid pure annotations and property-key annotations also
    /// have dedicated emission sites.
    #[inline]
    fn has_attached_comments_at(&self, start: u32) -> bool {
        self.comments.leading_at(start).is_some_and(|comments| {
            comments.iter().any(|comment| {
                (!self.suppress_normal_comments && comment.is_normal())
                    || comment.is_jsdoc()
                    || (comment.is_annotation()
                        && comment.content != CommentContent::PureNotApplied
                        && !comment.is_pure()
                        && !comment.is_no_side_effects()
                        && !comment.is_property_key_annotation())
            })
        })
    }

    #[inline]
    pub(crate) fn print_attached_comments_at(&mut self, start: u32) {
        if !self.comments.may_have_anchor(start) {
            return;
        }
        self.print_attached_comments_at_slow(start);
    }

    #[cold]
    #[inline(never)]
    fn print_attached_comments_at_slow(&mut self, start: u32) {
        let comments = self.take_attached_comments_at(start);
        if !comments.is_empty() {
            self.print_comments(&comments);
            if self.last_byte() != Some(b'\n') {
                self.consume_pending_indent_space();
            }
        }
    }

    fn take_attached_comments_at(&mut self, start: u32) -> CommentList {
        if !self.has_attached_comments_at(start) {
            return CommentList::new();
        }
        let suppress_normal_comments = self.suppress_normal_comments;
        self.comments.take_matching_leading_at(start, |comment| {
            (!suppress_normal_comments && comment.is_normal())
                || comment.is_jsdoc()
                || (comment.is_annotation()
                    && comment.content != CommentContent::PureNotApplied
                    && !comment.is_pure()
                    && !comment.is_no_side_effects()
                    && !comment.is_property_key_annotation())
        })
    }

    #[inline]
    pub(crate) fn print_trailing_attached_comments_at(&mut self, end: u32) {
        if !self.comments.may_have_anchor(end) {
            return;
        }
        self.print_trailing_attached_comments_at_slow(end);
    }

    #[cold]
    #[inline(never)]
    fn print_trailing_attached_comments_at_slow(&mut self, end: u32) {
        let source_text = self.source_text;
        let should_print = self.comments.trailing_at(end).is_some_and(|comments| {
            comments.iter().any(|comment| {
                !is_html_comment(*comment, source_text)
                    && (comment.is_normal()
                        || comment.is_jsdoc()
                        || (comment.is_annotation() && !comment.is_property_key_annotation()))
            })
        });
        if should_print {
            // Statement printers commonly emit their terminating newline
            // before the generic `Gen::print` trailing hook runs. Move that
            // newline behind the trailing comment so `x; // comment` does not
            // become a leading comment for the next statement on pass two.
            let removed_newline = self.last_byte() == Some(b'\n');
            if removed_newline {
                self.code.pop_byte();
            }
            let needs_space = self.comments.trailing_at(end).is_some_and(|comments| {
                comments
                    .iter()
                    .find(|comment| !is_html_comment(**comment, source_text))
                    .is_some_and(|comment| {
                        source_text.is_none_or(|source_text| {
                            let Ok(start) = usize::try_from(end) else { return true };
                            let Ok(comment_start) = usize::try_from(comment.span.start) else {
                                return true;
                            };
                            source_text.get(start..comment_start).is_none_or(|gap| {
                                gap.bytes().any(|byte| byte.is_ascii_whitespace())
                            })
                        })
                    })
            });
            if self.last_byte() != Some(b'\n') {
                if needs_space {
                    if !self.consume_pending_indent_space() {
                        self.print_soft_space();
                    }
                } else {
                    self.clear_pending_indent_space();
                }
            }
            let has_html = self.comments.trailing_at(end).is_some_and(|comments| {
                comments.iter().any(|comment| is_html_comment(*comment, source_text))
            });
            let comments = if has_html {
                self.comments.take_matching_trailing_at(end, |comment| {
                    !is_html_comment(*comment, source_text)
                })
            } else {
                self.comments.take_trailing_at(end).unwrap()
            };
            self.print_comments(&comments);
            self.clear_pending_indent_space();
            if removed_newline && self.last_byte() != Some(b'\n') {
                self.print_hard_newline();
            }
        }
    }

    pub(crate) fn print_attached_comments_before_expression(
        &mut self,
        expression: &Expression<'_>,
    ) {
        if is_pife_function(expression) || matches!(expression, Expression::ObjectExpression(_)) {
            return;
        }
        let start = expression.span().start;
        let comments = self.take_attached_comments_at(start);
        if !comments.is_empty() {
            self.print_comments(&comments);
            if self.last_byte() == Some(b'\n') {
                self.print_indent();
            }
        }
        if let Expression::ParenthesizedExpression(paren) = expression {
            self.print_attached_comments_before_expression(&paren.expression);
        }
    }

    /// Parentheses around a return/yield/arrow operand cannot be discarded when
    /// printing its leading comments would put the operand on the next line.
    /// Without them, ASI changes the parse on the following codegen pass.
    pub(crate) fn leading_comments_cause_newline_before_expression(
        &self,
        expression: &Expression<'_>,
    ) -> bool {
        if self.comments.remaining == 0 || is_pife_function(expression) {
            return false;
        }
        let start = expression.span().start;
        let causes_newline = self.comments.leading_at(start).is_some_and(|comments| {
            comments.iter().any(|comment| {
                let printable = (!self.suppress_normal_comments && comment.is_normal())
                    || comment.is_jsdoc()
                    || (comment.is_annotation()
                        && comment.content != CommentContent::PureNotApplied
                        && !comment.is_pure()
                        && !comment.is_no_side_effects()
                        && !comment.is_property_key_annotation());
                printable && (comment.is_line() || comment.preceded_by_newline())
            })
        });
        causes_newline
            || match expression {
                Expression::ParenthesizedExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.expression)
                }
                Expression::TSAsExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.expression)
                }
                Expression::TSSatisfiesExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.expression)
                }
                Expression::TSTypeAssertion(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.expression)
                }
                Expression::TSNonNullExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.expression)
                }
                Expression::TSInstantiationExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.expression)
                }
                Expression::BinaryExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.left)
                }
                Expression::LogicalExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.left)
                }
                Expression::ConditionalExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.test)
                }
                Expression::StaticMemberExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.object)
                }
                Expression::ComputedMemberExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.object)
                }
                Expression::CallExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.callee)
                }
                Expression::TaggedTemplateExpression(expression) => {
                    self.leading_comments_cause_newline_before_expression(&expression.tag)
                }
                _ => false,
            }
    }

    /// Print leading comments at `start` and glue the next token to them: after a
    /// group ending in a newline (line comment / `followed_by_newline`), print the
    /// indent — mid-expression callers have no statement machinery to do it, and an
    /// unindented next token renders differently once the parser re-anchors the
    /// comments to it (codegen would no longer be idempotent). Otherwise consume the
    /// pending indent-as-space so the token glues with a single space.
    #[inline]
    pub(crate) fn print_leading_comments_anchored_to_self(&mut self, start: u32) {
        if let Some(comments) = self.comments.take_leading_at(start) {
            self.print_comments(&comments);
            if self.last_byte() == Some(b'\n') {
                self.print_indent();
            } else {
                self.consume_pending_indent_space();
            }
        }
    }

    /// Print a property-key annotation attached directly to a string or template literal.
    #[inline]
    pub(crate) fn print_property_key_annotation(&mut self, start: u32) {
        if !self.has_property_key_annotations {
            return;
        }
        if self.comments.leading_at(start).is_some_and(|comments| {
            comments.iter().any(|comment| comment.is_property_key_annotation())
        }) {
            self.print_leading_comments_anchored_to_self(start);
        }
    }

    /// Print comments attached to an expression that survives codegen.
    ///
    /// Probes the parenthesized layers too: `a || /* c */ (x)` anchors the
    /// comment at the `(`, `a || (/* c */ x)` at `x` — an operand printer only
    /// sees one node, so the walk happens here for every emission site.
    pub(crate) fn print_leading_comments_before_expression(&mut self, expression: &Expression<'_>) {
        if is_pife_function(expression) {
            return;
        }
        let start = expression.span().start;
        let comments = self.comments.take_matching_leading_at(start, |comment| {
            !comment.is_pure() && !comment.is_no_side_effects()
        });
        if !comments.is_empty() {
            self.print_comments(&comments);
            if self.last_byte() == Some(b'\n') {
                self.print_indent();
            } else {
                self.consume_pending_indent_space();
            }
        }
        if let Expression::ParenthesizedExpression(paren) = expression {
            self.print_leading_comments_before_expression(&paren.expression);
        }
    }

    /// Print an expression's comment group only when it contains an annotation
    /// comment, probing parenthesized layers like
    /// [`Self::print_leading_comments_before_expression`].
    ///
    /// This is the variant for emission sites that mutating consumers move
    /// statements into (the minifier merges `if(a)x;if(b)x;` into
    /// `if(a||(b,..))x`; rolldown finalizes moved nodes with their original
    /// spans). Comments are anchored by source position, so a dissolved
    /// statement's leading normal-comment group can coincide with the moved
    /// operand's span start — printing it there misplaces statement-level
    /// trivia inside an expression and is not idempotent
    /// (`test_normal_comment_before_logical_rhs_not_printed` documents the
    /// falsifier). Annotations are the one comment kind with expression-level
    /// meaning, so they still pass through.
    pub(crate) fn print_annotation_comments_before_expression(
        &mut self,
        expression: &Expression<'_>,
    ) {
        if is_pife_function(expression) {
            return;
        }
        let start = expression.span().start;
        let has_annotation = self.comments.leading_at(start).is_some_and(|comments| {
            comments.iter().any(|comment| {
                comment.is_annotation()
                    && !comment.is_pure()
                    && !comment.is_no_side_effects()
                    && !comment.is_property_key_annotation()
            })
        });
        if has_annotation {
            let comments = self.comments.take_matching_leading_at(start, |comment| {
                comment.is_annotation()
                    && !comment.is_pure()
                    && !comment.is_no_side_effects()
                    && !comment.is_property_key_annotation()
            });
            self.print_comments(&comments);
            if self.last_byte() == Some(b'\n') {
                self.print_indent();
            } else {
                self.consume_pending_indent_space();
            }
        }
        if let Expression::ParenthesizedExpression(paren) = expression {
            self.print_annotation_comments_before_expression(&paren.expression);
        }
    }

    /// Whether an orphan comment with `attached_to < end` is still pending.
    /// Used by block emitters to keep an empty body multi-line.
    #[inline]
    pub(crate) fn has_orphan_comments_before(&self, end: u32) -> bool {
        self.comments.has_orphan_before(end)
    }

    /// Drain pending orphan comments with `attached_to < end` and emit them in
    /// source order. Called at every statement boundary so legal and file-level
    /// coverage comments survive when their original anchor was removed by an
    /// upstream pass.
    #[inline]
    pub(crate) fn print_orphan_comments_before(&mut self, end: u32) {
        let mut orphans = self.comments.take_orphans_before(end);
        if let Some(last) = orphans.last_mut() {
            // Orphans aren't in their original position, so the source's
            // `followed_by_newline` hint no longer applies. Force it on so
            // `print_comments` emits a trailing newline instead of setting
            // `print_next_indent_as_space` — otherwise the next indent (often
            // before `}`) collapses to a space and pass 2 stops matching.
            last.set_followed_by_newline(true);
            self.print_comments(&orphans);
        }
    }

    /// Print comments attached to any position in the given range `(start, end)` (exclusive).
    /// Returns `true` if any comments were printed.
    pub(crate) fn print_comments_in_range(&mut self, start: u32, end: u32) -> bool {
        let comments = self.comments.take_between(start, end, |comment| {
            !comment.is_pure() && !comment.is_no_side_effects()
        });
        if comments.is_empty() {
            return false;
        }
        self.print_comments(&comments);
        true
    }

    pub(crate) fn has_comments_in_range(&self, start: u32, end: u32) -> bool {
        self.comments.has_between(start, end)
    }

    pub(crate) fn comments_in_range_need_space_after(&self, start: u32, end: u32) -> bool {
        let Some(comment) = self.comments.first_between(start, end, |comment| {
            !comment.is_pure() && !comment.is_no_side_effects()
        }) else {
            return false;
        };
        self.source_text.is_none_or(|source_text| {
            let Ok(start) = usize::try_from(start) else { return true };
            let Ok(comment_start) = usize::try_from(comment.span.start) else { return true };
            source_text
                .get(start..comment_start)
                .is_none_or(|gap| gap.as_bytes().last().is_some_and(u8::is_ascii_whitespace))
        })
    }

    pub(crate) fn print_comments_before_closing_delimiter(&mut self, close: u32) -> bool {
        let Some(comments) = self.comments.take_leading_at(close) else { return false };
        let needs_space = comments.last().is_some_and(|comment| {
            self.source_text.is_some_and(|source_text| {
                let Ok(comment_end) = usize::try_from(comment.span.end) else { return false };
                let Ok(close) = usize::try_from(close) else { return false };
                source_text
                    .get(comment_end..close)
                    .and_then(|gap| gap.as_bytes().last())
                    .is_some_and(u8::is_ascii_whitespace)
            })
        });
        self.print_comments(&comments);
        if needs_space && self.last_byte() != Some(b'\n') {
            self.consume_pending_indent_space();
        } else {
            self.clear_pending_indent_space();
        }
        true
    }

    pub(crate) fn print_comments_in_range_anchored_to_next(
        &mut self,
        start: u32,
        end: u32,
    ) -> bool {
        if !self.print_comments_in_range(start, end) {
            return false;
        }
        if self.last_byte() == Some(b'\n') {
            self.print_indent();
        } else {
            self.consume_pending_indent_space();
        }
        true
    }

    pub(crate) fn print_leading_comments_in_range_anchored_to_next(
        &mut self,
        start: u32,
        end: u32,
    ) -> bool {
        let comments = self.comments.take_between(start, end, |comment| {
            comment.is_leading() && !comment.is_pure() && !comment.is_no_side_effects()
        });
        if comments.is_empty() {
            return false;
        }
        self.print_comments(&comments);
        if self.last_byte() == Some(b'\n') {
            self.print_indent();
        } else {
            self.consume_pending_indent_space();
        }
        true
    }

    pub(crate) fn print_trailing_comments_in_range_anchored_to_next(
        &mut self,
        start: u32,
        end: u32,
    ) -> bool {
        let comments = self.comments.take_between(start, end, |comment| {
            comment.is_trailing() && !comment.is_pure() && !comment.is_no_side_effects()
        });
        if comments.is_empty() {
            return false;
        }
        self.print_comments(&comments);
        if self.last_byte() == Some(b'\n') {
            self.print_indent();
        } else {
            self.consume_pending_indent_space();
        }
        true
    }

    pub(crate) fn print_expr_comments_before_closing_delimiter(
        &mut self,
        start: u32,
        close: u32,
    ) -> bool {
        let mut comments = self.comments.take_between(start, close, |comment| {
            !comment.is_pure() && !comment.is_no_side_effects()
        });
        comments.extend(self.comments.take_matching_leading_at(close, |comment| {
            !comment.is_pure() && !comment.is_no_side_effects()
        }));
        comments.sort_unstable_by_key(|comment| comment.span.start);
        self.print_expr_comment_list(&comments)
    }

    pub(crate) fn print_all_remaining_orphan_comments(&mut self) {
        let mut comments = self.comments.take_orphans_before(u32::MAX);
        if comments.is_empty() {
            return;
        }
        if self.last_byte() != Some(b'\n')
            && comments.first().is_some_and(|comment| !comment.preceded_by_newline())
        {
            self.print_soft_space();
        }
        comments.last_mut().unwrap().set_followed_by_newline(true);
        self.print_comments(&comments);
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        let comments = self
            .comments
            .take_matching_at(start, |comment| !comment.is_pure() && !comment.is_no_side_effects());
        self.print_expr_comment_list(&comments)
    }

    fn print_expr_comment_list(&mut self, comments: &[Comment]) -> bool {
        if comments.is_empty() {
            return false;
        }
        self.node_comments.remove_comments(comments);
        for comment in comments {
            self.print_hard_newline();
            self.print_indent();
            self.print_comment(comment);
        }

        if comments.is_empty() {
            false
        } else {
            self.print_hard_newline();
            true
        }
    }

    pub(crate) fn print_comments(&mut self, comments: &[Comment]) {
        self.node_comments.remove_comments(comments);
        self.print_comments_inner(comments);
    }

    fn print_comments_inner(&mut self, comments: &[Comment]) {
        let Some((first, rest)) = comments.split_first() else {
            return;
        };

        if first.preceded_by_newline() {
            // Skip printing newline if this comment is already on a newline.
            if let Some(b) = self.last_byte() {
                match b {
                    b'\n' => self.print_indent(),
                    b'\t' => { /* noop */ }
                    _ => {
                        self.print_hard_newline();
                        self.print_indent();
                    }
                }
            }
        } else if !self.consume_pending_indent_space()
            && matches!(self.last_byte(), None | Some(b'\n'))
        {
            // Only indent at a line start. Mid-line emission sites (`a ?? /* c */ b`,
            // `key: /* c */ value`, `${/* c */ expr}`) would otherwise get a full
            // indent injected mid-line, growing indentation on every codegen pass.
            self.print_indent();
        }
        self.print_comment(first);

        if let Some((last, middle)) = rest.split_last() {
            for comment in middle {
                if comment.preceded_by_newline() {
                    self.print_hard_newline();
                    self.print_indent();
                } else if comment.is_legal() {
                    self.print_hard_newline();
                } else {
                    self.print_soft_space();
                }
                self.print_comment(comment);
            }

            if last.preceded_by_newline() {
                self.print_hard_newline();
                self.print_indent();
            } else if last.is_legal() {
                self.print_hard_newline();
            } else {
                self.print_soft_space();
            }
            self.print_comment(last);

            if last.is_line() || last.followed_by_newline() {
                self.print_hard_newline();
            } else {
                self.print_next_indent_as_space = true;
            }
        } else if first.is_line() || first.followed_by_newline() {
            self.print_hard_newline();
        } else {
            self.print_next_indent_as_space = true;
        }
    }

    fn print_comment(&mut self, comment: &Comment) {
        let Some(source_text) = self.source_text else {
            return;
        };
        let comment_source = comment.span.source_text(source_text);
        match comment.kind {
            CommentKind::Line | CommentKind::SingleLineBlock => {
                self.print_str_escaping_script_close_tag(comment_source);
            }
            CommentKind::MultiLineBlock => {
                for line in LineTerminatorSplitter::new(comment_source) {
                    if !line.starts_with("/*") {
                        self.print_indent();
                    }
                    self.print_str_escaping_script_close_tag(line.trim_start());
                    if !line.ends_with("*/") {
                        self.print_hard_newline();
                    }
                }
            }
        }
    }

    /// Handle Eof / Linked / External Comments.
    /// Return a list of comments of linked or external.
    pub(crate) fn handle_eof_linked_or_external_comments(
        &mut self,
        program: &Program<'_>,
    ) -> Vec<Comment> {
        let legal_comments = &self.options.comments.legal;
        if matches!(legal_comments, LegalComment::None | LegalComment::Inline) {
            return vec![];
        }

        // Dedupe legal comments for smaller output size.
        let mut set = FxHashSet::default();
        let mut comments = vec![];

        let source_text = program.source_text;
        for comment in program.comments.iter().filter(|c| c.is_legal()) {
            let mut text = Cow::Borrowed(comment.span.source_text(source_text));
            if comment.is_multiline_block() {
                let mut buffer = String::with_capacity(text.len());
                // Print block comments with our own indentation.
                for line in LineTerminatorSplitter::new(&text) {
                    if !line.starts_with("/*") {
                        buffer.push('\t');
                    }
                    buffer.push_str(line.trim_start());
                    if !line.ends_with("*/") {
                        buffer.push('\n');
                    }
                }
                text = Cow::Owned(buffer);
            }
            if set.insert(text) {
                comments.push(*comment);
            }
        }

        if comments.is_empty() {
            return vec![];
        }

        match legal_comments {
            LegalComment::Eof => {
                self.print_hard_newline();
                // Clear the flag to ensure consistent formatting for all EOF comments
                self.print_next_indent_as_space = false;
                for c in comments {
                    self.print_comment(&c);
                    self.print_hard_newline();
                }
                vec![]
            }
            LegalComment::Linked(path) => {
                let path = path.clone();
                self.print_hard_newline();
                self.print_str("/*! For license information please see ");
                self.print_str(&path);
                self.print_str(" */");
                comments
            }
            LegalComment::External => comments,
            LegalComment::None | LegalComment::Inline => unreachable!(),
        }
    }
}
