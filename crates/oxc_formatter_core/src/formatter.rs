#![allow(clippy::module_inception)]

use oxc_allocator::{Allocator, Vec as ArenaVec};

use crate::{
    Argument, Arguments, Buffer, FormatContext, FormatElement, FormatState, VecBuffer,
    builders::{FillBuilder, JoinBuilder},
    format::{Format, write},
    format_element::Interned,
};

/// Handles the formatting of a CST and stores the context how the CST should be formatted (user preferences).
///
/// The formatter is passed to the [Format] implementation of every node in the CST so that they
/// can use it to format their children.
pub struct Formatter<'buf, 'ast, C> {
    pub(crate) buffer: &'buf mut dyn Buffer<'ast, C>,
}

impl<'buf, 'ast, C> Formatter<'buf, 'ast, C> {
    /// Creates a new context that uses the given formatter context
    pub fn new(buffer: &'buf mut (dyn Buffer<'ast, C> + 'buf)) -> Self {
        Self { buffer }
    }

    pub fn allocator(&self) -> &'ast Allocator {
        self.state().allocator()
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

    /// Returns the format options.
    #[inline]
    pub fn options(&self) -> &<C as FormatContext>::Options
    where
        C: FormatContext,
    {
        self.context().options()
    }

    /// Formats `content` into an interned element without writing it to the formatter's buffer.
    pub fn intern(&mut self, content: &dyn Format<'ast, C>) -> Option<FormatElement<'ast>> {
        let mut buffer = VecBuffer::new(self.state_mut());
        write(&mut buffer, Arguments::new(&[Argument::new(&content)]));
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
            1 => elements.pop(),
            _ => Some(FormatElement::Interned(Interned::new(elements))),
        }
    }

    /// Creates a [`JoinBuilder`] that joins entries together without a separator.
    pub fn join<'fmt>(&'fmt mut self) -> JoinBuilder<'fmt, 'buf, 'ast, (), C> {
        JoinBuilder::new(self)
    }

    /// Creates a [`JoinBuilder`] that joins entries together using `joiner` as a separator.
    pub fn join_with<'fmt, Joiner>(
        &'fmt mut self,
        joiner: Joiner,
    ) -> JoinBuilder<'fmt, 'buf, 'ast, Joiner, C>
    where
        Joiner: Format<'ast, C>,
    {
        JoinBuilder::with_separator(self, joiner)
    }

    /// Creates a [`FillBuilder`] that fills as many elements as possible on a single line.
    pub fn fill<'fmt>(&'fmt mut self) -> FillBuilder<'fmt, 'buf, 'ast, C> {
        FillBuilder::new(self)
    }
}

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
