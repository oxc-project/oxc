use std::ops::Range;

use oxc_ast::{AstKind, Comment, ast::Program};
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;

use crate::Visit;

/// Placement of a source comment relative to its owning AST node.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommentPlacement {
    /// Print before the owning node.
    Before,
    /// Print after the owning node.
    After,
    /// Print inside a childless owning node.
    Inside,
}

/// A source comment index and its placement relative to its owning AST node.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AttachedComment {
    /// Index into the parser-produced `Program.comments` slice.
    pub comment_index: u32,
    /// Placement relative to the owning node.
    pub placement: CommentPlacement,
    /// Whether the comment begins on the same line as the preceding token.
    pub same_line: bool,
}

impl AttachedComment {
    /// Resolve this attachment against the parser-produced comments from which
    /// its sidecar was built.
    ///
    /// # Panics
    ///
    /// Panics if `comments` is not the original comment slice or has since been
    /// shortened.
    #[inline]
    pub fn comment(self, comments: &[Comment]) -> &Comment {
        &comments[self.comment_index as usize]
    }
}

/// Immutable mapping from dense [`NodeId`]s to their source comments.
///
/// The mapping is a separate sidecar: callers deliberately carry it with the
/// AST through transformations and pass it to consumers such as codegen.
#[derive(Debug, Eq, PartialEq)]
pub struct CommentAttachments {
    node_count: usize,
    host_presence: Box<[u64]>,
    hosts: Box<[CommentHost]>,
    comments: Box<[AttachedComment]>,
}

impl CommentAttachments {
    /// Create an empty sidecar for an AST containing `node_count` nodes.
    pub fn empty(node_count: usize) -> Self {
        Self {
            node_count,
            host_presence: Box::new([]),
            hosts: Box::new([]),
            comments: Box::new([]),
        }
    }

    /// Number of AST nodes addressable by this sidecar.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Number of attached comments.
    #[inline]
    pub fn len(&self) -> usize {
        self.comments.len()
    }

    /// Returns `true` when no comments are attached.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    /// Returns whether `node_id` owns any comments.
    #[inline]
    pub fn has_comments(&self, node_id: NodeId) -> bool {
        let index = node_id.index();
        index < self.node_count
            && self
                .host_presence
                .get(index / u64::BITS as usize)
                .is_some_and(|word| word & (1 << (index % u64::BITS as usize)) != 0)
    }

    /// Returns comments owned by `node_id`, in original source order.
    #[inline]
    pub fn comments_for(&self, node_id: NodeId) -> &[AttachedComment] {
        self.comments_for_with_range(node_id).map_or(&[], |(_, comments)| comments)
    }

    /// Returns the storage range and comments owned by `node_id`.
    ///
    /// This supports consumers which keep compact per-comment claim state
    /// alongside the immutable attachment mapping.
    #[doc(hidden)]
    #[inline]
    pub fn comments_for_with_range(
        &self,
        node_id: NodeId,
    ) -> Option<(Range<usize>, &[AttachedComment])> {
        if !self.has_comments(node_id) {
            return None;
        }
        let Ok(host_index) = self.hosts.binary_search_by_key(&node_id, |host| host.node_id) else {
            debug_assert!(false, "presence bit must have a corresponding comment host");
            return None;
        };
        let range = self.hosts[host_index].comment_range.clone();
        let range = range.start as usize..range.end as usize;
        Some((range.clone(), &self.comments[range]))
    }

    /// Returns attached comments in a range previously returned by
    /// [`Self::comments_for_with_range`].
    #[doc(hidden)]
    #[inline]
    pub fn comments_in_range(&self, range: Range<usize>) -> &[AttachedComment] {
        &self.comments[range]
    }

    /// Number of comments which still have active hosts.
    #[doc(hidden)]
    #[inline]
    pub fn active_comment_count(&self) -> usize {
        self.comments.len()
    }

    /// Presence bits for active comment hosts.
    #[doc(hidden)]
    #[inline]
    pub fn host_presence(&self) -> &[u64] {
        &self.host_presence
    }

    /// Rehome active comments after an AST traversal has reassigned node IDs.
    ///
    /// Each pair is `(old_id, new_id)`. When an old host appears more than
    /// once, only its first mapping is used, so cloned nodes do not duplicate
    /// comments. Comments on hosts without a mapping are discarded.
    #[doc(hidden)]
    pub fn remap_node_ids(&mut self, remaps: &[(NodeId, NodeId)], node_count: usize) {
        if remaps.len() == self.hosts.len()
            && remaps
                .iter()
                .zip(&self.hosts)
                .all(|(&(old_id, new_id), host)| old_id == host.node_id && old_id == new_id)
        {
            self.node_count = node_count;
            return;
        }

        let mut remaps = remaps.to_vec();
        remaps.sort_by_key(|(old_id, _)| *old_id);
        remaps.dedup_by_key(|(old_id, _)| *old_id);

        let mut grouped = Vec::with_capacity(self.comments.len());
        let mut remap_index = 0;
        for host in &self.hosts {
            while remap_index < remaps.len() && remaps[remap_index].0 < host.node_id {
                remap_index += 1;
            }
            let range = host.comment_range.start as usize..host.comment_range.end as usize;
            if remap_index < remaps.len() && remaps[remap_index].0 == host.node_id {
                let new_id = remaps[remap_index].1;
                grouped.extend(self.comments[range].iter().map(|&comment| (new_id, comment)));
            }
        }

        *self = Self::from_grouped(node_count, grouped);
    }

    fn from_grouped(node_count: usize, mut grouped: Vec<(NodeId, AttachedComment)>) -> Self {
        grouped.sort_by_key(|(host_id, _)| *host_id);

        let mut host_presence = if grouped.is_empty() {
            Vec::new()
        } else {
            vec![0_u64; node_count.div_ceil(u64::BITS as usize)]
        };
        let mut hosts = Vec::new();
        let mut comments = Vec::with_capacity(grouped.len());
        let mut grouped_index = 0;
        while grouped_index < grouped.len() {
            let host_id = grouped[grouped_index].0;
            let start = comments.len();
            while grouped_index < grouped.len() && grouped[grouped_index].0 == host_id {
                comments.push(grouped[grouped_index].1);
                grouped_index += 1;
            }
            let end = comments.len();
            debug_assert!(host_id.index() < node_count);
            #[expect(clippy::cast_possible_truncation)]
            hosts.push(CommentHost {
                node_id: host_id,
                comment_range: (start as u32)..(end as u32),
            });
            let host_index = host_id.index();
            host_presence[host_index / u64::BITS as usize] |=
                1 << (host_index % u64::BITS as usize);
        }

        Self {
            node_count,
            host_presence: host_presence.into_boxed_slice(),
            hosts: hosts.into_boxed_slice(),
            comments: comments.into_boxed_slice(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CommentHost {
    node_id: NodeId,
    comment_range: Range<u32>,
}

/// Builds comment attachments without performing semantic analysis.
///
/// The builder assigns dense [`NodeId`]s in the same traversal order as
/// semantic analysis, then gives every parser-produced comment exactly one
/// owner using [`CommentPlacement::Before`], [`CommentPlacement::After`], or
/// [`CommentPlacement::Inside`].
pub struct CommentAttachmentBuilder;

impl CommentAttachmentBuilder {
    /// Assign node IDs and build a separate attachment sidecar for `program`.
    pub fn build(program: &Program<'_>) -> CommentAttachments {
        let mut collector = StandaloneAttachmentCollector::new(&program.comments);
        collector.visit_program(program);
        collector.finish()
    }
}

struct StandaloneAttachmentCollector<'c> {
    collector: CommentAttachmentCollector<'c>,
    node_count: u32,
}

impl<'c> StandaloneAttachmentCollector<'c> {
    fn new(comments: &'c [Comment]) -> Self {
        Self { collector: CommentAttachmentCollector::new(comments), node_count: 0 }
    }

    fn finish(self) -> CommentAttachments {
        self.collector.finish(self.node_count as usize)
    }
}

impl<'a> Visit<'a> for StandaloneAttachmentCollector<'_> {
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        let node_id = NodeId::new(self.node_count as usize);
        self.node_count += 1;
        kind.set_node_id(node_id);
        self.collector.enter_node(kind);
    }

    #[inline]
    fn leave_node(&mut self, _: AstKind<'a>) {
        self.collector.leave_node();
    }
}

/// Collects AST structure for comment attachment while another pass assigns
/// dense [`NodeId`]s.
///
/// This is primarily used to share attachment collection with semantic
/// traversal. Most callers should use [`CommentAttachmentBuilder`] instead.
#[doc(hidden)]
pub struct CommentAttachmentCollector<'c> {
    comments: &'c [Comment],
    #[cfg(debug_assertions)]
    next_node_id: usize,
    depth: usize,
    relevant_nodes: Vec<RelevantNode>,
    relevant_stack: Vec<(usize, usize)>,
}

impl<'c> CommentAttachmentCollector<'c> {
    /// Create a collector for parser-produced `comments`.
    pub fn new(comments: &'c [Comment]) -> Self {
        debug_assert!(comments.windows(2).all(|pair| pair[0].span.end <= pair[1].span.start));
        Self {
            comments,
            #[cfg(debug_assertions)]
            next_node_id: 0,
            depth: 0,
            relevant_nodes: Vec::new(),
            relevant_stack: Vec::new(),
        }
    }

    /// Record entry into a node after its dense [`NodeId`] has been assigned.
    pub fn enter_node(&mut self, kind: AstKind<'_>) {
        let node_id = kind.node_id();
        #[cfg(debug_assertions)]
        {
            debug_assert_eq!(node_id.index(), self.next_node_id);
            self.next_node_id += 1;
        }

        let comment_range = if node_id == NodeId::ROOT {
            0..self.comments.len()
        } else if let Some(&(parent_index, parent_depth)) = self.relevant_stack.last()
            && parent_depth + 1 == self.depth
        {
            self.relevant_nodes[parent_index].record_child(node_id, kind.span(), self.comments)
        } else {
            0..0
        };
        if !comment_range.is_empty() {
            let index = self.relevant_nodes.len();
            self.relevant_nodes.push(RelevantNode {
                node_id,
                next_comment_index: comment_range.start,
                comment_range,
                previous_child_end: 0,
                has_children: false,
                boundaries: None,
            });
            self.relevant_stack.push((index, self.depth));
        }
        self.depth += 1;
    }

    /// Record exit from a node.
    pub fn leave_node(&mut self) {
        let Some(depth) = self.depth.checked_sub(1) else {
            debug_assert!(false, "node stack must be balanced");
            return;
        };
        self.depth = depth;
        if self.relevant_stack.last().is_some_and(|&(_, node_depth)| node_depth == depth) {
            let popped = self.relevant_stack.pop();
            debug_assert!(popped.is_some(), "relevant node stack must be balanced");
        }
    }

    /// Finish collection and assign exactly one owner to every comment.
    ///
    /// # Panics
    ///
    /// Panics if the recorded AST structure cannot provide an owner for every
    /// parser-produced comment.
    pub fn finish(self, node_count: usize) -> CommentAttachments {
        debug_assert_eq!(self.depth, 0);
        debug_assert!(self.relevant_stack.is_empty());
        debug_assert!(node_count > 0);
        #[cfg(debug_assertions)]
        debug_assert_eq!(node_count, self.next_node_id);

        let mut assignments = vec![None; self.comments.len()];
        assign_comments(self.comments, &self.relevant_nodes, &mut assignments);

        let mut grouped = Vec::with_capacity(self.comments.len());
        for (comment_index, (comment, assignment)) in
            self.comments.iter().copied().zip(assignments).enumerate()
        {
            let assignment = assignment.expect("every source comment must have an owner");
            #[expect(clippy::cast_possible_truncation)]
            let comment_index = comment_index as u32;
            grouped.push((
                assignment.host_id,
                AttachedComment {
                    comment_index,
                    placement: assignment.placement,
                    same_line: !comment.preceded_by_newline(),
                },
            ));
        }
        CommentAttachments::from_grouped(node_count, grouped)
    }
}

#[derive(Debug)]
struct RelevantNode {
    node_id: NodeId,
    comment_range: Range<usize>,
    next_comment_index: usize,
    previous_child_end: u32,
    has_children: bool,
    boundaries: Option<Box<BoundaryCandidates>>,
}

impl RelevantNode {
    fn record_child(
        &mut self,
        child_id: NodeId,
        child_span: Span,
        comments: &[Comment],
    ) -> Range<usize> {
        self.has_children = true;
        let child_comment_range = if child_span.start >= self.previous_child_end {
            let mut start = self.next_comment_index;
            while start < self.comment_range.end && comments[start].span.end <= child_span.start {
                start += 1;
            }
            let mut end = start;
            while end < self.comment_range.end && comments[end].span.start < child_span.end {
                end += 1;
            }
            self.next_comment_index = end;
            start..end
        } else {
            // Parsed AST children are normally visited in source order. Keep a
            // correctness fallback for exceptional overlapping or reordered spans.
            let range = comments_inside_span(&comments[self.comment_range.clone()], child_span);
            (self.comment_range.start + range.start)..(self.comment_range.start + range.end)
        };
        self.previous_child_end = self.previous_child_end.max(child_span.end);

        if child_comment_range.end < self.comment_range.end {
            self.boundaries
                .get_or_insert_with(|| Box::new(BoundaryCandidates::default()))
                .record_previous(child_comment_range.end, child_id, child_span);
        }

        if child_comment_range.start > self.comment_range.start {
            self.boundaries
                .get_or_insert_with(|| Box::new(BoundaryCandidates::default()))
                .record_next(child_comment_range.start - 1, child_id, child_span);
        }

        child_comment_range
    }
}

#[derive(Debug, Default)]
struct BoundaryCandidates {
    previous: Vec<(usize, ChildCandidate)>,
    next: Vec<(usize, ChildCandidate)>,
}

impl BoundaryCandidates {
    fn record_previous(&mut self, comment_index: usize, node_id: NodeId, span: Span) {
        record_candidate(
            &mut self.previous,
            comment_index,
            ChildCandidate { node_id, boundary: span.end },
            |new, current| new.boundary > current.boundary,
        );
    }

    fn record_next(&mut self, comment_index: usize, node_id: NodeId, span: Span) {
        record_candidate(
            &mut self.next,
            comment_index,
            ChildCandidate { node_id, boundary: span.start },
            |new, current| new.boundary < current.boundary,
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct ChildCandidate {
    node_id: NodeId,
    boundary: u32,
}

#[derive(Debug, Clone, Copy)]
struct Assignment {
    host_id: NodeId,
    placement: CommentPlacement,
}

fn assign_comments(
    comments: &[Comment],
    nodes: &[RelevantNode],
    assignments: &mut [Option<Assignment>],
) {
    if comments.is_empty() {
        return;
    }

    let containers = deepest_containers(comments.len(), nodes);
    let mut owned_comments = containers.into_iter().enumerate().collect::<Vec<_>>();
    owned_comments.sort_by_key(|(_, node_index)| *node_index);

    let mut start = 0;
    while start < owned_comments.len() {
        let node_index = owned_comments[start].1;
        let mut end = start + 1;
        while end < owned_comments.len() && owned_comments[end].1 == node_index {
            end += 1;
        }
        assign_owned_comments(
            comments,
            &nodes[node_index],
            &owned_comments[start..end],
            assignments,
        );
        start = end;
    }
}

fn deepest_containers(comment_count: usize, nodes: &[RelevantNode]) -> Vec<usize> {
    let mut containers = vec![usize::MAX; comment_count];
    let mut next_unassigned = (0..=comment_count).collect::<Vec<_>>();

    // Relevant nodes are recorded in preorder, so descendants precede their
    // ancestors here. The successor structure skips comments already claimed
    // by a deeper node, making each comment assignment effectively constant-time.
    for (node_index, node) in nodes.iter().enumerate().rev() {
        let mut comment_index =
            find_next_unassigned(&mut next_unassigned, node.comment_range.start);
        while comment_index < node.comment_range.end {
            containers[comment_index] = node_index;
            let next = find_next_unassigned(&mut next_unassigned, comment_index + 1);
            next_unassigned[comment_index] = next;
            comment_index = next;
        }
    }

    debug_assert!(containers.iter().all(|container| *container != usize::MAX));
    containers
}

fn find_next_unassigned(next_unassigned: &mut [usize], index: usize) -> usize {
    let mut root = index;
    while next_unassigned[root] != root {
        root = next_unassigned[root];
    }

    let mut current = index;
    while next_unassigned[current] != current {
        let next = next_unassigned[current];
        next_unassigned[current] = root;
        current = next;
    }
    root
}

fn assign_owned_comments(
    comments: &[Comment],
    node: &RelevantNode,
    owned_comments: &[(usize, usize)],
    assignments: &mut [Option<Assignment>],
) {
    if !node.has_children {
        for &(comment_index, _) in owned_comments {
            assignments[comment_index] =
                Some(Assignment { host_id: node.node_id, placement: CommentPlacement::Inside });
        }
        return;
    }

    let Some(boundaries) = &node.boundaries else {
        debug_assert!(false, "a directly owned comment must have an adjacent child");
        for &(comment_index, _) in owned_comments {
            assignments[comment_index] =
                Some(Assignment { host_id: node.node_id, placement: CommentPlacement::Inside });
        }
        return;
    };

    let mut previous = None;
    let mut previous_updates = boundaries.previous.iter().peekable();
    let mut next_update = 0;
    for &(comment_index, _) in owned_comments {
        while previous_updates.peek().is_some_and(|(index, _)| *index <= comment_index) {
            previous = previous_updates.next().map(|(_, candidate)| candidate.node_id);
        }
        while next_update < boundaries.next.len() && boundaries.next[next_update].0 < comment_index
        {
            next_update += 1;
        }
        let next = boundaries.next.get(next_update).map(|(_, candidate)| candidate.node_id);
        assignments[comment_index] =
            Some(gap_assignment(comments[comment_index], node.node_id, previous, next));
    }
}

fn comments_inside_span(comments: &[Comment], span: Span) -> Range<usize> {
    let start = comments.partition_point(|comment| comment.span.end <= span.start);
    let end = comments.partition_point(|comment| comment.span.start < span.end);
    start..end
}

fn record_candidate(
    candidates: &mut Vec<(usize, ChildCandidate)>,
    comment_index: usize,
    candidate: ChildCandidate,
    is_better: impl FnOnce(ChildCandidate, ChildCandidate) -> bool,
) {
    match candidates.binary_search_by_key(&comment_index, |(index, _)| *index) {
        Ok(index) => {
            if is_better(candidate, candidates[index].1) {
                candidates[index].1 = candidate;
            }
        }
        Err(index) => candidates.insert(index, (comment_index, candidate)),
    }
}

fn gap_assignment(
    comment: Comment,
    parent_id: NodeId,
    previous_child: Option<NodeId>,
    next_child: Option<NodeId>,
) -> Assignment {
    match (previous_child, next_child) {
        (None, None) => Assignment { host_id: parent_id, placement: CommentPlacement::Inside },
        (None, Some(next_id)) => {
            Assignment { host_id: next_id, placement: CommentPlacement::Before }
        }
        (Some(previous_id), None) => {
            Assignment { host_id: previous_id, placement: CommentPlacement::After }
        }
        (Some(previous_id), Some(next_id)) => {
            if comment.is_trailing() && !comment.preceded_by_newline() {
                Assignment { host_id: previous_id, placement: CommentPlacement::After }
            } else {
                Assignment { host_id: next_id, placement: CommentPlacement::Before }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::{AstType, CommentKind, ast::Program};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::*;

    #[test]
    fn attached_comment_is_compact() {
        assert_eq!(size_of::<AttachedComment>(), 8);
    }

    #[test]
    fn remaps_surviving_hosts_and_discards_removed_hosts() {
        let attached = |comment_index| AttachedComment {
            comment_index,
            placement: CommentPlacement::Before,
            same_line: false,
        };
        let mut attachments = CommentAttachments::from_grouped(
            3,
            vec![(NodeId::new(1), attached(0)), (NodeId::new(2), attached(1))],
        );

        attachments.remap_node_ids(
            &[
                (NodeId::new(1), NodeId::new(2)),
                // A cloned host must not duplicate its comments.
                (NodeId::new(1), NodeId::new(3)),
            ],
            4,
        );

        assert!(attachments.comments_for(NodeId::new(1)).is_empty());
        assert_eq!(attachments.comments_for(NodeId::new(2)), [attached(0)]);
        assert!(attachments.comments_for(NodeId::new(3)).is_empty());
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments.node_count(), 4);
    }

    #[test]
    fn falls_back_for_out_of_order_child_spans() {
        let comments = [
            Comment::new(10, 15, CommentKind::SingleLineBlock),
            Comment::new(30, 35, CommentKind::SingleLineBlock),
        ];
        let mut node = RelevantNode {
            node_id: NodeId::ROOT,
            comment_range: 0..comments.len(),
            next_comment_index: 0,
            previous_child_end: 0,
            has_children: false,
            boundaries: None,
        };

        assert_eq!(node.record_child(NodeId::new(1), Span::new(20, 25), &comments), 1..1);
        assert_eq!(node.record_child(NodeId::new(2), Span::new(5, 18), &comments), 0..1);
    }

    #[test]
    fn assigns_each_comment_to_its_deepest_container() {
        let node = |id, comment_range| RelevantNode {
            node_id: NodeId::new(id),
            comment_range,
            next_comment_index: 0,
            previous_child_end: 0,
            has_children: false,
            boundaries: None,
        };
        let nodes = [node(0, 0..4), node(1, 1..4), node(2, 2..3), node(3, 3..4)];

        assert_eq!(deepest_containers(4, &nodes), [0, 1, 2, 3]);
    }

    #[test]
    fn attaches_comments_to_structural_hosts() {
        let allocator = Allocator::default();
        let source = concat!(
            "/*a*/ const x = [/*b*/ 1 /*c*/, /*d*/]; //e\n",
            "function /*f*/ name(/*g*/) { /*h*/ }\n",
        );
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty());

        let attachments = CommentAttachmentBuilder::build(&parsed.program);
        let actual = collect_attachments(source, &parsed.program, &attachments);

        assert_eq!(attachments.len(), parsed.program.comments.len());
        assert_eq!(
            actual,
            [
                ("/*a*/", AstType::VariableDeclaration, CommentPlacement::Before),
                ("/*b*/", AstType::NumericLiteral, CommentPlacement::Before),
                ("/*c*/", AstType::NumericLiteral, CommentPlacement::After),
                ("/*d*/", AstType::NumericLiteral, CommentPlacement::After),
                ("//e", AstType::VariableDeclaration, CommentPlacement::After),
                ("/*f*/", AstType::BindingIdentifier, CommentPlacement::Before),
                ("/*g*/", AstType::FormalParameters, CommentPlacement::Inside),
                ("/*h*/", AstType::FunctionBody, CommentPlacement::Inside),
            ]
        );
    }

    #[test]
    fn attaches_a_comment_only_program_to_program() {
        let allocator = Allocator::default();
        let source = "/* only */";
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty());

        let attachments = CommentAttachmentBuilder::build(&parsed.program);
        let actual = collect_attachments(source, &parsed.program, &attachments);

        assert_eq!(actual, [("/* only */", AstType::Program, CommentPlacement::Inside)]);
    }

    #[test]
    fn retains_only_comment_relevant_nodes() {
        let allocator = Allocator::default();
        let source =
            format!("{}/* marker */\n{}", "before();\n".repeat(1_000), "after();\n".repeat(1_000));
        let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty());

        let mut collector = StandaloneAttachmentCollector::new(&parsed.program.comments);
        collector.visit_program(&parsed.program);

        assert!(collector.node_count > 4_000);
        assert_eq!(collector.collector.relevant_nodes.len(), 1);
        assert_eq!(collector.collector.relevant_nodes[0].node_id, NodeId::ROOT);

        let node_count = collector.node_count as usize;
        let attachments = collector.finish();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments.hosts.len(), 1);
        assert_eq!(attachments.host_presence.len(), node_count.div_ceil(u64::BITS as usize));
    }

    fn collect_attachments<'a>(
        source: &'a str,
        program: &'a Program<'a>,
        attachments: &'a CommentAttachments,
    ) -> Vec<(&'a str, AstType, CommentPlacement)> {
        let mut collector = AttachmentHostCollector {
            source,
            source_comments: &program.comments,
            attachments,
            comments: Vec::new(),
        };
        collector.visit_program(program);
        collector.comments.sort_unstable_by_key(|(comment, _, _)| {
            comment.as_ptr() as usize - source.as_ptr() as usize
        });
        collector.comments
    }

    struct AttachmentHostCollector<'a> {
        source: &'a str,
        source_comments: &'a [Comment],
        attachments: &'a CommentAttachments,
        comments: Vec<(&'a str, AstType, CommentPlacement)>,
    }

    impl<'a> Visit<'a> for AttachmentHostCollector<'a> {
        fn enter_node(&mut self, kind: AstKind<'a>) {
            for attached in self.attachments.comments_for(kind.node_id()) {
                let span = attached.comment(self.source_comments).span;
                let text = &self.source[span.start as usize..span.end as usize];
                self.comments.push((text, kind.ty(), attached.placement));
            }
        }
    }
}
