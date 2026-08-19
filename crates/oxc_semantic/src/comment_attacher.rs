use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use oxc_allocator::HashMap as ArenaHashMap;
use oxc_ast::{AstKind, CommentId, CommentPosition, CommentStore, NodeComments};
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;

/// Attaches comments while semantic analysis performs its existing AST traversal.
///
/// The original node map is taken up front so parser IDs cannot collide with freshly allocated
/// dense semantic IDs while they are being remapped.
pub(crate) struct CommentAttacher<'a> {
    comments: &'a CommentStore<'a>,
    original_nodes: ArenaHashMap<'a, NodeId, NodeComments<'a>>,
    leading: FxHashMap<u32, SmallVec<[CommentId; 1]>>,
    leading_boundaries: Vec<u32>,
    trailing: FxHashMap<u32, SmallVec<[CommentId; 1]>>,
    leading_candidates: FxHashMap<u32, NodeId>,
    trailing_candidates: FxHashMap<u32, NodeId>,
    original_leading_boundaries: FxHashSet<u32>,
    original_trailing_boundaries: FxHashSet<u32>,
    dangling_candidates: FxHashMap<CommentId, (u32, NodeId)>,
}

impl<'a> CommentAttacher<'a> {
    pub fn new(comments: &'a CommentStore<'a>) -> Self {
        let original_nodes = comments.take_attachments();
        let mut attached = FxHashSet::default();
        let mut original_leading_boundaries = FxHashSet::default();
        let mut original_trailing_boundaries = FxHashSet::default();
        for node_comments in original_nodes.values() {
            attached.extend(node_comments.leading.iter().copied());
            attached.extend(node_comments.trailing.iter().copied());
            attached.extend(node_comments.dangling.iter().copied());
            original_leading_boundaries.extend(
                node_comments
                    .leading
                    .iter()
                    .map(|comment_id| comments[comment_id.index()].attached_to),
            );
            original_trailing_boundaries.extend(
                node_comments
                    .trailing
                    .iter()
                    .map(|comment_id| comments[comment_id.index()].attached_to),
            );
        }

        let mut leading = FxHashMap::default();
        let mut trailing = FxHashMap::default();
        for (index, comment) in comments.iter().enumerate() {
            let comment_id = CommentId::from_usize(index);
            if attached.contains(&comment_id) || comments.is_suppressed(comment_id) {
                continue;
            }
            let target = match comment.position {
                CommentPosition::Leading => &mut leading,
                CommentPosition::Trailing => &mut trailing,
            };
            target.entry(comment.attached_to).or_insert_with(SmallVec::new).push(comment_id);
        }
        let mut leading_boundaries = leading.keys().copied().collect::<Vec<_>>();
        leading_boundaries.sort_unstable();

        Self {
            comments,
            original_nodes,
            leading,
            leading_boundaries,
            trailing,
            leading_candidates: FxHashMap::default(),
            trailing_candidates: FxHashMap::default(),
            original_leading_boundaries,
            original_trailing_boundaries,
            dangling_candidates: FxHashMap::default(),
        }
    }

    /// Remap parser/previous-epoch ownership and claim leading comments for this node.
    pub fn enter_node(&mut self, kind: AstKind<'a>, old_node_id: NodeId, new_node_id: NodeId) {
        if old_node_id != NodeId::DUMMY
            && let Some(node_comments) = self.original_nodes.remove(&old_node_id)
        {
            self.comments.attach_node_comments(new_node_id, node_comments);
        }

        if matches!(kind, AstKind::Program(_) | AstKind::Hashbang(_)) {
            return;
        }
        if self.original_leading_boundaries.contains(&kind.span().start) {
            self.leading_candidates.entry(kind.span().start).or_insert(new_node_id);
        }
        if let Some(comment_ids) = self.leading.remove(&kind.span().start) {
            self.comments.attach(new_node_id, CommentPosition::Leading, comment_ids);
        }
        if matches!(
            kind,
            AstKind::VariableDeclarator(_) | AstKind::CallExpression(_) | AstKind::NewExpression(_)
        ) {
            let span = kind.span();
            let span_len = span.size();
            let start = self.leading_boundaries.partition_point(|&boundary| boundary <= span.start);
            let end = self.leading_boundaries.partition_point(|&boundary| boundary < span.end);
            for boundary in &self.leading_boundaries[start..end] {
                if let Some(comment_ids) = self.leading.get(boundary) {
                    for &comment_id in comment_ids {
                        let candidate = self
                            .dangling_candidates
                            .entry(comment_id)
                            .or_insert((span_len, new_node_id));
                        if span_len < candidate.0 {
                            *candidate = (span_len, new_node_id);
                        }
                    }
                }
            }
        }
    }

    /// Claim trailing comments for the innermost node ending at this boundary.
    pub fn leave_node(&mut self, kind: AstKind<'a>) {
        if matches!(kind, AstKind::Program(_) | AstKind::Hashbang(_)) {
            return;
        }
        if self.original_trailing_boundaries.contains(&kind.span().end) {
            self.trailing_candidates.entry(kind.span().end).or_insert(kind.node_id());
        }
        if let Some(comment_ids) = self.trailing.remove(&kind.span().end) {
            self.comments.attach(kind.node_id(), CommentPosition::Trailing, comment_ids);
        }
    }

    /// Suppress ordinary comments whose owning nodes disappeared before this semantic rebuild.
    /// Legal and file-level coverage comments remain unresolved for orphan rescue.
    pub fn finish(mut self) {
        for comment_ids in self.leading.values() {
            for &comment_id in comment_ids {
                if let Some(&(_, node_id)) = self.dangling_candidates.get(&comment_id) {
                    self.comments.attach_dangling(node_id, [comment_id]);
                }
            }
        }
        while let Some(node_id) = self.original_nodes.keys().next().copied() {
            let node_comments = self.original_nodes.remove(&node_id).unwrap();
            for comment_id in node_comments.leading {
                self.transfer_or_suppress(comment_id, CommentPosition::Leading);
            }
            for comment_id in node_comments.trailing {
                self.transfer_or_suppress(comment_id, CommentPosition::Trailing);
            }
            for comment_id in node_comments.dangling {
                self.comments.suppress_comment_if_ordinary(comment_id);
            }
        }
    }

    fn transfer_or_suppress(&self, comment_id: CommentId, position: CommentPosition) {
        let comment = self.comments[comment_id.index()];
        let candidate = match position {
            CommentPosition::Leading => self.leading_candidates.get(&comment.attached_to),
            CommentPosition::Trailing => self.trailing_candidates.get(&comment.attached_to),
        };
        if let Some(&node_id) = candidate {
            self.comments.attach(node_id, position, [comment_id]);
        } else {
            self.comments.suppress_comment_if_ordinary(comment_id);
        }
    }
}
