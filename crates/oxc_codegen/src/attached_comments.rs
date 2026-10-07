use std::ops::Range;

use oxc_ast::{Comment, CommentContent, CommentPlacement, ast::Expression};
use oxc_syntax::node::NodeId;

use crate::{
    Codegen,
    comment::{AnnotationKind, preserve_when_orphaned},
};

struct AttachedComment {
    node_id: NodeId,
    placement: CommentPlacement,
    comment: Comment,
    printed: bool,
    // Only the first record for each owner uses this flag. Enum and struct
    // printers can visit the same ID; the outermost printer owns the comments.
    claimed: bool,
}

/// Scratch storage proportional to retained comments, with no per-node table.
///
/// Records are sorted by owner, placement, and source order. Entering an ordered
/// node advances the cursor; an earlier ID uses binary search without rewinding it.
/// Claimed ranges stay available while children print, so trailing comments do
/// not require a second lookup when returning to an enclosing node.
#[derive(Default)]
pub struct AttachedComments {
    records: Vec<AttachedComment>,
    cursor: usize,
    high_water: NodeId,
    has_statement_comments: bool,
    current: Range<usize>,
    current_dangling: usize,
    frames: Vec<CommentFrame>,
}

/// Restore the enclosing owner's range and dangling cursor after a child prints.
/// For `f(/* argument */ x) /* call */ + y`, the argument's frame temporarily replaces
/// the call's frame; returning to the call then emits its trailing comment.
/// Only comment owners create frames. Printers carry a small node-ID token.
struct CommentFrame {
    owner: NodeId,
    binary: bool,
    leading_deferred: bool,
    previous: Range<usize>,
    previous_dangling: usize,
}

impl AttachedComments {
    pub fn reserve(&mut self, len: usize) {
        self.records.reserve(len);
    }

    pub fn push(&mut self, comment: Comment) {
        // Applied annotations attach to expressions/functions, so they cannot
        // change a statement boundary when its final semicolon is omitted.
        self.has_statement_comments |= !comment.is_pure() && !comment.is_no_side_effects();
        self.records.push(AttachedComment {
            node_id: comment.node_id.get(),
            placement: comment.placement,
            comment,
            printed: false,
            claimed: false,
        });
    }

    pub fn sort(&mut self) {
        self.records.sort_unstable_by_key(|entry| {
            (entry.node_id, entry.placement, entry.comment.span.start)
        });
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    #[inline]
    pub fn has_statement_comments(&self) -> bool {
        self.has_statement_comments
    }

    fn find(&self, node_id: NodeId) -> Range<usize> {
        let start = self.records.partition_point(|entry| entry.node_id < node_id);
        let end = start + self.records[start..].partition_point(|entry| entry.node_id == node_id);
        start..end
    }

    fn claim(&mut self, node_id: NodeId) -> Option<Range<usize>> {
        let range = if node_id >= self.high_water {
            self.high_water = node_id;
            while self.cursor < self.records.len() && self.records[self.cursor].node_id < node_id {
                self.cursor += 1;
            }
            let start = self.cursor;
            while self.cursor < self.records.len() && self.records[self.cursor].node_id == node_id {
                self.cursor += 1;
            }
            start..self.cursor
        } else {
            self.find(node_id)
        };
        if range.is_empty() || self.records[range.start].claimed {
            return None;
        }
        self.records[range.start].claimed = true;
        Some(range)
    }

    #[inline]
    pub fn has_comments(&self, node_id: NodeId) -> bool {
        if self.is_empty() {
            return false;
        }
        self.records[self.find(node_id)].iter().any(|entry| !entry.printed)
    }
}

impl Codegen<'_> {
    /// An annotation that the parser did not apply must not become applicable
    /// merely because ownership prints it immediately before a call or function.
    pub(crate) fn discard_unapplied_attached_annotation(
        &mut self,
        node_id: NodeId,
        kind: AnnotationKind,
    ) {
        let range = self.attached_comments.find(node_id);
        for entry in &mut self.attached_comments.records[range] {
            let unapplied = match kind {
                AnnotationKind::Pure => entry.comment.content == CommentContent::PureNotApplied,
                AnnotationKind::NoSideEffects => {
                    entry.comment.content == CommentContent::NoSideEffectsNotApplied
                }
            };
            if unapplied || kind.matches(&entry.comment) {
                entry.printed = true;
            }
        }
    }

    /// Establish ownership once, even when enum and struct printers share an ID.
    #[inline]
    pub(crate) fn start_node_comments(&mut self, node_id: NodeId) -> Option<NodeId> {
        self.start_node_comments_with_deferred_leading(node_id, |_| false)
    }

    #[inline]
    pub(crate) fn start_node_comments_with_deferred_leading(
        &mut self,
        node_id: NodeId,
        defer_leading: impl FnOnce(&Self) -> bool,
    ) -> Option<NodeId> {
        // Some printers call this directly rather than through Gen/GenExpr.
        // Leave both cursor and owner state untouched on the unassigned path.
        if self.attached_comments.is_empty() {
            return None;
        }
        // An ordered node with no comments stops here, without entering the
        // owner printer or searching the table. This is the sparse hot path.
        // claim updates the watermark when records are consumed or skipped;
        // comment-free nodes do not need to write lookup state.
        if node_id >= self.attached_comments.high_water {
            let next = self.attached_comments.records.get(self.attached_comments.cursor)?;
            if next.node_id > node_id {
                return None;
            }
        }
        let defer_leading = defer_leading(self);
        self.start_node_comments_slow(node_id, defer_leading)
    }

    #[inline(never)]
    fn start_node_comments_slow(&mut self, node_id: NodeId, defer_leading: bool) -> Option<NodeId> {
        let range = self.attached_comments.claim(node_id)?;
        let dangling = range.start
            + self.attached_comments.records[range.clone()]
                .partition_point(|entry| entry.placement < CommentPlacement::Dangling);
        let previous_dangling =
            std::mem::replace(&mut self.attached_comments.current_dangling, dangling);
        let previous = std::mem::replace(&mut self.attached_comments.current, range);
        // A hashbang must come before file-level leading comments. Program's
        // printer emits these after its hashbang instead.
        // Wrapping printers defer the whole leading group, so neighboring
        // comments keep their source order inside generated parentheses.
        if node_id != NodeId::ROOT && !defer_leading {
            let offset = self.code.len();
            let statement = self.start_of_stmt == offset;
            let arrow = self.start_of_arrow_expr == offset;
            let export = self.start_of_default_export == offset;
            self.print_attached_placement(CommentPlacement::Leading);
            if self.last_byte() == Some(b'\n') {
                self.print_indent();
            } else {
                self.consume_pending_indent_space();
            }
            // Leading comments do not change the expression's syntactic position.
            let offset = self.code.len();
            if statement {
                self.start_of_stmt = offset;
            }
            if arrow {
                self.start_of_arrow_expr = offset;
            }
            if export {
                self.start_of_default_export = offset;
            }
        }
        self.attached_comments.frames.push(CommentFrame {
            owner: node_id,
            binary: false,
            leading_deferred: defer_leading,
            previous,
            previous_dangling,
        });
        Some(node_id)
    }

    /// Emit an expression owner's leading group after its generated opening paren.
    #[inline]
    pub(crate) fn print_deferred_leading_comments(&mut self, node_id: NodeId) {
        let Some(frame) = self.attached_comments.frames.last() else { return };
        if frame.owner != node_id || !frame.leading_deferred {
            return;
        }
        self.print_deferred_leading_comments_slow();
    }

    #[inline(never)]
    fn print_deferred_leading_comments_slow(&mut self) {
        self.attached_comments.frames.last_mut().unwrap().leading_deferred = false;
        self.print_attached_placement(CommentPlacement::Leading);
        if self.last_byte() == Some(b'\n') {
            self.print_indent();
        } else {
            self.consume_pending_indent_space();
        }
    }

    /// Keep an expression's trailing comments inside its generated parentheses,
    /// so reparsing cannot transfer them to a disposable parenthesized node.
    #[inline]
    pub(crate) fn print_trailing_comments_inside_parens(&mut self, node_id: NodeId) {
        if self.attached_comments.frames.last().is_some_and(|frame| frame.owner == node_id) {
            self.print_trailing_comments_inside_parens_slow();
        }
    }

    #[inline(never)]
    fn print_trailing_comments_inside_parens_slow(&mut self) {
        self.print_attached_placement(CommentPlacement::Trailing);
    }

    #[inline]
    pub(crate) fn finish_node_comments(&mut self, owner: Option<NodeId>) {
        if let Some(owner) = owner {
            self.finish_node_comments_slow(owner);
        }
    }

    #[inline(never)]
    fn finish_node_comments_slow(&mut self, owner: NodeId) {
        let frame = self.attached_comments.frames.pop().unwrap();
        debug_assert_eq!(frame.owner, owner);
        let has_trailing = self.attached_comments.records[self.attached_comments.current.clone()]
            .iter()
            .any(|entry| !entry.printed && entry.placement == CommentPlacement::Trailing);
        if has_trailing {
            self.print_semicolon_if_needed();
            let newline = self.last_byte() == Some(b'\n');
            if newline {
                self.code.truncate(self.code.len() - 1);
            }
            self.print_attached_placement(CommentPlacement::Trailing);
            if newline {
                self.print_next_indent_as_space = false;
                if self.last_byte() != Some(b'\n') {
                    self.print_soft_newline();
                }
            }
        }
        self.print_next_indent_as_space = false;
        self.attached_comments.current = frame.previous;
        self.attached_comments.current_dangling = frame.previous_dangling;
    }

    pub(crate) fn print_attached_placement(&mut self, placement: CommentPlacement) {
        // Dangling comments are source ordered at the end of the owner range.
        // The owner's content printer emits the whole range once.
        if placement == CommentPlacement::Dangling {
            while self.attached_comments.current_dangling < self.attached_comments.current.end {
                let index = self.attached_comments.current_dangling;
                let entry = &mut self.attached_comments.records[index];
                self.attached_comments.current_dangling += 1;
                if entry.printed {
                    continue;
                }
                entry.printed = true;
                let comment = entry.comment.clone();
                self.print_attached_comment(&comment);
            }
            return;
        }
        for index in self.attached_comments.current.clone() {
            let entry = &mut self.attached_comments.records[index];
            if entry.printed || entry.placement != placement {
                continue;
            }
            entry.printed = true;
            let comment = entry.comment.clone();
            self.print_attached_comment(&comment);
        }
    }

    fn print_attached_comment(&mut self, comment: &Comment) {
        // Source hints decide line breaks, while generated whitespace decides
        // indentation and spacing. Never carry a pending space across a newline.
        self.print_next_indent_as_space = false;
        let at_line_start = self
            .code()
            .as_bytes()
            .iter()
            .rev()
            .find(|&&byte| !matches!(byte, b' ' | b'\t'))
            .is_none_or(|&byte| byte == b'\n');
        if comment.preceded_by_newline() && !at_line_start {
            // An earlier inline comment may have consumed a pending soft space.
            // A following comment's source newline supersedes that separator.
            while self.last_byte().is_some_and(|byte| matches!(byte, b' ' | b'\t')) {
                self.code.truncate(self.code.len() - 1);
            }
            #[cfg(feature = "sourcemap")]
            if let Some(builder) = &mut self.sourcemap_builder {
                builder.truncate_generated_whitespace(self.code.len());
            }
            self.print_hard_newline();
        }
        if self.last_byte().is_none_or(|byte| byte == b'\n') {
            self.print_indent();
        } else if self
            .last_byte()
            .is_some_and(|byte| !matches!(byte, b' ' | b'\t' | b'(' | b'[' | b'{'))
        {
            // A division followed immediately by a block comment would become //.
            if self.last_byte() == Some(b'/') {
                self.print_hard_space();
            } else {
                self.print_soft_space();
            }
        }
        self.print_comment(comment);
        if comment.is_line() || comment.followed_by_newline() {
            self.print_hard_newline();
        } else {
            self.print_next_indent_as_space = true;
        }
    }

    /// A line break after return/throw/yield must stay inside parentheses. Follow
    /// the first printed operand as annotations can belong to a nested callee.
    pub(crate) fn expression_starts_with_comment_newline(
        &self,
        mut expression: &Expression<'_>,
    ) -> bool {
        if self.attached_comments.is_empty() {
            return false;
        }
        loop {
            if self.attached_comments.records[self.attached_comments.find(expression.node_id())]
                .iter()
                .any(|entry| {
                    !entry.printed
                        && entry.placement == CommentPlacement::Leading
                        && (entry.comment.is_line()
                            || entry.comment.is_multiline_block()
                            || entry.comment.preceded_by_newline()
                            || entry.comment.followed_by_newline())
                })
            {
                return true;
            }
            expression = match expression {
                Expression::BinaryExpression(node) => &node.left,
                Expression::LogicalExpression(node) => &node.left,
                Expression::ConditionalExpression(node) => &node.test,
                Expression::CallExpression(node) => &node.callee,
                Expression::StaticMemberExpression(node) => &node.object,
                Expression::ComputedMemberExpression(node) => &node.object,
                Expression::PrivateFieldExpression(node) => &node.object,
                Expression::SequenceExpression(node) => {
                    let Some(first) = node.expressions.first() else {
                        return false;
                    };
                    first
                }
                Expression::TSAsExpression(node) => &node.expression,
                Expression::TSSatisfiesExpression(node) => &node.expression,
                Expression::TSNonNullExpression(node) => &node.expression,
                Expression::TSInstantiationExpression(node) => &node.expression,
                // Explicit parentheses are preserved, so comments within them
                // cannot introduce a line break after the enclosing keyword.
                _ => return false,
            };
        }
    }

    /// Iterative binary printing keeps only comment-bearing frames on this stack.
    /// The ordinary binary visitor stays the same size, and comment-free chains
    /// need no extra allocation or recursive calls.
    pub(crate) fn start_binary_comments(
        &mut self,
        node_id: NodeId,
        defer_leading: impl FnOnce(&Self) -> bool,
    ) {
        if self.start_node_comments_with_deferred_leading(node_id, defer_leading).is_some() {
            self.attached_comments.frames.last_mut().unwrap().binary = true;
        }
    }

    pub(crate) fn finish_binary_comments(&mut self, node_id: NodeId) {
        if self
            .attached_comments
            .frames
            .last()
            .is_some_and(|frame| frame.binary && frame.owner == node_id)
        {
            self.finish_node_comments(Some(node_id));
        }
    }

    /// Whether this node owns unprinted comments inside its syntax.
    /// Program emits its file comments separately; generated nodes may also use ID 0.
    #[inline]
    pub(crate) fn has_inside_comments(&self, node_id: NodeId) -> bool {
        node_id != NodeId::ROOT
            && self.attached_comments.frames.last().is_some_and(|frame| frame.owner == node_id)
            && self.attached_comments.current_dangling < self.attached_comments.current.end
    }

    /// Print this node's dangling comments as part of its contents.
    /// Call once inside the containing syntax, alongside its children.
    #[inline]
    pub(crate) fn print_inside_comments(&mut self, node_id: NodeId) -> bool {
        if !self.has_inside_comments(node_id) {
            return false;
        }
        self.print_attached_placement(CommentPlacement::Dangling);
        self.print_next_indent_as_space = false;
        true
    }

    /// AST flags can request a synthesized annotation, but a source annotation
    /// already printed for that node must not be printed a second time.
    pub(crate) fn print_attached_annotation(
        &mut self,
        node_id: NodeId,
        kind: AnnotationKind,
    ) -> bool {
        if self.attached_comments.is_empty() {
            return false;
        }
        let range = self.attached_comments.find(node_id);
        let mut found = false;
        for index in range {
            let entry = &mut self.attached_comments.records[index];
            if !kind.matches(&entry.comment) {
                continue;
            }
            found = true;
            if !entry.printed {
                entry.printed = true;
                let comment = entry.comment.clone();
                self.print_attached_comment(&comment);
            }
        }
        found
    }

    /// Preserve legal and file-level comments whose owners disappeared, or whose
    /// syntax was elided by the printer. Discard other unclaimed comments.
    pub(crate) fn print_unclaimed_attached_comments(&mut self) {
        let mut comments: Vec<_> = self
            .attached_comments
            .records
            .iter_mut()
            .filter_map(|entry| {
                if entry.printed {
                    return None;
                }
                entry.printed = true;
                if !preserve_when_orphaned(&entry.comment) {
                    return None;
                }
                Some(entry.comment.clone())
            })
            .collect();
        comments.sort_unstable_by_key(|comment| comment.span.start);
        for mut comment in comments {
            comment.set_preceded_by_newline(true);
            comment.set_followed_by_newline(true);
            self.print_attached_comment(&comment);
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_ast::CommentPlacement;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::{Codegen, Context, Gen};

    #[test]
    fn live_owners_print_without_eof_fallback() {
        for source in [
            "/* leading */ f(); // tail\n/* eof */",
            "(/* string */ 'value' /* string end */);",
            "const a = [/* array */], b = { /* object */ }; new C(/* new */); f(/* call */);",
            "function f(/* param */ x /* params end */) { /* body */ }",
            "const o = { /* shorthand */ a, m(/* param */) { /* method */ } };",
            "const { /* binding */ a } = o; const { b: /* alias */ b } = o;",
            "import { /* import */ a as /* local */ b } from /* source */ 'mod'; export { /* export */ b /* end */ };",
            "import { /* empty */ } from 'mod'; export { /* export empty */ };",
            "import(/* webpackChunkName: 'chunk' */ 'mod' /* end */);",
            "class C extends /* super */ Base { /* member */ m(/* param */) { /* body */ } }",
            "try { /* try */ } catch (/* error */ err) { /* catch */ }",
            "const t = `a${/* before */ value /* after */}b`;",
            "const view = <C /* attr */ a={/* value */ x}>{/* child */}</C>;",
            "type T = { /* empty */ }; type U = [/* tuple */]; enum E { /* enum */ }",
            "const f = (/* empty */) => x; value /* postfix */ ++;",
            "interface I { /* body */ } type F = (/* params */ p: T) /* signature */ => T;",
            "[/* hole */ , value, /* end */]; const { /* key */ a = /* default */ 1 } = obj;",
            "const fragment = < /* open */ ></ /* close */>;",
            "switch (value) { case 1: /* empty case */ default: /* final case */ }",
            "for (/* init */ let i = 0; /* test */ i < 1; /* update */ i++) { /* body */ }",
            "if (/* test */ value /* test end */) first(); /* then */ else second();",
            "const o = { [/* key */ value /* key end */]: item /* property end */ };",
            "const o = { ...value /* spread end */ }; [ ...items /* spread end */ ];",
            "type T = (/* parenthesized */ U /* end */); type M = T[/* key */ K /* end */];",
            "type T = import(/* import type */ 'mod' /* end */).T;",
            "type M = { [K in keyof T /* key end */]: T[K] /* mapped end */ };",
            "function f() { return /* return */; } debugger /* debugger */;",
            "class C { [/* key */ value /* end */]() {} static { /* static */ } }",
        ] {
            let allocator = Allocator::default();
            let program = Parser::new(&allocator, source, SourceType::tsx()).parse().program;

            let mut codegen = Codegen::new().with_source_text(source);
            codegen.build_comments(&program.comments);
            program.print(&mut codegen, Context::default());
            let remaining: Vec<_> = codegen
                .attached_comments
                .records
                .iter()
                .filter(|entry| !entry.printed)
                .map(|entry| entry.comment.span.source_text(source))
                .collect();
            assert!(
                remaining.is_empty(),
                "{source}\nPrinted:\n{}\nRemaining: {remaining:?}",
                codegen.code().as_str()
            );
            assert!(codegen.attached_comments.current.is_empty());
            assert!(codegen.attached_comments.frames.is_empty());
        }
    }

    #[test]
    fn ordered_cursor_and_backward_lookup() {
        let allocator = Allocator::default();
        let source = "/* a */ a(); /* b */ b(); /* c */ c();";
        let program = Parser::new(&allocator, source, SourceType::mjs()).parse().program;

        let mut codegen = Codegen::new().with_source_text(source);
        codegen.build_comments(&program.comments);
        let ids: Vec<_> = program.comments.iter().map(|comment| comment.node_id.get()).collect();
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        let table = &mut codegen.attached_comments;
        assert!(table.claim(ids[2]).is_some());
        let end = table.cursor;
        assert!(table.claim(ids[0]).is_some());
        assert!(table.claim(ids[1]).is_some());
        assert_eq!(table.cursor, end);
        assert!(table.claim(ids[0]).is_none());
        assert!(table.records.iter().all(|entry| entry.placement == CommentPlacement::Leading));
    }
}
