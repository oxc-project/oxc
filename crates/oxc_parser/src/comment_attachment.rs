//! Assign source comments to AST nodes.
//!
//! Each comment is owned by the deepest node containing it, or by a child of that node next to it:
//!
//! * Leading on the following child.
//! * Trailing on the preceding child, if the comment is on the same line as the end of that child
//!   or the comma after it, or if no following child is next to it.
//! * Otherwise dangling on the containing node.
//!
//! Most comments are assigned during parsing, where the owner follows from the surrounding tokens:
//! comments after a token such as `,`, `(`, `:`, `=`, `return`, a binary operator, `?` or `:` in
//! a conditional lead the following node, and comments before the closing token after the last
//! element of a list trail that element. These produce the same result as the general rules.
//!
//! Lists without separators (statements, class elements and switch cases) keep a
//! [`ListComments`]. When an element is complete, comments between the previous element and it
//! lead it or trail the previous element. Comments inside it which no nested node took are
//! assigned by walking the element. The walk only descends into children containing unassigned
//! comments. Checking whether there is any such comment is a single comparison with the number of
//! comments lexed, so elements without comments around them cost almost nothing.
//!
//! Some comments are left for the walk of the enclosing list element, or for a walk of Program
//! after parsing if there is none:
//!
//! * Comments before an element whose first token is before its span, e.g. decorators before
//!   `export`.
//! * Comments before an empty class element `;`, which is not a node.
//! * Comments lexed before a rewind.
//! * Comments not inside any list element.
//!
//! A comment whose owner is [`NodeId::ROOT`] is unassigned until parsing finishes. Program is the
//! only node with that ID and is not complete until then.

use std::ops::Range;

use oxc_ast::{
    AstKind, Comment, CommentContent, CommentPlacement,
    ast::{
        BindingPattern, BindingProperty, ClassElement, Declaration, Directive,
        ExportDefaultDeclarationKind, Expression, JSXChild, ObjectPropertyKind, Program, Statement,
        SwitchCase, TSSignature, TSTemplateLiteralType, TSType, TemplateLiteral,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_white_space,
    line_terminator::is_line_terminator,
    node::{GetNodeId, NodeId},
};

use crate::{ParserConfig as Config, ParserImpl};

/// Reusable buffers for walks, to avoid allocating for each walk.
#[derive(Default)]
pub struct CommentAttacher {
    pending: Vec<u32>,
    frames: Vec<Frame>,
}

/// A node containing pending comments, which are assigned in it unless a child contains them.
#[derive(Clone, Copy)]
struct Frame {
    node_id: NodeId,
    /// Depth of the node in the walk.
    depth: u32,
    /// Index in `pending` of the first comment after the children visited so far.
    cursor: u32,
    /// End of the range of `pending` inside the node.
    end: u32,
    /// Last child visited.
    preceding: Option<NodeId>,
    /// End of the last child visited. Children starting before it are out of source order.
    preceding_end: u32,
}

/// Comments between a delimiter and the following token.
#[derive(Clone, Copy)]
pub struct DelimitedComments {
    /// Number of comments lexed. The comments are before this index.
    end: u32,
    /// End of the delimiter.
    from: u32,
    /// Start of the token.
    to: u32,
}

/// Comment state of a list without separators: statements, class elements or switch cases.
pub struct ListComments {
    /// Number of comments lexed when the list last had no unassigned comments.
    seen: u32,
    /// Start of the list. Comments after it and before the first element lead that element.
    start: u32,
    /// Number of unassigned comments lexed before the list, which are outside it.
    outside: u32,
}

impl ListComments {
    /// Restart the list after a token at `start` which is not a list element. Comments before it
    /// are assigned by walking the list's parent.
    pub(crate) fn restart(&mut self, start: u32) {
        self.start = start;
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Number of comments lexed before the current token, minus the number assigned.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn unassigned_comments(&self) -> u32 {
        let trivia = &self.lexer.trivia_builder;
        let lexed = if self.cur_token().follows_comment() {
            trivia.token_comments_start
        } else {
            trivia.comments.len() as u32
        };
        lexed.wrapping_sub(self.assigned_comments)
    }

    /// Record comment state before parsing a list, which starts after the previous token.
    #[inline]
    pub(crate) fn start_list_comments(&self) -> ListComments {
        let outside = self.unassigned_comments();
        ListComments {
            seen: outside.wrapping_add(self.assigned_comments),
            start: self.prev_token_end,
            outside,
        }
    }

    /// Assign comments preceding and inside a completed list element which are not owned by its
    /// nested statements or list elements. `elements` precede it in its list, after `directives`
    /// in a statement list.
    #[inline]
    pub(crate) fn finish_list_element_comments<T: ListElement>(
        &mut self,
        element: &T,
        list: &mut ListComments,
        directives: &[Directive<'a>],
        elements: &[T],
    ) {
        #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32")]
        if self.lexer.trivia_builder.comments.len() as u32 != list.seen {
            self.finish_list_element_comments_cold(element, list, directives, elements);
        }
    }

    #[cold]
    #[inline(never)]
    fn finish_list_element_comments_cold<T: ListElement>(
        &mut self,
        element: &T,
        list: &mut ListComments,
        directives: &[Directive<'a>],
        elements: &[T],
    ) {
        self.attach_list_element_leading_comments(element, list, || match elements.last() {
            Some(previous) => Some((previous.span().end, previous.node_id())),
            None => directives.last().map(|previous| (previous.span.end, previous.node_id.get())),
        });
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn attach_list_element_leading_comments<T: ListElement>(
        &mut self,
        element: &T,
        list: &mut ListComments,
        previous: impl FnOnce() -> Option<(u32, NodeId)>,
    ) {
        // Comments before the current token are after the element, and lead the next one.
        // Others are assigned once the element is complete, so the list has no unassigned
        // comments unless some precede or are inside this element.
        let unassigned = self.unassigned_comments();
        let lexed = unassigned.wrapping_add(self.assigned_comments) as usize;
        #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32")]
        {
            list.seen = lexed as u32;
        }
        let pending = unassigned.wrapping_sub(list.outside) as usize;
        if pending == 0 {
            return;
        }
        let (from, sibling) = match previous() {
            Some((end, node_id)) => (end, Some(node_id)),
            None => (list.start, None),
        };
        let span = element.span();
        let comments = &self.lexer.trivia_builder.comments;
        // The pending comments are usually the last comments lexed, between the previous element
        // and this one. Otherwise some are inside it or not next to it, or annotation comments
        // assigned to the element are among them.
        let end = lexed;
        let start = end.wrapping_sub(pending);
        if start >= end
            || comments[end - 1].span.start >= span.start
            || comments[start].span.start < from
            || element.starts_before_span()
            || comments[start..end].iter().any(|comment| comment.node_id.get() != NodeId::ROOT)
        {
            self.attach_list_element_comments(element, lexed, pending, from, sibling);
            return;
        }
        let node_id = element.node_id();
        let comments = &mut self.lexer.trivia_builder.comments;
        for index in start..end {
            assign_between(comments, index, from, span.start, sibling, node_id);
        }
        #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32")]
        {
            self.assigned_comments += pending as u32;
        }
    }

    /// Assign `pending` comments before and inside a list element, which may be followed by
    /// comments lexed before a rewind. `from` and `sibling` are the end and ID of the previous
    /// element.
    #[cold]
    #[inline(never)]
    fn attach_list_element_comments<T: ListElement>(
        &mut self,
        element: &T,
        lexed: usize,
        pending: usize,
        from: u32,
        sibling: Option<NodeId>,
    ) {
        let span = element.span();
        let comments = &self.lexer.trivia_builder.comments;
        let inside_end = partition_before(comments, lexed, span.end);
        let inside_start = partition_before(comments, inside_end, span.start);
        let mut leading_start = inside_start;
        while leading_start > 0 && comments[leading_start - 1].span.start >= from {
            leading_start -= 1;
        }
        // Comments before and after the element are unassigned, so any other pending comments
        // are inside it.
        let leading = inside_start - leading_start;
        if pending != leading + (lexed - inside_end) {
            self.walk_comments(inside_start..inside_end, |walker| element.visit(walker));
        }
        // Comments before decorators preceding the span are assigned by walking Program.
        if element.starts_before_span() {
            return;
        }
        let node_id = element.node_id();
        let comments = &mut self.lexer.trivia_builder.comments;
        for index in leading_start..inside_start {
            if comments[index].node_id.get() == NodeId::ROOT {
                assign_between(comments, index, from, span.start, sibling, node_id);
                self.assigned_comments += 1;
            }
        }
    }

    /// Assign an unassigned comment.
    fn assign_comment(&mut self, index: usize, node_id: NodeId, placement: CommentPlacement) {
        let comment = &mut self.lexer.trivia_builder.comments[index];
        debug_assert_eq!(comment.node_id.get(), NodeId::ROOT);
        comment.node_id.set(node_id);
        comment.placement = placement;
        self.assigned_comments += 1;
    }

    /// Assign the unassigned comments in `range` by walking the node `visit` visits, which
    /// contains them.
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn walk_comments(&mut self, range: Range<usize>, visit: impl FnOnce(&mut Walker<'_>)) {
        let trivia = &mut self.lexer.trivia_builder;
        let pending = &mut self.comment_attacher.pending;
        pending.clear();
        pending.extend(
            range
                .filter(|&index| trivia.comments[index].node_id.get() == NodeId::ROOT)
                .map(|index| index as u32),
        );
        if pending.is_empty() {
            return;
        }
        self.assigned_comments += pending.len() as u32;
        let mut walker = Walker::new(
            &mut trivia.comments,
            &trivia.comma_comments,
            &mut self.comment_attacher,
            self.source_text,
        );
        visit(&mut walker);
        walker.finish();
    }

    /// Assign comments after the last element of a list and before the closing token as trailing
    /// on that element, if no separator follows it.
    #[inline]
    pub(crate) fn attach_comments_before_close<T: GetSpan + GetNodeId>(&mut self, list: &[T]) {
        if self.cur_token().follows_comment()
            && let Some(last) = list.last()
            && last.span().end == self.prev_token_end
        {
            self.attach_comments_before_close_cold(last.node_id());
        }
    }

    #[cold]
    #[inline(never)]
    fn attach_comments_before_close_cold(&mut self, node_id: NodeId) {
        let (start, end) = self.leading_comments();
        for index in start as usize..end as usize {
            if self.lexer.trivia_builder.comments[index].node_id.get() == NodeId::ROOT {
                self.assign_comment(index, node_id, CommentPlacement::Trailing);
            }
        }
    }

    /// Assign comments after a list separator which trail the preceding element.
    #[cold]
    #[inline(never)]
    pub(crate) fn attach_comments_after_separator<T: GetSpan + GetNodeId>(&mut self, element: &T) {
        self.attach_comments_after_element(element.node_id(), element.span().end);
    }

    /// Assign comments after a separator which trail the preceding element, if it is not an array
    /// pattern elision.
    #[cold]
    #[inline(never)]
    pub(crate) fn attach_comments_after_separated_element<T: SeparatedElement>(
        &mut self,
        element: &T,
    ) {
        if let Some((node_id, end)) = element.id_and_end() {
            self.attach_comments_after_element(node_id, end);
        }
    }

    fn attach_comments_after_element(&mut self, node_id: NodeId, element_end: u32) {
        let (start, end) = self.leading_comments();
        let token_start = self.cur_token().start();
        for index in start as usize..end as usize {
            let comments = &self.lexer.trivia_builder.comments;
            if comments[index].node_id.get() == NodeId::ROOT
                && trails_preceding(comments, index, element_end, token_start)
            {
                self.assign_comment(index, node_id, CommentPlacement::Trailing);
            }
        }
    }

    /// Replace the owner of a statement's comments, when the statement is replaced by a node with
    /// a different ID.
    pub(crate) fn replace_statement_comment_owner(&mut self, old: NodeId, new: NodeId, span: Span) {
        for comment in self.lexer.trivia_builder.comments.iter_mut().rev() {
            if comment.node_id.get() == old {
                comment.node_id.set(new);
            } else if comment.span.end <= span.start {
                break;
            }
        }
    }

    /// Parse a node which follows a token without a node of its own, such as an opening bracket
    /// or a separator. Comments after that token lead the node.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    pub(crate) fn parse_after_delimiter<T: GetSpan + GetNodeId>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let token = self.cur_token();
        let leading = token.follows_comment().then(|| DelimitedComments {
            end: self.lexer.trivia_builder.comments.len() as u32,
            from: self.prev_token_end,
            to: token.start(),
        });
        let node = parse(self);
        if let Some(leading) = leading {
            self.attach_delimited_leading_comments(leading, node.span().start, node.node_id());
        }
        node
    }

    #[cold]
    #[inline(never)]
    fn attach_delimited_leading_comments(
        &mut self,
        leading: DelimitedComments,
        node_start: u32,
        node_id: NodeId,
    ) {
        // The node does not own the comments if they are inside it, e.g. `(/* c */ a)` without
        // parenthesized expressions.
        if node_start != leading.to {
            return;
        }
        // Comments after the token may have been lexed before a rewind.
        let comments = &self.lexer.trivia_builder.comments;
        let mut index = partition_before(comments, leading.end as usize, leading.to);
        while index > 0 {
            index -= 1;
            let comment = &self.lexer.trivia_builder.comments[index];
            if comment.span.start < leading.from {
                break;
            }
            if comment.node_id.get() == NodeId::ROOT {
                self.assign_comment(index, node_id, CommentPlacement::Leading);
            }
        }
    }

    /// Range of comments between the previous token and the current token.
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn leading_comments(&self) -> (u32, u32) {
        let comments = &self.lexer.trivia_builder.comments;
        let token_start = self.cur_token().start();
        // Comments after the current token may have been lexed before a rewind.
        let end = partition_before(comments, comments.len(), token_start);
        let mut start = end;
        while start > 0 && comments[start - 1].span.start >= self.prev_token_end {
            start -= 1;
        }
        (start as u32, end as u32)
    }

    /// Unassign comments in a node which is discarded, after it is converted to another node.
    #[cold]
    #[inline(never)]
    pub(crate) fn unassign_comments(&mut self, span: Span) {
        let comments = &mut self.lexer.trivia_builder.comments;
        let start = partition_before(comments, comments.len(), span.start);
        for comment in &mut comments[start..] {
            if comment.span.start >= span.end {
                break;
            }
            if comment.node_id.get() != NodeId::ROOT {
                comment.node_id.set(NodeId::ROOT);
                self.assigned_comments -= 1;
            }
        }
    }

    /// Unassign comments assigned after a checkpoint, whose owners may be discarded.
    /// `prev_token_end` and `token_start` are the end of the previous token and the start of the
    /// current token at the checkpoint.
    ///
    /// Nodes completed after the checkpoint start at or after `token_start`, so they own comments
    /// after `prev_token_end` as leading or dangling comments. Comments between the tokens which
    /// trail a node were assigned before the checkpoint.
    #[cold]
    #[inline(never)]
    pub(crate) fn rewind_comments(&mut self, prev_token_end: u32, token_start: u32) {
        let comments = &mut self.lexer.trivia_builder.comments;
        let start = partition_before(comments, comments.len(), prev_token_end);
        for comment in &mut comments[start..] {
            if comment.node_id.get() != NodeId::ROOT
                && (comment.span.start >= token_start
                    || comment.placement != CommentPlacement::Trailing)
            {
                comment.node_id.set(NodeId::ROOT);
                self.assigned_comments -= 1;
            }
        }
    }

    /// Assign comments to the node an annotation applies to.
    pub(crate) fn attach_annotation_comments(&mut self, start: u32, end: u32, node_id: NodeId) {
        for index in start as usize..end as usize {
            let comment = &self.lexer.trivia_builder.comments[index];
            if matches!(comment.content, CommentContent::Pure | CommentContent::NoSideEffects)
                && comment.node_id.get() == NodeId::ROOT
            {
                self.assign_comment(index, node_id, CommentPlacement::Leading);
            }
        }
    }

    /// Assign the remaining comments by walking Program.
    pub(crate) fn attach_remaining_comments(&mut self, program: &Program<'a>) {
        let len = self.lexer.trivia_builder.comments.len();
        if self.assigned_comments as usize != len {
            self.walk_comments(0..len, |walker| walker.visit_program(program));
        }
    }
}

/// Assign every comment to Program after the parsed AST has been discarded.
pub fn attach_comments_to_program(comments: &mut [Comment]) {
    for comment in comments {
        comment.node_id.set(NodeId::ROOT);
        comment.placement = CommentPlacement::Dangling;
    }
}

pub struct Walker<'w> {
    comments: &'w mut [Comment],
    /// Sorted indices of comments following a comma on the same line.
    comma_comments: &'w [u32],
    pending: &'w [u32],
    frames: &'w mut Vec<Frame>,
    source_text: &'w str,
    /// Depth of the current node. Only children of the innermost frame's node can own comments.
    depth: u32,
}

impl<'w> Walker<'w> {
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn new(
        comments: &'w mut [Comment],
        comma_comments: &'w [u32],
        attacher: &'w mut CommentAttacher,
        source_text: &'w str,
    ) -> Self {
        let CommentAttacher { pending, frames } = attacher;
        frames.clear();
        // The root node of the walk is the only child of this frame, and contains every comment.
        frames.push(Frame {
            node_id: NodeId::ROOT,
            depth: 0,
            cursor: 0,
            end: pending.len() as u32,
            preceding: None,
            preceding_end: 0,
        });
        Self { comments, comma_comments, pending, frames, source_text, depth: 0 }
    }

    fn finish(mut self) {
        while let Some(frame) = self.frames.pop() {
            self.assign_rest(frame);
        }
    }

    /// Visit a child of the innermost frame's node. Returns whether it contains pending comments.
    fn child(&mut self, node_id: NodeId, span: Span) -> bool {
        let frame = self.frames.last_mut().unwrap();
        if span.start < frame.preceding_end {
            return false;
        }
        let mut frame = *frame;
        while frame.cursor < frame.end {
            let index = self.pending[frame.cursor as usize];
            if self.comments[index as usize].span.end > span.start {
                break;
            }
            self.assign(index, &frame, Some((node_id, span.start)));
            frame.cursor += 1;
        }
        let start = frame.cursor;
        let mut end = start;
        if end < frame.end {
            if self.comment_start(frame.end - 1) < span.end {
                end = frame.end;
            } else {
                while self.comment_start(end) < span.end {
                    end += 1;
                }
            }
        }
        frame.cursor = end;
        frame.preceding = Some(node_id);
        frame.preceding_end = span.end;
        *self.frames.last_mut().unwrap() = frame;
        if start == end {
            return false;
        }
        self.frames.push(Frame {
            node_id,
            depth: frame.depth + 1,
            cursor: start,
            end,
            preceding: None,
            preceding_end: span.start,
        });
        true
    }

    fn comment_start(&self, position: u32) -> u32 {
        self.comments[self.pending[position as usize] as usize].span.start
    }

    /// Visit a node behind an enum, without walking it unless it contains pending comments.
    #[inline]
    fn enter_enum(&mut self, node_id: NodeId, span: Span) -> bool {
        let frame = self.frames.last().unwrap();
        if self.depth == frame.depth {
            self.child(node_id, span)
        } else {
            // A frame was pushed for this node by an enclosing enum, or it is in a child without
            // pending comments.
            frame.depth == self.depth + 1 && frame.node_id == node_id
        }
    }

    /// Assign comments after the last child of a frame's node.
    fn assign_rest(&mut self, frame: Frame) {
        for position in frame.cursor..frame.end {
            let index = self.pending[position as usize];
            self.assign(index, &frame, None);
        }
    }

    /// Assign a comment in a frame's node, after its children visited so far.
    ///
    /// `following` is the next child and its start.
    fn assign(&mut self, index: u32, frame: &Frame, following: Option<(NodeId, u32)>) {
        let preceding = frame.preceding.filter(|_| {
            self.comma_comments.binary_search(&index).is_ok()
                || self.follows(index, frame.preceding_end)
        });
        let following = following.filter(|&(_, start)| self.precedes(index, start));
        let (node_id, placement) = match (preceding, following) {
            (Some(preceding), Some((following, following_start))) => {
                if trails_preceding(
                    self.comments,
                    index as usize,
                    frame.preceding_end,
                    following_start,
                ) {
                    (preceding, CommentPlacement::Trailing)
                } else {
                    (following, CommentPlacement::Leading)
                }
            }
            (Some(preceding), None) => (preceding, CommentPlacement::Trailing),
            (None, Some((following, _))) => (following, CommentPlacement::Leading),
            (None, None) => (frame.node_id, CommentPlacement::Dangling),
        };
        let comment = &mut self.comments[index as usize];
        comment.node_id.set(node_id);
        comment.placement = placement;
    }

    /// Whether only whitespace and comments separate the comment from a preceding child ending at
    /// `preceding_end`.
    fn follows(&self, index: u32, preceding_end: u32) -> bool {
        let mut index = index as usize;
        loop {
            let start = self.comments[index].span.start;
            if start < preceding_end {
                return false;
            }
            // Line comments may end with whitespace, so the gap starts after the previous comment.
            let gap_start = index.checked_sub(1).map_or(preceding_end, |previous| {
                self.comments[previous].span.end.max(preceding_end)
            });
            if !self.source_text[gap_start as usize..start as usize].chars().all(is_whitespace) {
                return false;
            }
            if gap_start == preceding_end {
                return true;
            }
            index -= 1;
        }
    }

    /// Whether only whitespace and comments separate the comment from a following child starting
    /// at `following_start`.
    #[expect(clippy::cast_possible_truncation, reason = "Source offsets fit within u32")]
    fn precedes(&self, index: u32, following_start: u32) -> bool {
        let mut index = index as usize;
        let mut position = self.comments[index].span.end;
        loop {
            if position > following_start {
                return false;
            }
            let gap = &self.source_text[position as usize..following_start as usize];
            position = following_start - gap.trim_start_matches(is_whitespace).len() as u32;
            if position == following_start {
                return true;
            }
            index += 1;
            if index == self.comments.len() || self.comments[index].span.start != position {
                return false;
            }
            position = self.comments[index].span.end;
        }
    }
}

/// Whether a comment between two nodes trails the first, rather than leading the second.
///
/// It does if it is on the same line as the first node, and that line ends after it or after
/// later comments before the second node. In `a, /* x */ // y` both comments trail `a`, and in
/// `a, /* x */ b` the comment leads `b`.
///
/// `from` is the end of the first node and `to` is the start of the second.
fn trails_preceding(comments: &[Comment], index: usize, from: u32, to: u32) -> bool {
    let mut first = index;
    loop {
        if comments[first].preceded_by_newline() {
            return false;
        }
        if first == 0 || comments[first - 1].span.start < from {
            break;
        }
        first -= 1;
    }
    let mut last = index;
    while !comments[last].followed_by_newline() {
        last += 1;
        if last == comments.len() || comments[last].span.start >= to {
            return false;
        }
    }
    true
}

/// Assign an unassigned comment between a list element ending at `from` with ID `sibling`, and
/// one starting at `to` with ID `node_id`, as trailing on the first or leading on the second.
/// The caller counts it as assigned.
#[expect(clippy::inline_always)]
#[inline(always)]
fn assign_between(
    comments: &mut [Comment],
    index: usize,
    from: u32,
    to: u32,
    sibling: Option<NodeId>,
    node_id: NodeId,
) {
    let (node_id, placement) = match sibling.filter(|_| trails_preceding(comments, index, from, to))
    {
        Some(sibling) => (sibling, CommentPlacement::Trailing),
        None => (node_id, CommentPlacement::Leading),
    };
    let comment = &mut comments[index];
    comment.node_id.set(node_id);
    comment.placement = placement;
}

/// Index of the first of `comments[..end]` which starts at or after `pos`. It is usually `end` or
/// close to it, but many comments may be after `pos` after a rewind.
fn partition_before(comments: &[Comment], end: usize, pos: u32) -> usize {
    let limit = end.saturating_sub(4);
    let mut index = end;
    while index > limit {
        if comments[index - 1].span.start < pos {
            return index;
        }
        index -= 1;
    }
    comments[..index].partition_point(|comment| comment.span.start < pos)
}

/// A node in a list without separators, which owns the comments before it.
pub trait ListElement: GetSpan + GetNodeId {
    /// Visit the node with a walker.
    fn visit(&self, walker: &mut Walker<'_>);

    /// Whether the node's first token is before its span, so comments before it are not next to
    /// it.
    fn starts_before_span(&self) -> bool {
        false
    }
}

impl ListElement for Statement<'_> {
    fn visit(&self, walker: &mut Walker<'_>) {
        walker.visit_statement(self);
    }

    /// Decorators can precede `export`, e.g. `@dec export class C {}`.
    fn starts_before_span(&self) -> bool {
        let class = match self {
            Statement::ExportDeclaration(decl) => match &decl.declaration {
                Declaration::ClassDeclaration(class) => class,
                _ => return false,
            },
            Statement::ExportDefaultDeclaration(decl) => match &decl.declaration {
                ExportDefaultDeclarationKind::ClassDeclaration(class) => class,
                _ => return false,
            },
            _ => return false,
        };
        class.decorators.first().is_some_and(|decorator| decorator.span.start < self.span().start)
    }
}

impl ListElement for ClassElement<'_> {
    fn visit(&self, walker: &mut Walker<'_>) {
        walker.visit_class_element(self);
    }
}

impl ListElement for SwitchCase<'_> {
    fn visit(&self, walker: &mut Walker<'_>) {
        walker.visit_switch_case(self);
    }
}

/// A list element which owns comments ending its line after the following separator.
pub trait SeparatedElement {
    /// ID and end of the element, or `None` for an elision.
    fn id_and_end(&self) -> Option<(NodeId, u32)>;
}

impl SeparatedElement for BindingProperty<'_> {
    fn id_and_end(&self) -> Option<(NodeId, u32)> {
        Some((self.node_id(), self.span.end))
    }
}

impl SeparatedElement for Option<BindingPattern<'_>> {
    fn id_and_end(&self) -> Option<(NodeId, u32)> {
        self.as_ref().map(|pattern| (pattern.node_id(), pattern.span().end))
    }
}

fn is_whitespace(c: char) -> bool {
    is_white_space(c) || is_line_terminator(c)
}

macro_rules! prune {
    ($($visit:ident, $walk:ident, $ty:ident;)*) => {
        $(
            fn $visit(&mut self, it: &$ty<'a>) {
                if self.enter_enum(it.node_id(), it.span()) {
                    walk::$walk(self, it);
                }
            }
        )*
    };
}

impl<'a> Visit<'a> for Walker<'_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.depth += 1;
        if self.depth == self.frames.last().unwrap().depth + 1 {
            self.child(kind.node_id(), kind.span());
        }
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        if self.frames.last().unwrap().depth == self.depth {
            let frame = self.frames.pop().unwrap();
            self.assign_rest(frame);
        }
        self.depth -= 1;
    }

    // Quasis and expressions are visited in source order, interleaved.
    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(self.alloc(it));
        self.enter_node(kind);
        for (index, quasi) in it.quasis.iter().enumerate() {
            self.visit_template_element(quasi);
            if let Some(expression) = it.expressions.get(index) {
                self.visit_expression(expression);
            }
        }
        self.leave_node(kind);
    }

    fn visit_ts_template_literal_type(&mut self, it: &TSTemplateLiteralType<'a>) {
        let kind = AstKind::TSTemplateLiteralType(self.alloc(it));
        self.enter_node(kind);
        for (index, quasi) in it.quasis.iter().enumerate() {
            self.visit_template_element(quasi);
            if let Some(ts_type) = it.types.get(index) {
                self.visit_ts_type(ts_type);
            }
        }
        self.leave_node(kind);
    }

    prune! {
        visit_statement, walk_statement, Statement;
        visit_expression, walk_expression, Expression;
        visit_binding_pattern, walk_binding_pattern, BindingPattern;
        visit_object_property_kind, walk_object_property_kind, ObjectPropertyKind;
        visit_class_element, walk_class_element, ClassElement;
        visit_jsx_child, walk_jsx_child, JSXChild;
        visit_ts_type, walk_ts_type, TSType;
        visit_ts_signature, walk_ts_signature, TSSignature;
    }
}
