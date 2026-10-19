//! JS/TS-specialized impls for [`Formatter`].
//!
//! Because `Formatter` lives in `oxc_formatter_core` we cannot add inherent methods
//! on `Formatter<'_, 'ast, JsFormatContext<'ast>>` directly here (orphan rule). Instead
//! the JS-specific operations are exposed via the [`JsFormatter`] type alias and an
//! extension trait that is blanket-implemented for the alias.

use oxc_formatter_core::{Format, FormatElements, Formatter, GroupId, SourceText};
use oxc_span::{GetSpan, Span};

use crate::{ast_nodes::AstNodes, source_text::SourceTextExt as _};

use crate::formatter::{
    Comments, JsFormatContext,
    builders::{JoinNodesBuilder, Line, Space},
    prelude::{hard_line_break, soft_line_break_or_space, space, token},
};

/// JS/TS-specialized [`Formatter`].
pub type JsFormatter<'buf, 'ast> = Formatter<'buf, 'ast, JsFormatContext<'ast>>;

/// Extension trait that adds JS/TS-specific convenience methods to [`JsFormatter`].
///
/// This is implemented for `Formatter<'buf, 'ast, JsFormatContext<'ast>>`. It exists
/// because `Formatter` is defined in `oxc_formatter_core`, so adding inherent methods
/// here would violate the orphan rule.
pub trait JsFormatterExt<'buf, 'ast> {
    fn source_text(&self) -> SourceText<'ast>;
    fn comments(&self) -> &Comments<'_>;
    fn lines_before(&self, span: Span) -> usize;
    fn group_id(&self, debug_name: &'static str) -> GroupId;
    fn join_nodes_with_soft_line<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line>;
    fn join_nodes_with_hardline<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line>;
    fn join_nodes_with_space<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Space>;
    fn speculate_will_break(
        &mut self,
        content: &(impl Format<'ast, JsFormatContext<'ast>> + GetSpan),
    ) -> bool;
    fn with_node<R>(
        &mut self,
        node: AstNodes<'ast>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R;
    fn with_current_node<R>(
        &mut self,
        node: AstNodes<'ast>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R;
    fn with_nodes<R>(
        &mut self,
        nodes: impl IntoIterator<Item = AstNodes<'ast>>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R;
    fn outside_current_node<R>(
        &mut self,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R;
}

impl<'buf, 'ast> JsFormatterExt<'buf, 'ast> for Formatter<'buf, 'ast, JsFormatContext<'ast>> {
    /// Returns the source text wrapper.
    #[inline]
    fn source_text(&self) -> SourceText<'ast> {
        self.context().source_text()
    }

    /// Returns the comments from the context.
    #[inline]
    fn comments(&self) -> &Comments<'_> {
        self.context().comments()
    }

    /// Returns the number of line breaks before `span`, counting the leading trivia
    /// of the first not-yet-printed comment. See [`SourceText::get_lines_before`].
    #[inline]
    fn lines_before(&self, span: Span) -> usize {
        self.source_text().get_lines_before(span, self.comments().first_unprinted_span())
    }

    /// Creates a new group id that is unique to this document.
    #[inline]
    fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.state().group_id(debug_name)
    }

    fn join_nodes_with_soft_line<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line> {
        JoinNodesBuilder::new(soft_line_break_or_space(), self)
    }

    fn join_nodes_with_hardline<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line> {
        JoinNodesBuilder::new(hard_line_break(), self)
    }

    fn join_nodes_with_space<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Space> {
        JoinNodesBuilder::new(space(), self)
    }

    fn speculate_will_break(
        &mut self,
        content: &(impl Format<'ast, JsFormatContext<'ast>> + GetSpan),
    ) -> bool {
        let snapshot = self.context().comments().snapshot();
        self.context_mut().comments_mut().skip_comments_before(content.span().start);
        let will_break = self.intern(content).is_some_and(|e| e.will_break());
        self.context_mut().comments_mut().restore(snapshot);
        will_break
    }

    /// Runs `content` with `node` on the ancestor stack, which must be a child of the current node.
    #[inline]
    fn with_node<R>(
        &mut self,
        node: AstNodes<'ast>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R {
        self.context_mut().ancestors_mut().push(node);
        let result = content(self);
        self.context_mut().ancestors_mut().pop();
        result
    }

    /// [`Self::with_node`], or just `content` when `node` is the current node already:
    /// for a printer of `node` called both from its `fmt` and from its parent's.
    #[inline]
    fn with_current_node<R>(
        &mut self,
        node: AstNodes<'ast>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R {
        let is_current = self.context().ancestors().current().is(&node);
        if !is_current {
            self.context_mut().ancestors_mut().push(node);
        }
        let result = content(self);
        if !is_current {
            self.context_mut().ancestors_mut().pop();
        }
        result
    }

    /// Runs `content` with `nodes` on the ancestor stack, each a child of the one before (the first of the current node).
    /// For printing a descendant of the current node outside of its ancestors' `fmt`.
    fn with_nodes<R>(
        &mut self,
        nodes: impl IntoIterator<Item = AstNodes<'ast>>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R {
        let mut count = 0;
        for node in nodes {
            self.context_mut().ancestors_mut().push(node);
            count += 1;
        }
        let result = content(self);
        for _ in 0..count {
            self.context_mut().ancestors_mut().pop();
        }
        result
    }

    /// Runs `content` with the current node off the ancestor stack,
    /// for printing a child of its parent in its `fmt` (a sibling field formatted inside it).
    fn outside_current_node<R>(
        &mut self,
        content: impl FnOnce(&mut Formatter<'_, 'ast, JsFormatContext<'ast>>) -> R,
    ) -> R {
        let current = self.context_mut().ancestors_mut().pop();
        let result = content(self);
        self.context_mut().ancestors_mut().push_unchecked(current);
        result
    }
}

/// `Format` impl for `&'static str` specialized to `JsFormatContext`.
///
/// Hardcoded to `JsFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap (`str` doesn't impl `Format` for any C).
/// Uses `Token` (not `Text`) so downstream IR transforms (e.g. `sort_imports`)
/// can match on token text shape.
impl<'ast> Format<'ast, JsFormatContext<'ast>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_, 'ast, JsFormatContext<'ast>>) {
        crate::write!(f, token(self));
    }
}
