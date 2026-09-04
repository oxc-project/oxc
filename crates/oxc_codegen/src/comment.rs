use std::borrow::Cow;

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use oxc_ast::{
    Comment, CommentKind,
    ast::{Expression, Program},
};
use oxc_ast_visit::{CommentAttachments, CommentPlacement};
use oxc_span::GetSpan;
use oxc_syntax::line_terminator::LineTerminatorSplitter;
use oxc_syntax::node::NodeId;

use crate::{Codegen, LegalComment, options::CommentOptions};

type CommentList = SmallVec<[Comment; 1]>;

pub type CommentsMap = FxHashMap</* attached_to */ u32, CommentList>;

/// Destructive claim state kept by codegen alongside the immutable sidecar.
///
/// This costs one bit per actively attached comment and avoids copying either
/// the parser's [`Comment`] records or the attachment mapping.
pub struct AttachedComments<'a> {
    attachments: &'a CommentAttachments,
    claimed: Box<[u64]>,
}

impl<'a> AttachedComments<'a> {
    pub fn new(attachments: &'a CommentAttachments) -> Self {
        let claimed = vec![0; attachments.active_comment_count().div_ceil(u64::BITS as usize)]
            .into_boxed_slice();
        Self { attachments, claimed }
    }

    pub fn comment_range(&self, node_id: NodeId) -> Option<std::ops::Range<usize>> {
        self.attachments.comments_for_with_range(node_id).map(|(range, _)| range)
    }

    fn has_printable_comments(
        &self,
        range: std::ops::Range<usize>,
        placement: CommentPlacement,
        source_comments: &[Comment],
        options: &crate::CodegenOptions,
    ) -> bool {
        self.attachments.comments_in_range(range).iter().any(|attached| {
            if attached.placement != placement {
                return false;
            }
            let comment = *attached.comment(source_comments);
            !comment.is_pure()
                && !comment.is_no_side_effects()
                && should_print_attached_comment(options, comment)
        })
    }

    fn take_boundary_comments(
        &mut self,
        range: std::ops::Range<usize>,
        placement: CommentPlacement,
        source_comments: &[Comment],
        options: &crate::CodegenOptions,
    ) -> CommentList {
        let attached_comments = self.attachments.comments_in_range(range.clone());

        let mut comments = CommentList::new();
        for (offset, attached) in attached_comments.iter().enumerate() {
            if attached.placement != placement {
                continue;
            }

            let attachment_index = range.start + offset;
            let word_index = attachment_index / u64::BITS as usize;
            let mask = 1 << (attachment_index % u64::BITS as usize);
            if self.claimed[word_index] & mask != 0 {
                continue;
            }

            let comment = *attached.comment(source_comments);
            // PURE and NO_SIDE_EFFECTS are claimed by their semantic emission
            // sites in a later integration step. Consuming them at the generic
            // boundary would lose their verbatim source spelling.
            if comment.is_pure() || comment.is_no_side_effects() {
                continue;
            }

            self.claimed[word_index] |= mask;
            if should_print_attached_comment(options, comment) {
                comments.push(comment);
            }
        }
        comments
    }
}

fn should_print_attached_comment(options: &crate::CodegenOptions, comment: Comment) -> bool {
    if comment.is_legal() {
        options.print_legal_comment()
    } else if comment.is_jsdoc() {
        options.print_jsdoc_comment()
    } else if comment.is_annotation() {
        options.print_annotation_comment()
    } else {
        options.print_normal_comment()
    }
}

/// Whether a comment remains meaningful if its original AST anchor is removed.
fn preserve_when_orphaned(comment: Comment) -> bool {
    comment.is_legal() || comment.is_coverage_ignore_file()
}

/// A `pife`-marked arrow or function expression prints its leading comments
/// inside its own `(` wrap, so operand emission sites must not consume them.
fn is_pife_function(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => arrow.pife,
        Expression::FunctionExpression(function) => function.pife,
        _ => false,
    }
}

/// Which annotation kind an emission site expects to recover from
/// [`Codegen::annotation_comments`].
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
    pub(crate) fn print_attached_comments_before(&mut self, range: std::ops::Range<usize>) {
        self.print_attached_comments(range, CommentPlacement::Before);
    }

    pub(crate) fn print_attached_comments_after(&mut self, range: std::ops::Range<usize>) {
        self.print_attached_comments(range, CommentPlacement::After);
    }

    pub(crate) fn has_attached_comments_inside(&self, node_id: NodeId) -> bool {
        let Some(source_comments) = self.source_comments else { return false };
        let Some(attached_comments) = &self.attached_comments else { return false };
        let Some(range) = attached_comments.comment_range(node_id) else { return false };
        attached_comments.has_printable_comments(
            range,
            CommentPlacement::Inside,
            source_comments,
            &self.options,
        )
    }

    pub(crate) fn print_attached_comments_inside(&mut self, node_id: NodeId) -> bool {
        let Some(attached_comments) = &self.attached_comments else { return false };
        let Some(range) = attached_comments.comment_range(node_id) else { return false };
        self.print_attached_comments(range, CommentPlacement::Inside)
    }

    fn print_attached_comments(
        &mut self,
        range: std::ops::Range<usize>,
        placement: CommentPlacement,
    ) -> bool {
        let Some(source_comments) = self.source_comments else { return false };
        let Some(attached_comments) = &mut self.attached_comments else { return false };
        let comments = attached_comments.take_boundary_comments(
            range,
            placement,
            source_comments,
            &self.options,
        );
        if comments.is_empty() {
            return false;
        }
        if placement == CommentPlacement::After {
            // In minified statement printers the semicolon is deferred. It
            // must precede a trailing comment, especially a line comment.
            self.print_semicolon_if_needed();
        }
        self.print_comments(&comments);
        true
    }

    pub(crate) fn build_comments(&mut self, comments: &[Comment]) {
        if self.options.comments == CommentOptions::disabled() {
            return;
        }
        // Each retained comment can create at most one map entry. Reserving
        // this upper bound avoids incremental map growth while preprocessing.
        self.comments.reserve(comments.len());
        for comment in comments {
            // Stash pure / no-side-effects comments by `attached_to` so the
            // emission site can recover the verbatim source text instead of
            // falling back to the canonical literal (rolldown#9408).
            // Best-effort: when several annotation comments share an
            // `attached_to`, only the last survives; the emission site falls
            // back to the canonical literal for the dropped ones.
            if comment.is_pure() || comment.is_no_side_effects() {
                if comment.is_leading() && self.options.print_annotation_comment() {
                    self.annotation_comments.insert(comment.attached_to, *comment);
                }
                continue;
            }

            let mut add = false;
            if comment.is_leading() {
                add = (comment.is_legal() && self.options.print_legal_comment())
                    || (comment.is_jsdoc() && self.options.print_jsdoc_comment())
                    || (comment.is_annotation() && self.options.print_annotation_comment())
                    || (comment.is_normal() && self.options.print_normal_comment());
            }

            if add {
                self.has_property_key_annotations |= comment.is_property_key_annotation();
                if preserve_when_orphaned(*comment)
                    && let Err(idx) = self.orphan_comment_keys.binary_search(&comment.attached_to)
                {
                    self.orphan_comment_keys.insert(idx, comment.attached_to);
                }
                self.comments.entry(comment.attached_to).or_default().push(*comment);
            }
        }
    }

    pub(crate) fn has_comment(&self, start: u32) -> bool {
        self.comments.contains_key(&start)
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
        if self.source_text.is_some()
            && let Some(comment) = self.annotation_comments.get(&start).copied()
            && kind.matches(&comment)
            // Inline line comments would swallow the rest of the line.
            && (!comment.is_line() || newline_after)
        {
            self.annotation_comments.remove(&start);
            self.print_comment(&comment);
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
        if let Some(comments) = self.comments.remove(&start) {
            self.print_comments(&comments);
        }
    }

    pub(crate) fn get_comments(&mut self, start: u32) -> Option<CommentList> {
        if self.comments.is_empty() {
            return None;
        }
        self.comments.remove(&start)
    }

    #[inline]
    pub(crate) fn print_comments_at(&mut self, start: u32) {
        if let Some(comments) = self.get_comments(start) {
            self.print_comments(&comments);
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
        if let Some(comments) = self.get_comments(start) {
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
        if self.comments.get(&start).is_some_and(|comments| {
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
        if self.comments.is_empty() || is_pife_function(expression) {
            return;
        }
        self.print_leading_comments_anchored_to_self(expression.span().start);
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
        if self.comments.is_empty() || is_pife_function(expression) {
            return;
        }
        let start = expression.span().start;
        if self
            .comments
            .get(&start)
            .is_some_and(|comments| comments.iter().any(|comment| comment.is_annotation()))
        {
            self.print_leading_comments_anchored_to_self(start);
        }
        if let Expression::ParenthesizedExpression(paren) = expression {
            self.print_annotation_comments_before_expression(&paren.expression);
        }
    }

    /// Whether an orphan comment with `attached_to < end` is still pending.
    /// Used by block emitters to keep an empty body multi-line.
    #[inline]
    pub(crate) fn has_orphan_comments_before(&self, end: u32) -> bool {
        self.orphan_comment_keys
            .iter()
            .take_while(|&&k| k < end)
            .any(|k| self.comments.contains_key(k))
    }

    /// Drain pending orphan comments with `attached_to < end` and emit them in
    /// source order. Called at every statement boundary so legal and file-level
    /// coverage comments survive when their original anchor was removed by an
    /// upstream pass.
    #[inline]
    pub(crate) fn print_orphan_comments_before(&mut self, end: u32) {
        if self.orphan_comment_keys.is_empty() {
            return;
        }
        let idx = self.orphan_comment_keys.partition_point(|&k| k < end);
        if idx == 0 {
            return;
        }
        // Concatenate across keys so `print_comments` sees one sequence;
        // per-key calls would leak `print_next_indent_as_space` and produce
        // stray leading spaces.
        let mut orphans: Vec<Comment> = Vec::new();
        let comments = &mut self.comments;
        for k in self.orphan_comment_keys.drain(..idx) {
            let Some(entry) = comments.get_mut(&k) else { continue };
            debug_assert!(entry.iter().any(|c| preserve_when_orphaned(*c)));
            entry.retain(|comment| {
                if preserve_when_orphaned(*comment) {
                    orphans.push(*comment);
                    false
                } else {
                    true
                }
            });
            if entry.is_empty() {
                comments.remove(&k);
            }
        }
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
        if self.comments.is_empty() {
            return false;
        }
        // Find and remove the first key in the range.
        let key = self.comments.keys().find(|&&k| k > start && k < end).copied();
        if let Some(key) = key {
            let comments = self.comments.remove(&key).unwrap();
            self.print_comments(&comments);
            return true;
        }
        false
    }

    pub(crate) fn print_expr_comments(&mut self, start: u32) -> bool {
        if self.comments.is_empty() {
            return false;
        }
        let Some(comments) = self.comments.remove(&start) else { return false };

        for comment in &comments {
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
