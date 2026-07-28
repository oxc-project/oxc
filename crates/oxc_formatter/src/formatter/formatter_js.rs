//! JS/TS-specialized impls for [`Formatter`].
//!
//! Because `Formatter` lives in `oxc_formatter_core` we cannot add inherent methods
//! on `Formatter<'_, 'ast, JsFormatContext<'ast>>` directly here (orphan rule). Instead
//! the JS-specific operations are exposed via the [`JsFormatter`] type alias and an
//! extension trait that is blanket-implemented for the alias.

use oxc_span::{GetSpan, Span};

use crate::source_text::SourceTextExt as _;

use crate::formatter::{
    Buffer, Comments, Format, Formatter, GroupId, JsFormatContext, SourceText,
    UniqueGroupIdBuilder,
    builders::{JoinNodesBuilder, Line, Space},
    format_element::FormatElements,
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
    fn group_id_builder(&self) -> &UniqueGroupIdBuilder;
    fn join_nodes_with_soft_line<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line>;
    fn join_nodes_with_hardline<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line>;
    fn join_nodes_with_space<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Space>;
    fn speculate_will_break(
        &mut self,
        content: &(impl Format<'ast, JsFormatContext<'ast>> + GetSpan),
    ) -> bool;
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

    /// Returns a reference to the unique group id builder for this document.
    #[inline]
    fn group_id_builder(&self) -> &UniqueGroupIdBuilder {
        self.state().group_id_builder()
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
