//! JS/TS-specialized impls for [`Formatter`].
//!
//! Because `Formatter` lives in `oxc_formatter_core` we cannot add inherent methods
//! on `Formatter<'_, 'ast, JsFormatContext<'ast>>` directly here (orphan rule). Instead
//! the JS-specific operations are exposed via the [`JsFormatter`] type alias and an
//! extension trait that is blanket-implemented for the alias.

use oxc_span::GetSpan;

use crate::{
    formatter::{
        Buffer, Comments, Format, Formatter, GroupId, JsFormatContext, SourceText,
        UniqueGroupIdBuilder,
        builders::{FillBuilder, JoinBuilder, JoinNodesBuilder, Line, Space},
        format_element::FormatElements,
        prelude::{hard_line_break, soft_line_break_or_space, space},
    },
    options::JsFormatOptions,
};

/// JS/TS-specialized [`Formatter`].
pub type JsFormatter<'buf, 'ast> = Formatter<'buf, 'ast, JsFormatContext<'ast>>;

/// Extension trait that adds JS/TS-specific convenience methods to [`JsFormatter`].
///
/// This is implemented for `Formatter<'buf, 'ast, JsFormatContext<'ast>>`. It exists
/// because `Formatter` is defined in `oxc_formatter_core`, so adding inherent methods
/// here would violate the orphan rule.
pub trait JsFormatterExt<'buf, 'ast> {
    fn options(&self) -> &JsFormatOptions;
    fn source_text(&self) -> SourceText<'ast>;
    fn comments(&self) -> &Comments<'_>;
    fn group_id(&self, debug_name: &'static str) -> GroupId;
    fn group_id_builder(&self) -> &UniqueGroupIdBuilder;
    fn join<'fmt>(&'fmt mut self) -> JoinBuilder<'fmt, 'buf, 'ast, ()>;
    fn join_with<'fmt, Joiner>(
        &'fmt mut self,
        joiner: Joiner,
    ) -> JoinBuilder<'fmt, 'buf, 'ast, Joiner>
    where
        Joiner: Format<'ast, JsFormatContext<'ast>>;
    fn join_nodes_with_soft_line<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line>;
    fn join_nodes_with_hardline<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Line>;
    fn join_nodes_with_space<'fmt>(&'fmt mut self) -> JoinNodesBuilder<'fmt, 'buf, 'ast, Space>;
    fn fill<'fmt>(&'fmt mut self) -> FillBuilder<'fmt, 'buf, 'ast>;
    fn speculate_will_break(
        &mut self,
        content: &(impl Format<'ast, JsFormatContext<'ast>> + GetSpan),
    ) -> bool;
}

impl<'buf, 'ast> JsFormatterExt<'buf, 'ast> for Formatter<'buf, 'ast, JsFormatContext<'ast>> {
    /// Returns the format options
    #[inline]
    fn options(&self) -> &JsFormatOptions {
        self.context().options()
    }

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

    fn join<'fmt>(&'fmt mut self) -> JoinBuilder<'fmt, 'buf, 'ast, ()> {
        JoinBuilder::new(self)
    }

    fn join_with<'fmt, Joiner>(
        &'fmt mut self,
        joiner: Joiner,
    ) -> JoinBuilder<'fmt, 'buf, 'ast, Joiner>
    where
        Joiner: Format<'ast, JsFormatContext<'ast>>,
    {
        JoinBuilder::with_separator(self, joiner)
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

    fn fill<'fmt>(&'fmt mut self) -> FillBuilder<'fmt, 'buf, 'ast> {
        FillBuilder::new(self)
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
