//! Assign source comments to AST nodes.
//!
//! Each comment is owned by the deepest node containing it, or by a child of that node next to it:
//!
//! * Leading on the following child.
//! * Trailing on the preceding child, if the comment is on the same line as the end of that child
//!   or the comma after it.
//! * Otherwise dangling on the containing node.
//!
//! Most comments are assigned during parsing, where the owner follows from the surrounding tokens:
//! comments before a statement, after a token such as `,`, `(`, `:`, `=` or `return`, and before
//! the closing token after the last statement or property. These produce the same result as the
//! general rules.
//!
//! Other comments are assigned when the innermost statement containing them is complete, by
//! walking that statement. The walk only descends into children containing unassigned comments.
//! Comments not inside any statement, or skipped because they were lexed before a rewind, are
//! assigned by walking Program after parsing.
//!
//! A comment whose owner is [`NodeId::ROOT`] is unassigned until parsing finishes. Program is the
//! only node with that ID and is not complete until then.

use oxc_ast::{
    AstKind, Comment, CommentContent, CommentPlacement,
    ast::{
        BindingPattern, ClassElement, Directive, Expression, JSXChild, ObjectPropertyKind, Program,
        Statement, TSSignature, TSTemplateLiteralType, TSType, TemplateLiteral,
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

/// Reusable buffers for walks, to avoid allocating for each statement.
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

/// Comment state at the start of a statement.
#[derive(Clone, Copy)]
pub struct StatementComments {
    /// Number of comments lexed.
    first: u32,
    /// Number of comments assigned.
    assigned: u32,
    /// Range of comments between the previous token and the statement.
    leading: u32,
    leading_end: u32,
    /// Start of the statement's first token.
    start: u32,
    /// Previous statement, if it ends at the previous token, or [`NodeId::DUMMY`].
    sibling: NodeId,
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Record comment state before parsing a statement.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    pub(crate) fn start_statement_comments(&self) -> StatementComments {
        let comments = &self.lexer.trivia_builder.comments;
        let first = comments.len() as u32;
        let mut state = StatementComments {
            first,
            assigned: self.assigned_comments,
            leading: first,
            leading_end: first,
            start: self.cur_token().start(),
            sibling: NodeId::DUMMY,
        };
        if self.cur_token().follows_comment() {
            (state.leading, state.leading_end) = self.leading_comments();
            if state.leading != state.leading_end && self.last_statement.0 == self.prev_token_end {
                state.sibling = self.last_statement.1;
            }
        }
        state
    }

    /// Assign comments preceding and inside a completed statement which are not owned by its
    /// nested statements.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    pub(crate) fn finish_statement_comments(
        &mut self,
        stmt: &Statement<'a>,
        state: StatementComments,
    ) {
        // Skip if nested statements and annotations assigned every comment lexed in the
        // statement. None of them can be leading, trailing or inside this statement.
        let lexed = (self.lexer.trivia_builder.comments.len() as u32).wrapping_sub(state.first);
        if lexed != self.assigned_comments.wrapping_sub(state.assigned)
            || state.leading != state.leading_end
        {
            self.finish_statement_comments_cold(stmt, state);
        }
    }

    #[cold]
    #[inline(never)]
    fn finish_statement_comments_cold(&mut self, stmt: &Statement<'a>, state: StatementComments) {
        let node_id = stmt.node_id();
        let span = stmt.span();
        self.attach_inside_statement(stmt, state);
        // Comments before decorators preceding the statement's span are not next to it.
        let leading_end = if span.start == state.start { state.leading_end } else { state.leading };
        for comment in
            &mut self.lexer.trivia_builder.comments[state.leading as usize..leading_end as usize]
        {
            if comment.node_id.get() != NodeId::ROOT {
                continue;
            }
            // A comment between two statements trails the first if it ends its line.
            if state.sibling != NodeId::DUMMY
                && !comment.preceded_by_newline()
                && comment.followed_by_newline()
            {
                comment.node_id.set(state.sibling);
                comment.placement = CommentPlacement::Trailing;
            } else {
                comment.node_id.set(node_id);
                comment.placement = CommentPlacement::Leading;
            }
            self.assigned_comments += 1;
        }
        self.last_statement = (span.end, node_id);
    }

    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn attach_inside_statement(&mut self, stmt: &Statement<'a>, state: StatementComments) {
        let first = state.first as usize;
        let comments = &self.lexer.trivia_builder.comments[first..];
        let end = stmt.span().end;
        // Comments lexed after the statement started are after its first token. Only a few
        // comments before the next token are after the statement.
        let mut inside = comments.len();
        while inside > 0 && comments[inside - 1].span.start >= end {
            inside -= 1;
        }
        // All of them are assigned if nested statements and annotations assigned this many.
        if inside as u32 == self.assigned_comments.wrapping_sub(state.assigned) {
            return;
        }
        self.walk_statement(stmt, first, inside);
    }

    /// Assign comments inside a statement which are not owned by its nested statements, by
    /// walking it.
    #[cold]
    #[inline(never)]
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn walk_statement(&mut self, stmt: &Statement<'a>, first: usize, inside: usize) {
        let comments = &self.lexer.trivia_builder.comments[first..];
        let pending = &mut self.comment_attacher.pending;
        pending.clear();
        pending.extend(
            (first..first + inside)
                .filter(|&index| comments[index - first].node_id.get() == NodeId::ROOT)
                .map(|index| index as u32),
        );
        if pending.is_empty() {
            return;
        }
        self.assigned_comments += pending.len() as u32;
        let trivia = &mut self.lexer.trivia_builder;
        let mut walker = Walker::new(
            &mut trivia.comments,
            &trivia.comma_comments,
            &mut self.comment_attacher,
            self.source_text,
        );
        walker.visit_statement(stmt);
        walker.finish();
    }

    /// Assign comments after the last directive or statement of a list and before the closing
    /// token at `end` as trailing on it.
    #[inline]
    pub(crate) fn attach_trailing_comments(
        &mut self,
        directives: &[Directive<'a>],
        statements: &[Statement<'a>],
        end: u32,
    ) {
        let last_end = match statements.last() {
            Some(stmt) => stmt.span().end,
            None => match directives.last() {
                Some(directive) => directive.span.end,
                None => return,
            },
        };
        if self
            .lexer
            .trivia_builder
            .comments
            .last()
            .is_some_and(|comment| comment.span.end > last_end)
        {
            let node_id = match statements.last() {
                Some(stmt) => stmt.node_id(),
                None => directives.last().unwrap().node_id.get(),
            };
            self.attach_trailing_comments_cold(last_end, node_id, end);
        }
    }

    #[cold]
    #[inline(never)]
    fn attach_trailing_comments_cold(&mut self, start: u32, node_id: NodeId, end: u32) {
        for comment in self.lexer.trivia_builder.comments.iter_mut().rev() {
            if comment.span.start < start {
                break;
            }
            if comment.span.end <= end && comment.node_id.get() == NodeId::ROOT {
                comment.node_id.set(node_id);
                comment.placement = CommentPlacement::Trailing;
                self.assigned_comments += 1;
            }
        }
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
        for comment in &mut self.lexer.trivia_builder.comments[start as usize..end as usize] {
            if comment.node_id.get() == NodeId::ROOT {
                comment.node_id.set(node_id);
                comment.placement = CommentPlacement::Trailing;
                self.assigned_comments += 1;
            }
        }
    }

    /// Assign comments ending the line after a list separator as trailing on the preceding
    /// element.
    #[cold]
    #[inline(never)]
    pub(crate) fn attach_comments_after_separator<T: GetSpan + GetNodeId>(&mut self, element: &T) {
        let node_id = element.node_id();
        let (start, end) = self.leading_comments();
        let trivia = &mut self.lexer.trivia_builder;
        for index in start..end {
            let comment = &mut trivia.comments[index as usize];
            if comment.followed_by_newline()
                && trivia.comma_comments.binary_search(&index).is_ok()
                && comment.node_id.get() == NodeId::ROOT
            {
                comment.node_id.set(node_id);
                comment.placement = CommentPlacement::Trailing;
                self.assigned_comments += 1;
            }
        }
    }

    /// Replace the owner of a statement's comments, when the statement is replaced by a node with
    /// a different ID.
    pub(crate) fn replace_statement_comment_owner(&mut self, old: NodeId, new: NodeId, span: Span) {
        if self.last_statement.1 == old {
            self.last_statement.1 = new;
        }
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
        let comments = &mut self.lexer.trivia_builder.comments;
        let mut index = leading.end as usize;
        while index > 0 {
            index -= 1;
            let comment = &mut comments[index];
            if comment.span.start < leading.from {
                break;
            }
            // Comments after the token may have been lexed before a rewind.
            if comment.span.start < leading.to && comment.node_id.get() == NodeId::ROOT {
                comment.node_id.set(node_id);
                comment.placement = CommentPlacement::Leading;
                self.assigned_comments += 1;
            }
        }
    }

    /// Range of comments between the previous token and the current token.
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    fn leading_comments(&self) -> (u32, u32) {
        let comments = &self.lexer.trivia_builder.comments;
        let token_start = self.cur_token().start();
        // Comments after the current token may have been lexed before a rewind.
        let mut end = comments.len();
        while end > 0 && comments[end - 1].span.start >= token_start {
            end -= 1;
        }
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
        let start = comments.partition_point(|comment| comment.span.start < span.start);
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

    /// Assign comments to the node an annotation applies to.
    pub(crate) fn attach_annotation_comments(&mut self, start: u32, end: u32, node_id: NodeId) {
        for comment in &mut self.lexer.trivia_builder.comments[start as usize..end as usize] {
            if matches!(comment.content, CommentContent::Pure | CommentContent::NoSideEffects)
                && comment.node_id.get() == NodeId::ROOT
            {
                comment.node_id.set(node_id);
                comment.placement = CommentPlacement::Leading;
                self.assigned_comments += 1;
            }
        }
    }

    /// Unassign comments assigned after a checkpoint, whose owners may have been discarded.
    ///
    /// Nodes completed after the checkpoint, and so the comments they own, start after
    /// `prev_token_end`.
    #[cold]
    #[inline(never)]
    pub(crate) fn rewind_comments(&mut self, prev_token_end: u32, assigned_comments: u32) {
        let comments = &mut self.lexer.trivia_builder.comments;
        let start = comments.partition_point(|comment| comment.span.start < prev_token_end);
        for comment in &mut comments[start..] {
            if comment.node_id.get() != NodeId::ROOT {
                comment.node_id.set(NodeId::ROOT);
                self.assigned_comments -= 1;
            }
        }
        debug_assert_eq!(self.assigned_comments, assigned_comments);
    }

    /// Assign the remaining comments by walking Program.
    #[expect(clippy::cast_possible_truncation, reason = "Comment counts fit within u32 offsets")]
    pub(crate) fn attach_remaining_comments(&mut self, program: &Program<'a>) {
        let trivia = &mut self.lexer.trivia_builder;
        let comments = &mut trivia.comments;
        if self.assigned_comments as usize == comments.len() {
            return;
        }
        let pending = &mut self.comment_attacher.pending;
        pending.clear();
        pending.extend(
            comments
                .iter()
                .enumerate()
                .filter(|(_, comment)| comment.node_id.get() == NodeId::ROOT)
                .map(|(index, _)| index as u32),
        );
        let mut walker = Walker::new(
            comments,
            &trivia.comma_comments,
            &mut self.comment_attacher,
            self.source_text,
        );
        walker.visit_program(program);
        walker.finish();
    }
}

/// Assign every comment to Program after the parsed AST has been discarded.
pub fn attach_comments_to_program(comments: &mut [Comment]) {
    for comment in comments {
        comment.node_id.set(NodeId::ROOT);
        comment.placement = CommentPlacement::Dangling;
    }
}

struct Walker<'w> {
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
        let following =
            following.filter(|&(_, start)| self.precedes(index, start)).map(|(node_id, _)| node_id);
        let (node_id, placement) = match (preceding, following) {
            (Some(preceding), Some(following)) => {
                let comment = &self.comments[index as usize];
                if !comment.preceded_by_newline() && comment.followed_by_newline() {
                    (preceding, CommentPlacement::Trailing)
                } else {
                    (following, CommentPlacement::Leading)
                }
            }
            (Some(preceding), None) => (preceding, CommentPlacement::Trailing),
            (None, Some(following)) => (following, CommentPlacement::Leading),
            (None, None) => (frame.node_id, CommentPlacement::Dangling),
        };
        let comment = &mut self.comments[index as usize];
        comment.node_id.set(node_id);
        comment.placement = placement;
    }

    /// Whether only whitespace and comments separate the comment from a preceding child ending at
    /// `preceding_end`.
    #[expect(clippy::cast_possible_truncation, reason = "Source offsets fit within u32")]
    fn follows(&self, index: u32, preceding_end: u32) -> bool {
        let mut index = index as usize;
        let mut position = self.comments[index].span.start;
        loop {
            if position < preceding_end {
                return false;
            }
            let gap = &self.source_text[preceding_end as usize..position as usize];
            position = preceding_end + gap.trim_end_matches(is_whitespace).len() as u32;
            if position == preceding_end {
                return true;
            }
            if index == 0 || self.comments[index - 1].span.end != position {
                return false;
            }
            index -= 1;
            position = self.comments[index].span.start;
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
