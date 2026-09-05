//! Semantic source-comment attachment.

use std::cmp::Reverse;

use oxc_ast::{
    AstKind, AstType, AttachedComment, AttachedCommentPosition, CommentAttachmentHost,
    CommentAttachments, CommentContent, ast::*,
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{line_terminator::is_line_terminator, node::NodeId, scope::ScopeFlags};

use crate::Visit;

const NO_NODE: u32 = u32::MAX;

#[derive(Debug)]
struct NodeRecord {
    node_id: NodeId,
    kind: AstType,
    parent: u32,
    span: Span,
    depth: u32,
    first_child: u32,
    next_sibling: u32,
}

pub struct CommentAttachmentCollector<'c> {
    comments: &'c [oxc_ast::Comment],
    anchors: std::vec::Vec<u32>,
    nodes: std::vec::Vec<NodeRecord>,
    stack: std::vec::Vec<u32>,
}

macro_rules! comment_pruning_visit_method {
    ($visit:ident, $walk:ident, $kind:ident, $ty:ty $(, $arg:ident: $arg_ty:ty)*) => {
        #[inline]
        fn $visit(&mut self, it: &$ty $(, $arg: $arg_ty)*) {
            let kind = AstKind::$kind(self.alloc(it));
            if self.intersects_comment(kind.span()) {
                crate::walk::$walk(self, it $(, $arg)*);
            } else {
                self.enter_node(kind);
                self.leave_node(kind);
            }
        }
    };
}

impl<'c> CommentAttachmentCollector<'c> {
    pub fn new(comments: &'c [oxc_ast::Comment]) -> Self {
        let mut anchors =
            comments.iter().map(|comment| comment.attached_to).collect::<std::vec::Vec<_>>();
        anchors.sort_unstable();
        anchors.dedup();
        Self { comments, anchors, nodes: std::vec::Vec::new(), stack: std::vec::Vec::new() }
    }

    fn intersects_comment(&self, span: Span) -> bool {
        let comment_index =
            self.comments.partition_point(|comment| comment.span.start < span.start);
        if self.comments.get(comment_index).is_some_and(|comment| comment.span.start < span.end) {
            return true;
        }
        let anchor_index = self.anchors.partition_point(|&anchor| anchor < span.start);
        self.anchors.get(anchor_index).is_some_and(|&anchor| anchor <= span.end)
    }
}

impl<'a> Visit<'a> for CommentAttachmentCollector<'_> {
    crate::generate_comment_pruning_visit_methods!();

    fn enter_node(&mut self, kind: AstKind<'a>) {
        let index = u32::try_from(self.nodes.len()).unwrap();
        let parent = self.stack.last().copied().unwrap_or(NO_NODE);
        let next_sibling = if parent == NO_NODE {
            NO_NODE
        } else {
            let parent_index = parent as usize;
            let first_child = self.nodes[parent_index].first_child;
            self.nodes[parent_index].first_child = index;
            first_child
        };
        self.nodes.push(NodeRecord {
            node_id: kind.node_id(),
            kind: kind.ty(),
            parent,
            span: kind.span(),
            depth: u32::try_from(self.stack.len()).unwrap(),
            first_child: NO_NODE,
            next_sibling,
        });
        self.stack.push(index);
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.stack.pop();
    }
}

#[derive(Clone, Copy)]
struct Assignment {
    host: u32,
    position: AttachedCommentPosition,
    same_line: bool,
    force_newline_after: bool,
}

impl Assignment {
    const UNASSIGNED: Self = Self {
        host: NO_NODE,
        position: AttachedCommentPosition::Before,
        same_line: false,
        force_newline_after: false,
    };
}

struct Attacher<'a> {
    source_text: &'a str,
    nodes: &'a [NodeRecord],
    comments: &'a [oxc_ast::Comment],
    assignments: std::vec::Vec<Assignment>,
    scratch: std::vec::Vec<usize>,
    cursor: usize,
}

impl<'a> Attacher<'a> {
    fn new(
        source_text: &'a str,
        nodes: &'a [NodeRecord],
        comments: &'a [oxc_ast::Comment],
    ) -> Self {
        Self {
            source_text,
            nodes,
            comments,
            assignments: vec![Assignment::UNASSIGNED; comments.len()],
            scratch: std::vec::Vec::with_capacity(32),
            cursor: 0,
        }
    }

    fn attach(mut self) -> std::vec::Vec<Assignment> {
        self.attach_pure_not_applied();
        if !self.nodes.is_empty() {
            self.walk_at(0);
        }
        while self.cursor < self.comments.len() {
            if self.assignments[self.cursor].host == NO_NODE {
                self.assignments[self.cursor] = Assignment {
                    host: 0,
                    position: AttachedCommentPosition::Inside,
                    same_line: false,
                    force_newline_after: false,
                };
            }
            self.cursor += 1;
        }
        self.assignments
    }

    /// `PureNotApplied` is the one deliberate parser-attachment override. It
    /// remains trailing on the deepest node ending at the parser's token
    /// boundary, e.g. `foo /* PURE */ = call()`.
    fn attach_pure_not_applied(&mut self) {
        for (comment_index, comment) in self.comments.iter().enumerate() {
            if comment.content != CommentContent::PureNotApplied {
                continue;
            }
            let host = self
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.span.end == comment.attached_to)
                .max_by_key(|(_, node)| (node.depth, Reverse(node.span.size())))
                .map_or(0, |(index, _)| index);
            self.assignments[comment_index] = Assignment {
                host: u32::try_from(host).unwrap(),
                position: AttachedCommentPosition::After,
                same_line: self.same_line(self.nodes[host].span.end, comment.span.start),
                force_newline_after: false,
            };
        }
    }

    fn walk_at(&mut self, node_index: usize) {
        if self.cursor >= self.comments.len() {
            return;
        }
        let node_span = self.nodes[node_index].span;
        if self.comments[self.cursor].span.start >= node_span.end {
            return;
        }

        let checkpoint = self.scratch.len();
        let mut child = self.nodes[node_index].first_child;
        while child != NO_NODE {
            self.scratch.push(child as usize);
            child = self.nodes[child as usize].next_sibling;
        }
        // Children are linked by prepending during the visitor walk. Restore
        // visitation order so the insertion sort below sees the overwhelmingly
        // common already-source-ordered case instead of its quadratic worst case.
        self.scratch[checkpoint..].reverse();
        self.sort_scratch_children(checkpoint);

        let mut previous = None;
        let mut previous_end = node_span.start;
        let child_count = self.scratch.len() - checkpoint;
        for offset in 0..child_count {
            let child = self.scratch[checkpoint + offset];
            let child_start = self.nodes[child].span.start;
            self.consume_between(node_index, previous, previous_end, Some(child), child_start);
            self.walk_at(child);
            previous = Some(child);
            previous_end = self.nodes[child].span.end;
        }
        self.consume_between(node_index, previous, previous_end, None, node_span.end);
        self.scratch.truncate(checkpoint);
    }

    fn consume_between(
        &mut self,
        host: usize,
        previous: Option<usize>,
        previous_end: u32,
        next: Option<usize>,
        next_start: u32,
    ) {
        while self.cursor < self.comments.len() {
            if self.assignments[self.cursor].host != NO_NODE {
                self.cursor += 1;
                continue;
            }
            let comment = self.comments[self.cursor];
            if comment.span.start >= next_start {
                return;
            }

            let assignment = match (previous, next) {
                (Some(previous), Some(next)) => {
                    if self.same_line(comment.span.end, next_start) {
                        Assignment {
                            host: u32::try_from(next).unwrap(),
                            position: AttachedCommentPosition::Before,
                            same_line: true,
                            force_newline_after: false,
                        }
                    } else if self.same_line(previous_end, comment.span.start) {
                        Assignment {
                            host: u32::try_from(previous).unwrap(),
                            position: AttachedCommentPosition::After,
                            same_line: true,
                            force_newline_after: true,
                        }
                    } else {
                        Assignment {
                            host: u32::try_from(next).unwrap(),
                            position: AttachedCommentPosition::Before,
                            same_line: false,
                            force_newline_after: false,
                        }
                    }
                }
                (None, Some(next)) => Assignment {
                    host: u32::try_from(next).unwrap(),
                    position: AttachedCommentPosition::Before,
                    same_line: self.same_line(comment.span.end, next_start),
                    force_newline_after: false,
                },
                (Some(previous), None) => Assignment {
                    host: u32::try_from(previous).unwrap(),
                    position: AttachedCommentPosition::After,
                    same_line: self.same_line(previous_end, comment.span.start),
                    force_newline_after: false,
                },
                (None, None) => Assignment {
                    host: u32::try_from(host).unwrap(),
                    position: AttachedCommentPosition::Inside,
                    same_line: false,
                    force_newline_after: false,
                },
            };
            self.assignments[self.cursor] = assignment;
            self.cursor += 1;
        }
    }

    fn same_line(&self, a: u32, b: u32) -> bool {
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        let Ok(start) = usize::try_from(start) else { return false };
        let Ok(end) = usize::try_from(end) else { return false };
        self.source_text.get(start..end).is_some_and(|text| !text.chars().any(is_line_terminator))
    }

    fn sort_scratch_children(&mut self, start: usize) {
        for index in start + 1..self.scratch.len() {
            let child = self.scratch[index];
            let child_span = self.nodes[child].span;
            let mut insertion = index;
            while insertion > start {
                let previous = self.scratch[insertion - 1];
                let previous_span = self.nodes[previous].span;
                if (previous_span.start, previous_span.end, previous)
                    <= (child_span.start, child_span.end, child)
                {
                    break;
                }
                self.scratch[insertion] = previous;
                insertion -= 1;
            }
            self.scratch[insertion] = child;
        }
    }
}

/// Assign every parser comment to an AST host after semantic NodeIds are available.
impl CommentAttachmentCollector<'_> {
    /// # Panics
    ///
    /// Panics if comment or node counts exceed the sidecar's `u32` indices.
    pub fn attach<'a>(self, program: &Program<'a>, attachments: &CommentAttachments<'a>) {
        attachments.clear();
        if program.comments.is_empty() {
            return;
        }

        let mut assignments =
            Attacher::new(program.source_text, &self.nodes, &program.comments).attach();
        let mut function_header_comments = vec![false; program.comments.len()];
        for function in &self.nodes {
            if function.kind != AstType::Function {
                continue;
            }
            let mut child = function.first_child;
            let mut params = None;
            while child != NO_NODE {
                if self.nodes[child as usize].kind == AstType::FormalParameters {
                    params = Some(child);
                    break;
                }
                child = self.nodes[child as usize].next_sibling;
            }
            let Some(params) = params else { continue };
            let params_start = self.nodes[params as usize].span.start;
            let first = program
                .comments
                .partition_point(|comment| comment.span.start < function.span.start);
            let last =
                program.comments.partition_point(|comment| comment.span.start < params_start);
            for comment_index in first..last {
                assignments[comment_index].host = params;
                assignments[comment_index].position = AttachedCommentPosition::Before;
                function_header_comments[comment_index] = true;
            }
        }

        let mut offsets = vec![0_u32; self.nodes.len() + 1];
        for assignment in &assignments {
            offsets[assignment.host as usize + 1] += 1;
        }
        for index in 0..self.nodes.len() {
            offsets[index + 1] += offsets[index];
        }
        let mut write_offsets = offsets[..self.nodes.len()].to_vec();
        let mut sorted = vec![AttachedComment::default(); program.comments.len()];
        let mut source_only_hosts = vec![false; self.nodes.len()];
        for (comment, assignment) in program.comments.iter().zip(&assignments) {
            if comment.is_line() || assignment.position == AttachedCommentPosition::Inside {
                source_only_hosts[assignment.host as usize] = true;
            }
        }
        for (comment_index, (mut comment, assignment)) in
            program.comments.iter().copied().zip(assignments).enumerate()
        {
            if assignment.force_newline_after {
                comment.set_followed_by_newline(true);
            }
            let host = assignment.host as usize;
            let node = &self.nodes[host];
            let exact_leading = comment.is_leading() && comment.attached_to == node.span.start;
            let owned_composite_gap = node.kind == AstType::FormalParameters
                || (node.kind == AstType::Function
                    && node.parent != NO_NODE
                    && matches!(
                        self.nodes[node.parent as usize].kind,
                        AstType::ObjectProperty | AstType::MethodDefinition
                    ));
            let owned_position = match assignment.position {
                AttachedCommentPosition::Before => exact_leading || owned_composite_gap,
                AttachedCommentPosition::After => {
                    comment.is_block()
                        && matches!(node.kind, AstType::FormalParameter | AstType::ObjectExpression)
                }
                AttachedCommentPosition::Inside => false,
            };
            let target = write_offsets[host];
            let attached = AttachedComment {
                comment,
                position: assignment.position,
                same_line: assignment.same_line,
                node_owned: function_header_comments[comment_index]
                    || (!source_only_hosts[host] && owned_position),
                node_exclusive: function_header_comments[comment_index]
                    || (!source_only_hosts[host]
                        && owned_position
                        && (!exact_leading
                            || assignment.position != AttachedCommentPosition::Before)),
            };
            sorted[target as usize] = attached;
            write_offsets[host] += 1;
        }

        for (index, node) in self.nodes.iter().enumerate() {
            let len = offsets[index + 1] - offsets[index];
            if len == 0 {
                continue;
            }
            attachments.push_host(CommentAttachmentHost {
                node_id: node.node_id,
                start: offsets[index],
                len,
            });
        }
        for (index, comment) in sorted.into_iter().enumerate() {
            attachments.set_comment(index, comment);
        }
    }
}
