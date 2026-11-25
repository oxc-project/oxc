#![allow(clippy::module_inception)]

use std::borrow::Cow;

use oxc_allocator::{Allocator, ArenaVec, GetAllocator};

use crate::{
    Arguments, Format, FormatContext, FormatElement, FormatSession, FormatState, ScratchBuffer,
    builders::{FillBuilder, JoinBuilder},
    format_element::Interned,
    write::buffer::SoftLines,
};

/// Interns an exactly-sized element sequence, encoding the interning policy:
/// the 0/1-element cases return without ever allocating an `ArenaVec`.
/// [`intern_tail`] is its twin for the element stack.
pub fn intern_exact<'ast>(
    allocator: &'ast Allocator,
    mut elements: impl ExactSizeIterator<Item = FormatElement<'ast>>,
) -> Option<FormatElement<'ast>> {
    match elements.len() {
        0 => None,
        // Doesn't get cheaper than calling clone, use the element directly
        1 => elements.next(),
        // The exact size hint makes `from_iter_in` allocate exactly-sized
        _ => Some(FormatElement::Interned(Interned::new(ArenaVec::from_iter_in(
            elements, &allocator,
        )))),
    }
}

/// Moves `elements[start..]` into an exactly-sized arena vector in one copy.
fn take_tail<'ast>(
    elements: &mut ArenaVec<'ast, FormatElement<'ast>>,
    start: usize,
    allocator: &'ast Allocator,
) -> ArenaVec<'ast, FormatElement<'ast>> {
    let tail = &elements[start..];
    let len = tail.len();
    let mut taken = ArenaVec::with_capacity_in(len, &allocator);
    // SAFETY: `taken` has room for `len` elements, and the tail moves into it bit for bit
    // while `elements` forgets it, so each element keeps exactly one owner
    unsafe {
        std::ptr::copy_nonoverlapping(tail.as_ptr(), taken.as_mut_ptr(), len);
        taken.set_len(len);
        elements.set_len(start);
    }
    taken
}

/// [`intern_exact`] for `elements[start..]`, which it removes.
fn intern_tail<'ast>(
    elements: &mut ArenaVec<'ast, FormatElement<'ast>>,
    start: usize,
    allocator: &'ast Allocator,
) -> Option<FormatElement<'ast>> {
    match elements.len() - start {
        0 => None,
        1 => elements.pop(),
        _ => Some(FormatElement::Interned(Interned::new(take_tail(elements, start, allocator)))),
    }
}

/// Lifts a `Cow<'ast, str>` to `&'ast str`, allocating in the arena only for the owned case.
/// Borrowed Cows already point into arena-resident source, so they pass through unchanged.
pub fn arena_cow_str<'ast, C>(cow: &Cow<'ast, str>, f: &Formatter<'_, 'ast, C>) -> &'ast str {
    match cow {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => f.allocator().alloc_str(s),
    }
}

/// Handles the formatting of a CST and stores the context how the CST should be formatted (user preferences).
///
/// The formatter is passed to the [Format] implementation of every node in the CST so that they
/// can use it to format their children.
///
/// All formatters of a format run write into one element stack owned by the [`FormatState`].
/// A formatter's elements are the stack's tail past `start`.
/// A nested formatter (interning, best fitting variants) appends past its parent's elements
/// and drains them before the parent writes again, which the exclusive borrow of the state guarantees.
/// The root formatter starts at 0, and its elements are taken as they are, in the arena.
pub struct Formatter<'buf, 'ast, C> {
    pub(crate) state: &'buf mut FormatState<'ast, C>,
    pub(crate) start: usize,
    /// Set inside [`Formatter::remove_soft_lines`].
    pub(crate) soft_lines: Option<&'buf mut SoftLines<'ast>>,
}

impl<'buf, 'ast, C> Formatter<'buf, 'ast, C> {
    /// Creates the root formatter of the format run of `state`, taken with [`Formatter::into_vec`].
    pub fn new(state: &'buf mut FormatState<'ast, C>) -> Self {
        debug_assert!(state.elements().is_empty(), "a root formatter on a state in use");
        Self { state, start: 0, soft_lines: None }
    }

    /// Like [`Formatter::new`], reserving room for `capacity` elements.
    /// The room stays reserved in the arena, and growing past it strands the outgrown vector.
    pub fn with_capacity(capacity: usize, state: &'buf mut FormatState<'ast, C>) -> Self {
        state.elements_mut().reserve(capacity);
        Self::new(state)
    }

    /// A formatter writing past the elements on the stack, which its caller takes off.
    pub(crate) fn nested(state: &'buf mut FormatState<'ast, C>) -> Self {
        let start = state.elements().len();
        Self { state, start, soft_lines: None }
    }

    /// Consumes the root formatter and returns the written elements as they are.
    pub fn into_vec(self) -> ArenaVec<'ast, FormatElement<'ast>> {
        let allocator = self.state.allocator();
        std::mem::replace(self.state.elements_mut(), ArenaVec::new_in(&allocator))
    }

    /// Writes what `content` writes past the elements on the stack,
    /// and returns where it starts and what `content` returns.
    pub(crate) fn stage<R>(
        &mut self,
        content: impl FnOnce(&mut Formatter<'_, 'ast, C>) -> R,
    ) -> (usize, R) {
        let mut staged = Formatter::nested(&mut *self.state);
        let result = content(&mut staged);
        (staged.start, result)
    }

    /// Formats `content` into an exactly-sized arena vector without writing it to this formatter.
    /// `content` writes without the [`Formatter::remove_soft_lines`] mode;
    /// it applies when the result is written to this formatter.
    pub fn format_to_vec(
        &mut self,
        content: impl FnOnce(&mut Formatter<'_, 'ast, C>),
    ) -> ArenaVec<'ast, FormatElement<'ast>> {
        let (start, ()) = self.stage(content);
        let allocator = self.state.allocator();
        take_tail(self.state.elements_mut(), start, allocator)
    }

    /// Writes what `content` writes with the soft line breaks removed, simulating an infinite print width.
    ///
    /// * Removes [`lines`](FormatElement::Line) with the mode [`Soft`](crate::LineMode::Soft).
    /// * Replaces [`lines`](FormatElement::Line) with the mode [`SoftOrSpace`](crate::LineMode::SoftOrSpace) with a [`Space`](FormatElement::Space).
    /// * Keeps only the most flat variant of best fitting elements, and only the flat conditional content.
    pub fn remove_soft_lines(&mut self, content: impl FnOnce(&mut Formatter<'_, 'ast, C>)) {
        let mut soft_lines = SoftLines::new(self.soft_lines.as_deref());
        content(&mut Formatter {
            state: &mut *self.state,
            start: self.start,
            soft_lines: Some(&mut soft_lines),
        });
    }

    pub fn allocator(&self) -> &'ast Allocator {
        self.state.allocator()
    }

    /// The [`FormatSession`] this format run executes under.
    pub fn session(&self) -> &FormatSession<'ast> {
        self.state.session()
    }

    /// Returns the Context specifying how to format the current CST
    #[inline]
    pub fn context(&self) -> &C {
        self.state.context()
    }

    /// Returns a mutable reference to the context.
    #[inline]
    pub fn context_mut(&mut self) -> &mut C {
        self.state.context_mut()
    }

    /// Returns the format options.
    #[inline]
    pub fn options(&self) -> &<C as FormatContext>::Options
    where
        C: FormatContext,
    {
        self.context().options()
    }

    /// Formats `content` into an interned element without writing it to this formatter,
    /// like [`Formatter::format_to_vec`].
    /// The arena only ever holds the final, exactly-sized interned slice.
    pub fn intern(&mut self, content: &dyn Format<'ast, C>) -> Option<FormatElement<'ast>> {
        let (start, ()) = self.stage(|f| content.fmt(f));
        let allocator = self.state.allocator();
        intern_tail(self.state.elements_mut(), start, allocator)
    }

    /// Interns a builder's heap accumulator, moving it into the arena exactly-sized.
    /// The source is emptied.
    pub fn intern_elements(
        &self,
        elements: &mut ScratchBuffer<'ast>,
    ) -> Option<FormatElement<'ast>> {
        intern_exact(self.allocator(), elements.0.drain(..))
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

    /// Writes an element.
    #[inline]
    pub fn write_element(&mut self, element: FormatElement<'ast>) {
        match &mut self.soft_lines {
            None => self.state.elements_mut().push_fast(element),
            Some(soft_lines) => soft_lines.write(element, self.state),
        }
    }

    /// Writes a sequence of elements.
    pub fn write_elements(&mut self, elements: impl IntoIterator<Item = FormatElement<'ast>>) {
        for element in elements {
            self.write_element(element);
        }
    }

    /// The elements this formatter has written.
    /// To look at what something writes, note `elements().len()` before writing it.
    pub fn elements(&self) -> &[FormatElement<'ast>] {
        &self.state.elements()[self.start..]
    }

    /// Glue for the [`write!`](crate::write!) macro.
    #[inline(always)]
    pub fn write_fmt(&mut self, arguments: Arguments<'_, 'ast, C>) {
        for argument in arguments.items() {
            argument.format(self);
        }
    }

    /// The state of the format run.
    pub fn state(&self) -> &FormatState<'ast, C> {
        self.state
    }

    /// The mutable state of the format run.
    pub(crate) fn state_mut(&mut self) -> &mut FormatState<'ast, C> {
        self.state
    }

    /// Replaces the elements from `start` (of [`Formatter::elements`]) on with `replacement`.
    ///
    /// Used by streaming IR transforms (currently `SortImportsTransform`)
    /// to splice a reordered chunk back.
    ///
    /// # Panics
    ///
    /// If `start` is past the elements.
    pub fn replace_end(&mut self, start: usize, replacement: &[FormatElement<'ast>]) {
        let start = self.start + start;
        let elements = self.state.elements_mut();
        // `truncate` silently ignores an out-of-bounds start; fail loudly instead
        assert!(start <= elements.len(), "replace_end start out of bounds");
        elements.truncate(start);
        elements.extend_from_slice(replacement);
    }
}

impl<'ast, C> GetAllocator<'ast> for Formatter<'_, 'ast, C> {
    #[inline]
    fn allocator(&self) -> &'ast Allocator {
        self.state.allocator()
    }
}
