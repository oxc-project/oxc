#![allow(clippy::module_inception)]

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_span::GetSpan;

use crate::options::JsFormatOptions;

use super::{
    Arguments, Buffer, Comments, FormatState, GroupId, JsFormatContext, SourceText, VecBuffer,
    builders::{FillBuilder, JoinBuilder, JoinNodesBuilder, Line},
    core_traits::FormatContext,
    prelude::*,
};

/// Handles the formatting of a CST and stores the context how the CST should be formatted (user preferences).
///
/// The formatter is passed to the [Format] implementation of every node in the CST so that they
/// can use it to format their children.
pub struct Formatter<'buf, 'ast, C> {
    pub(super) buffer: &'buf mut dyn Buffer<'ast, C>,
}

// --- Generic impl (no bounds on C) ---

impl<'buf, 'ast, C> Formatter<'buf, 'ast, C> {
    /// Creates a new context that uses the given formatter context
    pub fn new(buffer: &'buf mut (dyn Buffer<'ast, C> + 'buf)) -> Self {
        Self { buffer }
    }

    pub fn allocator(&self) -> &'ast Allocator {
        self.state().allocator()
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    #[inline]
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.state().group_id(debug_name)
    }

    /// Returns a reference to the unique group id builder for this document.
    #[inline]
    pub fn group_id_builder(&self) -> &super::UniqueGroupIdBuilder {
        self.state().group_id_builder()
    }

    /// Returns the Context specifying how to format the current CST
    #[inline]
    pub fn context(&self) -> &C {
        self.state().context()
    }

    /// Returns a mutable reference to the context.
    #[inline]
    pub fn context_mut(&mut self) -> &mut C {
        self.state_mut().context_mut()
    }

    /// Joins multiple [Format] together without any separator
    pub fn join<'fmt>(&'fmt mut self) -> JoinBuilder<'fmt, 'buf, 'ast, C, ()> {
        JoinBuilder::new(self)
    }

    /// Joins the objects by placing the specified separator between every two items.
    pub fn join_with<'fmt, Joiner>(
        &'fmt mut self,
        joiner: Joiner,
    ) -> JoinBuilder<'fmt, 'buf, 'ast, C, Joiner>
    where
        Joiner: Format<'ast, C>,
    {
        JoinBuilder::with_separator(self, joiner)
    }

    /// Concatenates a list of [crate::Format] objects with spaces and line breaks to fit
    /// them on as few lines as possible. Each element introduces a conceptual group. The printer
    /// first tries to print the item in flat mode but then prints it in expanded mode if it doesn't fit.
    pub fn fill<'fmt>(&'fmt mut self) -> FillBuilder<'fmt, 'buf, 'ast, C> {
        FillBuilder::new(self)
    }

    /// Formats `content` into an interned element without writing it to the formatter's buffer.
    pub fn intern(&mut self, content: &dyn Format<'ast, C>) -> Option<FormatElement<'ast>> {
        let mut buffer = VecBuffer::new(self.state_mut());
        crate::write!(&mut buffer, [content]);
        let elements = buffer.into_vec();

        self.intern_vec(elements)
    }

    #[expect(clippy::unused_self)] // Keep `self` the same as the original source
    pub fn intern_vec(
        &self,
        mut elements: ArenaVec<'ast, FormatElement<'ast>>,
    ) -> Option<FormatElement<'ast>> {
        match elements.len() {
            0 => None,
            // Doesn't get cheaper than calling clone, use the element directly
            // SAFETY: Safe because of the `len == 1` check in the match arm.
            1 => Some(elements.pop().unwrap()),
            _ => Some(FormatElement::Interned(Interned::new(elements))),
        }
    }
}

// --- FormatContext-bounded impl ---

impl<C: FormatContext> Formatter<'_, '_, C> {
    /// Returns the format options
    #[inline]
    pub fn options(&self) -> &C::Options {
        self.context().options()
    }
}

// --- JS-specific impl ---

impl<'buf, 'ast> Formatter<'buf, 'ast, JsFormatContext<'ast>> {
    /// Returns the JS format options
    #[inline]
    pub fn js_options(&self) -> &JsFormatOptions {
        self.context().options()
    }

    /// Returns the source text wrapper.
    #[inline]
    pub fn source_text(&self) -> SourceText<'ast> {
        self.context().source_text()
    }

    /// Returns the comments from the context.
    #[inline]
    pub fn comments(&self) -> &Comments<'_> {
        self.context().comments()
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by a space, soft
    /// line break or empty line depending on the input file.
    ///
    /// This function inspects the input source and separates consecutive elements with either
    /// a [crate::builders::soft_line_break_or_space] or [crate::builders::empty_line] depending on how many line breaks were
    /// separating the elements in the original file.
    pub fn join_nodes_with_soft_line<'fmt>(
        &'fmt mut self,
    ) -> JoinNodesBuilder<'fmt, 'buf, 'ast, JsFormatContext<'ast>, Line> {
        JoinNodesBuilder::new(soft_line_break_or_space(), self)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by one or more
    /// line breaks depending on the input file.
    ///
    /// This function inspects the input source and separates consecutive elements with either
    /// a [crate::builders::hard_line_break] or [crate::builders::empty_line] depending on how many line breaks were separating the
    /// elements in the original file.
    pub fn join_nodes_with_hardline<'fmt>(
        &'fmt mut self,
    ) -> JoinNodesBuilder<'fmt, 'buf, 'ast, JsFormatContext<'ast>, Line> {
        JoinNodesBuilder::new(hard_line_break(), self)
    }

    /// Specialized version of [crate::Formatter::join_with] for joining SyntaxNodes separated by a simple space.
    ///
    /// This function *disregards* the input source and always separates consecutive elements with a plain
    /// [crate::builders::space], forcing a flat layout regardless of any line breaks or spaces were separating
    /// the elements in the original file.
    ///
    /// This function should likely only be used in a `best_fitting!` context, where one variant attempts to
    /// force a list of nodes onto a single line without any possible breaks, then falls back to a broken
    /// out variant if the content does not fit.
    pub fn join_nodes_with_space<'fmt>(
        &'fmt mut self,
    ) -> JoinNodesBuilder<'fmt, 'buf, 'ast, JsFormatContext<'ast>, Space> {
        JoinNodesBuilder::new(space(), self)
    }

    /// Speculatively formats `content` and returns whether the result would break across lines.
    ///
    /// This snapshots and restores the comment state so that the speculative formatting
    /// doesn't permanently advance the comment cursor. Comments before the content's span
    /// are skipped so they don't get incorrectly included as leading comments.
    pub fn speculate_will_break(
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

// --- Buffer implementation ---

impl<'ast, C> Buffer<'ast, C> for Formatter<'_, 'ast, C> {
    #[inline(always)]
    fn write_element(&mut self, element: FormatElement<'ast>) {
        self.buffer.write_element(element);
    }

    fn elements(&self) -> &[FormatElement<'ast>] {
        self.buffer.elements()
    }

    #[inline(always)]
    fn write_fmt(&mut self, arguments: Arguments<'_, 'ast, C>) {
        for argument in arguments.items() {
            argument.format(self);
        }
    }

    fn state(&self) -> &FormatState<'ast, C> {
        self.buffer.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<'ast, C> {
        self.buffer.state_mut()
    }

    fn replace_end(&mut self, start: usize, replacement: &[FormatElement<'ast>]) {
        self.buffer.replace_end(start, replacement);
    }
}
