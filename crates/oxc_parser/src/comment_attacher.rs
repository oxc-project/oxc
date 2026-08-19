use std::{cell::UnsafeCell, ops::Range};

use oxc_allocator::{Allocator, HashMap as ArenaHashMap, Vec as ArenaVec};
use oxc_ast::{AstType, CommentId, CommentPosition, CommentStore, NodeComments};
use oxc_span::Span;
use oxc_syntax::node::NodeId;

struct PendingNodeComments<'a> {
    leading: ArenaVec<'a, CommentId>,
    trailing: ArenaVec<'a, CommentId>,
    dangling: ArenaVec<'a, CommentId>,
}

impl<'a> PendingNodeComments<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            leading: ArenaVec::new_in(&allocator),
            trailing: ArenaVec::new_in(&allocator),
            dangling: ArenaVec::new_in(&allocator),
        }
    }
}

struct Inner<'a> {
    allocator: &'a Allocator,
    comment_boundaries: ArenaVec<'a, Option<(CommentPosition, u32)>>,
    leading: ArenaHashMap<'a, u32, ArenaVec<'a, CommentId>>,
    leading_boundaries: ArenaVec<'a, u32>,
    trailing: ArenaHashMap<'a, u32, ArenaVec<'a, CommentId>>,
    leading_owners: ArenaHashMap<'a, u32, NodeId>,
    trailing_owners: ArenaHashMap<'a, u32, NodeId>,
    dangling_candidates: ArenaHashMap<'a, CommentId, (u32, NodeId)>,
    nodes: ArenaHashMap<'a, NodeId, PendingNodeComments<'a>>,
}

impl<'a> Inner<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            comment_boundaries: ArenaVec::new_in(&allocator),
            leading: ArenaHashMap::new_in(allocator),
            leading_boundaries: ArenaVec::new_in(&allocator),
            trailing: ArenaHashMap::new_in(allocator),
            leading_owners: ArenaHashMap::new_in(allocator),
            trailing_owners: ArenaHashMap::new_in(allocator),
            dangling_candidates: ArenaHashMap::new_in(allocator),
            nodes: ArenaHashMap::new_in(allocator),
        }
    }

    fn node_comments(&mut self, node_id: NodeId) -> &mut PendingNodeComments<'a> {
        let allocator = self.allocator;
        self.nodes.entry(node_id).or_insert_with(|| PendingNodeComments::new(allocator))
    }
}

/// Parser-side comment ownership shared by lexer trivia and AST construction.
///
/// Parser and lexer operations are sequential under `UniquePromise`; `UnsafeCell` avoids adding a
/// runtime borrow check to every token and AST node.
pub struct ParserCommentAttacher<'a> {
    inner: UnsafeCell<Inner<'a>>,
}

impl std::fmt::Debug for ParserCommentAttacher<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ParserCommentAttacher")
    }
}

impl<'a> ParserCommentAttacher<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { inner: UnsafeCell::new(Inner::new(allocator)) }
    }

    #[inline]
    fn inner(&self) -> &mut Inner<'a> {
        // SAFETY: No references into `Inner` escape a method call, and parser operations do not run
        // concurrently.
        unsafe { &mut *self.inner.get() }
    }

    pub fn record_comments(&self, position: CommentPosition, boundary: u32, range: Range<usize>) {
        if range.is_empty() {
            return;
        }
        let inner = self.inner();
        if inner.comment_boundaries.len() < range.end {
            inner.comment_boundaries.resize(range.end, None);
        }
        if position == CommentPosition::Leading
            && let Err(index) = inner.leading_boundaries.binary_search(&boundary)
        {
            inner.leading_boundaries.insert(index, boundary);
        }
        let allocator = inner.allocator;
        for index in range {
            let comment_id = CommentId::from_usize(index);
            inner.comment_boundaries[index] = Some((position, boundary));
            let target = match position {
                CommentPosition::Leading => &mut inner.leading,
                CommentPosition::Trailing => &mut inner.trailing,
            };
            let ids = target.entry(boundary).or_insert_with(|| ArenaVec::new_in(&allocator));
            if !ids.contains(&comment_id) {
                ids.push(comment_id);
            }
        }
    }

    pub fn node_needs_id(&self, span: Span, ty: AstType) -> bool {
        if matches!(ty, AstType::Program | AstType::Hashbang) {
            return false;
        }
        let inner = self.inner();
        if inner.leading.contains_key(&span.start)
            || inner.leading_owners.contains_key(&span.start)
            || inner.trailing.contains_key(&span.end)
        {
            return true;
        }
        matches!(ty, AstType::VariableDeclarator | AstType::CallExpression | AstType::NewExpression)
            && inner.leading_boundaries.iter().any(|&boundary| {
                boundary > span.start
                    && boundary < span.end
                    && inner.leading.contains_key(&boundary)
            })
    }

    pub fn finish_node(&self, node_id: NodeId, span: Span, ty: AstType) {
        if matches!(ty, AstType::Program | AstType::Hashbang) {
            return;
        }
        let inner = self.inner();

        let mut transferred = false;
        let leading = if let Some(ids) = inner.leading.remove(&span.start) {
            Some(ids)
        } else if let Some(previous_owner) = inner.leading_owners.get(&span.start).copied() {
            if inner.nodes.get(&previous_owner).is_some_and(|comments| {
                comments.trailing.is_empty() && comments.dangling.is_empty()
            }) {
                let comments = inner.nodes.remove(&previous_owner).unwrap();
                inner.nodes.insert(node_id, comments);
                transferred = true;
                None
            } else {
                let allocator = inner.allocator;
                inner.nodes.get_mut(&previous_owner).map(|comments| {
                    std::mem::replace(&mut comments.leading, ArenaVec::new_in(&allocator))
                })
            }
        } else {
            None
        };
        if let Some(ids) = leading
            && !ids.is_empty()
        {
            debug_assert!(inner.node_comments(node_id).leading.is_empty());
            inner.node_comments(node_id).leading = ids;
            inner.leading_owners.insert(span.start, node_id);
        } else if transferred {
            inner.leading_owners.insert(span.start, node_id);
        }

        if !inner.trailing_owners.contains_key(&span.end)
            && let Some(ids) = inner.trailing.remove(&span.end)
        {
            debug_assert!(inner.node_comments(node_id).trailing.is_empty());
            inner.node_comments(node_id).trailing = ids;
            inner.trailing_owners.insert(span.end, node_id);
        }

        if matches!(
            ty,
            AstType::VariableDeclarator | AstType::CallExpression | AstType::NewExpression
        ) {
            let span_len = span.size();
            let start =
                inner.leading_boundaries.partition_point(|&boundary| boundary <= span.start);
            let end = inner.leading_boundaries.partition_point(|&boundary| boundary < span.end);
            for boundary_index in start..end {
                let boundary = inner.leading_boundaries[boundary_index];
                let Some(comment_ids) = inner.leading.get(&boundary) else { continue };
                for &comment_id in comment_ids {
                    let candidate =
                        inner.dangling_candidates.entry(comment_id).or_insert((span_len, node_id));
                    if span_len < candidate.0 {
                        *candidate = (span_len, node_id);
                    }
                }
            }
        }
    }

    pub fn rewind_nodes(&self, first_removed: NodeId) {
        let inner = self.inner();
        while let Some(node_id) =
            inner.nodes.keys().find(|node_id| node_id.index() >= first_removed.index()).copied()
        {
            let Some(comments) = inner.nodes.remove(&node_id) else { continue };
            for comment_id in comments.leading.into_iter().chain(comments.trailing) {
                let Some((position, boundary)) = inner.comment_boundaries[comment_id.index()]
                else {
                    continue;
                };
                let target = match position {
                    CommentPosition::Leading => &mut inner.leading,
                    CommentPosition::Trailing => &mut inner.trailing,
                };
                target
                    .entry(boundary)
                    .or_insert_with(|| ArenaVec::new_in(&inner.allocator))
                    .push(comment_id);
            }
        }
        inner.leading_owners.retain(|_, node_id| node_id.index() < first_removed.index());
        inner.trailing_owners.retain(|_, node_id| node_id.index() < first_removed.index());
        inner.dangling_candidates.retain(|_, (_, node_id)| node_id.index() < first_removed.index());
    }

    pub fn finish(&self, comments: &CommentStore<'_>) {
        let inner = self.inner();
        let mut dangling =
            ArenaVec::with_capacity_in(inner.dangling_candidates.len(), &inner.allocator);
        dangling.extend(inner.dangling_candidates.iter().map(|(&id, &candidate)| (id, candidate)));
        for (comment_id, (_, node_id)) in dangling {
            if let Some((CommentPosition::Leading, boundary)) =
                inner.comment_boundaries[comment_id.index()]
                && inner.leading.get(&boundary).is_some_and(|ids| ids.contains(&comment_id))
            {
                inner.node_comments(node_id).dangling.push(comment_id);
            }
        }
        while let Some(node_id) = inner.nodes.keys().next().copied() {
            let node_comments = inner.nodes.remove(&node_id).unwrap();
            comments.attach_node_comments(
                node_id,
                NodeComments {
                    leading: node_comments.leading,
                    trailing: node_comments.trailing,
                    dangling: node_comments.dangling,
                },
            );
        }
    }
}
