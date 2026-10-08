use std::{cell::Cell, ops::Range};

use oxc_allocator::ArenaVec;
use oxc_ast::{
    AstKind, Comment, CommentAttachment, CommentContent, CommentPlacement,
    ast::{
        BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, JSXChild, Program,
        Statement, TSType, TemplateElement,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;

const INLINE_COMMENTS: usize = 8;

#[derive(Clone, Copy)]
struct Neighbor {
    id: NodeId,
    // Span is pointer-aligned, which would pad every neighbor on 64-bit targets.
    // Store its endpoints separately to keep this scratch record compact.
    start: u32,
    end: u32,
}

impl Neighbor {
    const fn new(id: NodeId, span: Span) -> Self {
        Self { id, start: span.start, end: span.end }
    }

    fn span(self) -> Span {
        Span::new(self.start, self.end)
    }
}

// One scratch entry per comment. Containers route windows without updating every
// entry; ownership is established when the deepest containing frame is left.
#[derive(Clone, Copy)]
struct Pending {
    container: Neighbor,
    previous: Option<Neighbor>,
    next: Option<Neighbor>,
    attachment: Option<(NodeId, CommentPlacement)>,
}

impl Pending {
    const fn new() -> Self {
        Self {
            container: Neighbor::new(NodeId::ROOT, Span::new(0, u32::MAX)),
            previous: None,
            next: None,
            attachment: None,
        }
    }
}

/// Ownership state for an active AST node whose comment window is nonempty.
///
/// Frames form a stack of enclosing nodes. Children resolve comments inside their
/// spans first; when a frame is popped, it assigns the remaining comments in gaps
/// between children. Nodes with empty comment windows skip this stack.
///
/// For example:
///
/// ```js
/// outer(1, inner(/* argument */ value));
/// ```
/// The inner call's frame includes the argument comment. The comment falls outside
/// `value`'s span, so the inner call assigns it as a leading comment on `value` when
/// leaving the call. The resolved count then reaches the outer frames, allowing
/// them to skip their gap sweeps if all their comments have been assigned.
struct Frame<'a> {
    /// AST node, used to select syntax-specific ownership rules.
    kind: AstKind<'a>,
    /// Owner candidate and effective span, including decorators or substitution edges.
    node: Neighbor,
    /// Range of indices in the source comment slice considered by this frame.
    window: Range<usize>,
    /// First comment starting inside the effective span; earlier entries are inherited prefixes.
    inside_begin: usize,
    /// Exclusive end of comments preceding the node's original AST span.
    ///
    /// Annotations can precede the span, e.g. before `export const`. Keeping that
    /// prefix in the window allows it to follow the declaration's target child.
    leading_end: usize,
    /// Forward cursor locating comments at or after the latest ordered child start.
    cursor: usize,
    /// Greatest child start seen by the forward cursor.
    ///
    /// A child starting before this offset is visited out of source order and
    /// locates its comments with a binary search instead of advancing `cursor`.
    high_water_start: u32,
    /// Whether the effective span includes the edges of a template substitution.
    ///
    /// In `` `head${/* before */ value /* after */}tail` ``, both comments lie
    /// outside `value`'s AST span. The expanded window keeps them with `value`;
    /// this flag makes the comment after its span trailing rather than dangling.
    substitution: bool,
    /// Upper bound on comments still requiring a gap sweep in this frame.
    /// Descendants reduce it as they establish ownership; overlapping spans can overestimate it.
    unresolved: usize,
    /// Newly assigned comments in this frame and its descendants, reported to the parent on exit.
    resolved: usize,
}

impl Frame<'_> {
    fn accepts(&self, index: usize, pending: &Pending) -> bool {
        pending.attachment.is_none()
            && (index >= self.inside_begin || pending.container.id == self.node.id)
    }

    fn can_resolve(&self, index: usize, pending: &Pending) -> bool {
        // Preserve narrower owners across overlapping sibling spans. A later,
        // narrower sibling may still replace an owner established earlier.
        (index >= self.inside_begin || pending.container.id == self.node.id)
            && (pending.attachment.is_none()
                || pending.container.span().size() > self.node.span().size())
    }
}

pub struct AssignmentVisitor<'a, 'p> {
    comments: &'a [Comment],
    pending: &'p mut [Pending],
    frames: Vec<Frame<'a>>,
    skipped_depth: usize,
}

impl<'a, 'p> AssignmentVisitor<'a, 'p> {
    pub fn assign(program: &mut Program<'a>) {
        let comment_count = program.comments.len();
        if comment_count <= INLINE_COMMENTS {
            let mut pending = [Pending::new(); INLINE_COMMENTS];
            Self::assign_with_pending(program, &mut pending[..comment_count]);
        } else {
            let mut pending = vec![Pending::new(); comment_count];
            Self::assign_with_pending(program, &mut pending);
        }
    }

    fn assign_with_pending(program: &mut Program<'a>, pending: &mut [Pending]) {
        {
            // Select inline or heap storage once. The traversal uses the same
            // slice in both cases, without checking the storage kind per access.
            let mut visitor = AssignmentVisitor::new(&program.comments, pending);
            visitor.visit_program(program);
            visitor.finish();
        }
        // Release the traversal's shared references before writing ownership.
        // Reuse the existing scratch entries; no separate output table is needed.
        for (comment, pending) in program.comments.iter_mut().zip(pending) {
            comment.attachment = pending.attachment.map(|(node_id, placement)| CommentAttachment {
                node_id: Cell::new(node_id),
                placement,
            });
        }
    }

    fn new(comments: &'a [Comment], pending: &'p mut [Pending]) -> Self {
        Self {
            comments,
            pending,
            // Sparse inputs usually need a short ancestor chain. Reserve a
            // smaller buffer for them; deeper trees can grow it normally.
            frames: Vec::with_capacity(if comments.len() <= INLINE_COMMENTS { 8 } else { 32 }),
            skipped_depth: 0,
        }
    }

    fn finish(self) {
        debug_assert!(self.frames.is_empty());
        debug_assert_eq!(self.skipped_depth, 0);
        debug_assert!(self.pending.iter().all(|pending| pending.attachment.is_some()));
    }

    #[inline(never)]
    fn enter(&mut self, kind: AstKind<'a>, original_span: Span, node_id: NodeId) {
        let mut span = effective_span(kind, original_span);
        let mut substitution = false;
        let mut leading_count = 0;
        let (begin, inside_begin, end) = if let Some(parent) = self.frames.last_mut() {
            if let Some(window) = substitution_span(parent.kind, original_span) {
                span = window;
                substitution = true;
            }
            // Ordered children entirely before the next comment cannot own any
            // comments. Record the boundary without constructing a frame or
            // scanning gaps. Inherited annotation prefixes use the full path.
            if parent.leading_end == parent.window.start
                && span.start >= parent.high_water_start
                && (parent.cursor == parent.window.end
                    || self.comments[parent.cursor].span.start >= span.end)
            {
                if parent.cursor < parent.window.end {
                    let pending = &mut self.pending[parent.cursor];
                    if parent.accepts(parent.cursor, pending) {
                        pending.previous = pending.previous.filter(|neighbor| {
                            parent.node.span().contains_inclusive(neighbor.span())
                        });
                        if pending.previous.is_none_or(|previous| previous.end < span.end) {
                            pending.previous = Some(Neighbor::new(node_id, span));
                        }
                    }
                }
                parent.high_water_start = span.start;
                self.skipped_depth = 1;
                return;
            }
            let comments = &self.comments[parent.window.clone()];
            let child = Neighbor::new(node_id, span);
            let mut gap_begin;
            let inside_begin = if span.start >= parent.high_water_start {
                // Increasing child starts are the common case. Each parent cursor
                // moves forward through only its own comment window.
                gap_begin = parent.cursor.max(parent.leading_end);
                while parent.cursor < parent.window.end
                    && self.comments[parent.cursor].span.start < span.start
                {
                    parent.cursor += 1;
                }
                parent.high_water_start = span.start;
                parent.cursor
            } else {
                gap_begin = parent.leading_end;
                parent.window.start
                    + comments.partition_point(|comment| comment.span.start < span.start)
            };
            gap_begin = gap_begin.min(inside_begin);
            let mut begin = inside_begin;
            // Leading annotations follow a selected child through this window.
            // Ordinary gap comments stay with the parent and its neighbors.
            for index in (parent.window.start..parent.leading_end.min(inside_begin))
                .chain(gap_begin..inside_begin)
            {
                if parent.accepts(index, &self.pending[index])
                    && carries_leading_comment(
                        parent.kind,
                        kind,
                        &self.comments[index],
                        index < parent.leading_end,
                    )
                {
                    begin = begin.min(index);
                    // Only inherited prefixes need routing information per
                    // comment. Interior comments simply pass through the window.
                    self.pending[index].container = child;
                    leading_count += 1;
                }
            }
            let end = if inside_begin < parent.window.end
                && self.comments[parent.window.end - 1].span.end <= span.end
            {
                // A child containing the rest of its parent's window is common
                // along the path to a sparse comment. Forward it in constant time.
                parent.window.end
            } else {
                let mut end = inside_begin;
                while end < parent.window.end && self.comments[end].span.end <= span.end {
                    end += 1;
                }
                end
            };
            if end < parent.window.end && self.comments[end].span.start >= span.end {
                let pending = &mut self.pending[end];
                if parent.accepts(end, pending) {
                    pending.previous = pending
                        .previous
                        .filter(|neighbor| parent.node.span().contains_inclusive(neighbor.span()));
                    if pending.previous.is_none_or(|previous| previous.end < span.end) {
                        pending.previous = Some(child);
                    }
                }
            }
            // Seed the nearest gap comment that remains in the parent. An
            // annotation moving into the child must not hide that boundary.
            for index in (gap_begin..inside_begin).rev() {
                let pending = &mut self.pending[index];
                if parent.accepts(index, pending)
                    && !carries_leading_comment(parent.kind, kind, &self.comments[index], false)
                {
                    pending.next = pending
                        .next
                        .filter(|neighbor| parent.node.span().contains_inclusive(neighbor.span()));
                    if self.comments[index].span.end <= span.start
                        && pending.next.is_none_or(|next| next.start > span.start)
                    {
                        pending.next = Some(child);
                    }
                    break;
                }
            }
            (begin, inside_begin, end)
        } else {
            // Include file comments even if a manually built Program has a narrow span.
            span = Span::new(0, u32::MAX);
            (0, 0, self.comments.len())
        };

        if begin == end {
            // Descendants of a comment-free child need no ownership work.
            self.skipped_depth = 1;
            return;
        }
        let node = Neighbor::new(node_id, span);
        let mut leading_end = inside_begin;
        if span.start < original_span.start {
            while leading_end < end && self.comments[leading_end].span.end <= original_span.start {
                leading_end += 1;
            }
        }
        // Jump from the sparse reservation to the usual capacity if it fills,
        // avoiding an intermediate allocation for sixteen frames.
        if self.frames.len() == self.frames.capacity() && self.frames.capacity() < 32 {
            self.frames.reserve(32 - self.frames.len());
        }
        self.frames.push(Frame {
            kind,
            node,
            window: begin..end,
            inside_begin,
            leading_end,
            cursor: inside_begin,
            high_water_start: span.start,
            substitution,
            // Already resolved comments in overlapping windows can make this an
            // overestimate. They never contribute to the resolved count again.
            unresolved: end - inside_begin + leading_count,
            resolved: 0,
        });
    }

    fn leave(&mut self) {
        let mut frame = self.frames.pop().unwrap();
        if frame.unresolved != 0 {
            self.resolve(&mut frame);
        }
        if let Some(parent) = self.frames.last_mut() {
            parent.unresolved -= frame.resolved;
            parent.resolved += frame.resolved;
        } else {
            debug_assert_eq!(frame.resolved, self.comments.len());
        }
    }

    #[inline(never)]
    fn resolve(&mut self, frame: &mut Frame<'a>) {
        // Child boundaries only seed adjacent comments. Carry those neighbors
        // across each gap, so long sibling lists never scan the whole window per child.
        let mut previous = None;
        for (offset, pending) in self.pending[frame.window.clone()].iter_mut().enumerate() {
            if frame.can_resolve(frame.window.start + offset, pending) {
                pending.previous = pending
                    .previous
                    .filter(|neighbor| frame.node.span().contains_inclusive(neighbor.span()));
                pending.next = pending
                    .next
                    .filter(|neighbor| frame.node.span().contains_inclusive(neighbor.span()));
                if pending.previous.is_some() {
                    previous = pending.previous;
                }
                pending.previous = previous;
            }
        }
        let mut next = None;
        for index in frame.window.clone().rev() {
            let pending = &mut self.pending[index];
            if !frame.can_resolve(index, pending) {
                continue;
            }
            if pending.next.is_some() {
                next = pending.next;
            }
            let comment = &self.comments[index];
            // File directives always belong to Program, including comments the
            // parser marks trailing because they share a line with a token.
            let (node_id, placement) = if comment.content == CommentContent::CoverageIgnoreFile {
                (NodeId::ROOT, CommentPlacement::Leading)
            } else if index < frame.leading_end {
                (frame.node.id, CommentPlacement::Leading)
            } else if frame.substitution && comment.span.start >= frame.kind.span().end {
                (frame.node.id, CommentPlacement::Trailing)
            } else if matches!(frame.kind, AstKind::ImportExpression(_))
                && (comment.is_webpack() || comment.is_vite() || comment.is_turbopack())
            {
                (frame.node.id, CommentPlacement::Dangling)
            } else if let Some(previous) = pending.previous
                && comment.is_trailing()
                && comment.attached_to == previous.end
            {
                (previous.id, CommentPlacement::Trailing)
            } else if let Some(next) = next {
                (next.id, CommentPlacement::Leading)
            } else {
                (frame.node.id, CommentPlacement::Dangling)
            };
            if pending.attachment.is_none() {
                frame.resolved += 1;
            }
            pending.attachment = Some((node_id, placement));
            pending.container = frame.node;
        }
    }
}

impl<'a> Visit<'a> for AssignmentVisitor<'a, '_> {
    #[inline]
    fn visit_statements(&mut self, statements: &ArenaVec<'a, Statement<'a>>) {
        if self.skipped_depth != 0 {
            return;
        }
        let mut remaining = statements.as_slice();
        while !remaining.is_empty() {
            let parent = self.frames.last().unwrap();
            if parent.leading_end == parent.window.start {
                if parent.cursor == parent.window.end {
                    break;
                }
                // Only the last sibling before a comment can be its previous
                // neighbor. Skip earlier siblings without descending into them.
                let comment_start = self.comments[parent.cursor].span.start;
                let before = remaining.partition_point(|node| node.span().end <= comment_start);
                remaining = &remaining[before.saturating_sub(1)..];
            }
            self.visit_statement(&remaining[0]);
            remaining = &remaining[1..];
        }
    }

    // Union visitors do not enter a node themselves. Once a parent has an empty
    // comment window, stop here before dispatching into its descendants.
    #[inline]
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        if self.skipped_depth == 0 {
            walk::walk_expression(self, expression);
        }
    }

    #[inline]
    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if self.skipped_depth == 0 {
            walk::walk_binding_pattern(self, pattern);
        }
    }

    #[inline]
    fn visit_ts_type(&mut self, ty: &TSType<'a>) {
        if self.skipped_depth == 0 {
            walk::walk_ts_type(self, ty);
        }
    }

    #[inline]
    fn visit_jsx_child(&mut self, child: &JSXChild<'a>) {
        if self.skipped_depth == 0 {
            walk::walk_jsx_child(self, child);
        }
    }

    #[inline]
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        if self.skipped_depth == 0 {
            walk::walk_statement(self, statement);
        }
    }

    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        // Keep the sparse-subtree path here so it can inline into the generated
        // walker without inlining the full ownership algorithm for every node kind.
        if self.skipped_depth != 0 {
            self.skipped_depth += 1;
            return;
        }
        // Raw text cannot receive JavaScript comments. In particular, visiting all
        // template quasis first must not consume substitution comments.
        if matches!(kind, AstKind::TemplateElement(_) | AstKind::JSXText(_)) {
            self.skipped_depth = 1;
            return;
        }
        // Extract these in the inlined walker while the node kind is known,
        // avoiding large AstKind dispatches in the shared ownership algorithm.
        self.enter(kind, kind.span(), kind.node_id());
    }

    #[inline]
    fn leave_node(&mut self, _kind: AstKind<'a>) {
        if self.skipped_depth != 0 {
            self.skipped_depth -= 1;
        } else {
            self.leave();
        }
    }
}

fn carries_leading_comment(
    parent: AstKind<'_>,
    child: AstKind<'_>,
    comment: &Comment,
    inherited: bool,
) -> bool {
    match comment.content {
        CommentContent::PropertyKey => return comment.attached_to == child.span().start,
        CommentContent::Pure | CommentContent::NoSideEffects => {}
        _ => return false,
    }
    if !inherited {
        return true;
    }
    // Stop at the node flagged by the parser, rather than passing the annotation
    // to a call in its arguments or a function in its body.
    let is_target = match parent {
        AstKind::CallExpression(node) => node.pure && comment.content == CommentContent::Pure,
        AstKind::NewExpression(node) => node.pure && comment.content == CommentContent::Pure,
        AstKind::Function(node) => node.pure && comment.content == CommentContent::NoSideEffects,
        AstKind::ArrowFunctionExpression(node) => {
            node.pure && comment.content == CommentContent::NoSideEffects
        }
        _ => false,
    };
    if is_target {
        return false;
    }
    // Follow the expression's leading operand or the declaration's initializer.
    // Other children, such as binding patterns and type arguments, cannot claim
    // an inherited annotation just because they are visited first.
    let target = match parent {
        AstKind::ExpressionStatement(node) => node.expression.span(),
        AstKind::ParenthesizedExpression(node) => node.expression.span(),
        AstKind::TSAsExpression(node) => node.expression.span(),
        AstKind::TSSatisfiesExpression(node) => node.expression.span(),
        AstKind::TSInstantiationExpression(node) => node.expression.span(),
        AstKind::TSNonNullExpression(node) => node.expression.span(),
        AstKind::TSTypeAssertion(node) => node.expression.span(),
        AstKind::ChainExpression(node) => node.expression.span(),
        AstKind::StaticMemberExpression(node) => node.object.span(),
        AstKind::ComputedMemberExpression(node) => node.object.span(),
        AstKind::PrivateFieldExpression(node) => node.object.span(),
        AstKind::CallExpression(node) => node.callee.span(),
        AstKind::NewExpression(node) => node.callee.span(),
        AstKind::BinaryExpression(node) => node.left.span(),
        AstKind::LogicalExpression(node) => node.left.span(),
        AstKind::ConditionalExpression(node) => node.test.span(),
        AstKind::AssignmentExpression(node) => node.left.span(),
        AstKind::UnaryExpression(node) => node.argument.span(),
        AstKind::UpdateExpression(node) => node.argument.span(),
        AstKind::AwaitExpression(node) => node.argument.span(),
        AstKind::SequenceExpression(node) => {
            let Some(expression) = node.expressions.first() else { return false };
            expression.span()
        }
        AstKind::ExportDeclaration(node) if comment.content == CommentContent::NoSideEffects => {
            node.declaration.span()
        }
        AstKind::ExportDefaultDeclaration(node)
            if comment.content == CommentContent::NoSideEffects =>
        {
            node.declaration.span()
        }
        AstKind::VariableDeclaration(node)
            if comment.content == CommentContent::NoSideEffects && node.kind.is_const() =>
        {
            let Some(declaration) = node.declarations.first() else { return false };
            declaration.span
        }
        AstKind::VariableDeclarator(node) if comment.content == CommentContent::NoSideEffects => {
            let Some(init) = &node.init else { return false };
            init.span()
        }
        _ => return false,
    };
    child.span() == target
}

fn effective_span(kind: AstKind<'_>, mut span: Span) -> Span {
    let decorators = match kind {
        AstKind::Class(node) => Some(&node.decorators),
        AstKind::MethodDefinition(node) => Some(&node.decorators),
        AstKind::PropertyDefinition(node) => Some(&node.decorators),
        AstKind::AccessorProperty(node) => Some(&node.decorators),
        AstKind::ExportDeclaration(node) => match &node.declaration {
            Declaration::ClassDeclaration(class) => Some(&class.decorators),
            _ => None,
        },
        AstKind::ExportDefaultDeclaration(node) => match &node.declaration {
            ExportDefaultDeclarationKind::ClassDeclaration(class)
            | ExportDefaultDeclarationKind::ClassExpression(class) => Some(&class.decorators),
            _ => None,
        },
        _ => None,
    };
    if let Some(decorator) = decorators.and_then(|decorators| decorators.first()) {
        span.start = span.start.min(decorator.span.start);
    }
    span
}

fn substitution_span(parent: AstKind<'_>, child: Span) -> Option<Span> {
    let (quasis, index): (&[TemplateElement<'_>], _) = match parent {
        AstKind::TemplateLiteral(template) => {
            let index =
                template.expressions.partition_point(|expr| expr.span().start < child.start);
            if template.expressions.get(index)?.span() != child {
                return None;
            }
            (&template.quasis, index)
        }
        AstKind::TSTemplateLiteralType(template) => {
            let index = template.types.partition_point(|ty| ty.span().start < child.start);
            if template.types.get(index)?.span() != child {
                return None;
            }
            (&template.quasis, index)
        }
        _ => return None,
    };
    // Quasi spans exclude their surrounding `}`, `${`, and backtick tokens.
    Some(Span::new(quasis[index].span.end + 2, quasis[index + 1].span.start - 1))
}
