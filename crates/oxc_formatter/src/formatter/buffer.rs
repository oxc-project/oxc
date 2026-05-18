#![expect(clippy::mutable_key_type)]
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, TakeIn, Vec as ArenaVec};

use super::{
    Arguments, Format, FormatElement, FormatState,
    format_element::Interned,
    prelude::{LineMode, PrintMode, Tag, tag::Condition},
};
use crate::write;

/// A trait for writing or formatting into `FormatElement`-accepting buffers or streams.
pub trait Buffer<'ast, C> {
    /// Writes a [crate::FormatElement] into this buffer, returning whether the write succeeded.
    fn write_element(&mut self, element: FormatElement<'ast>);

    /// Returns a slice containing all elements written into this buffer.
    ///
    /// Prefer using [BufferExtensions::start_recording] over accessing [Buffer::elements] directly.
    #[doc(hidden)]
    fn elements(&self) -> &[FormatElement<'ast>];

    /// Glue for usage of the [`write!`] macro with implementors of this trait.
    ///
    /// This method should generally not be invoked manually, but rather through the [`write!`] macro itself.
    fn write_fmt(mut self: &mut Self, arguments: Arguments<'_, 'ast, C>) {
        super::write(&mut self, arguments);
    }

    /// Returns the formatting state relevant for this formatting session.
    fn state(&self) -> &FormatState<'ast, C>;

    /// Returns the mutable formatting state relevant for this formatting session.
    fn state_mut(&mut self) -> &mut FormatState<'ast, C>;

    /// Replaces the elements starting at `start` with `replacement`.
    ///
    /// Used by streaming IR transforms (currently `SortImportsTransform`) to splice a reordered
    /// chunk back into the buffer. Only `VecBuffer` supports this; the wrapper buffers
    /// (`PreambleBuffer`, `Inspect`, `RemoveSoftLinesBuffer`) are only ever active inside
    /// inner-expression contexts, never on the call stack while a streaming chunk is being
    /// flushed, so they implement this as `unreachable!()`.
    fn replace_end(&mut self, start: usize, replacement: &[FormatElement<'ast>]);
}

/// Implements the `[Buffer]` trait for all mutable references of objects implementing [Buffer].
impl<'ast, C, W: Buffer<'ast, C> + ?Sized> Buffer<'ast, C> for &mut W {
    fn write_element(&mut self, element: FormatElement<'ast>) {
        (**self).write_element(element);
    }

    fn elements(&self) -> &[FormatElement<'ast>] {
        (**self).elements()
    }

    fn write_fmt(&mut self, args: Arguments<'_, 'ast, C>) {
        (**self).write_fmt(args);
    }

    fn state(&self) -> &FormatState<'ast, C> {
        (**self).state()
    }

    fn state_mut(&mut self) -> &mut FormatState<'ast, C> {
        (**self).state_mut()
    }

    fn replace_end(&mut self, start: usize, replacement: &[FormatElement<'ast>]) {
        (**self).replace_end(start, replacement);
    }
}

/// Vector backed [`Buffer`] implementation.
///
/// The buffer writes all elements into the internal elements buffer.
#[derive(Debug)]
pub struct VecBuffer<'buf, 'ast, C> {
    state: &'buf mut FormatState<'ast, C>,
    elements: ArenaVec<'ast, FormatElement<'ast>>,
}

impl<'buf, 'ast, C> VecBuffer<'buf, 'ast, C> {
    pub fn new(state: &'buf mut FormatState<'ast, C>) -> Self {
        Self::new_with_vec(state, ArenaVec::new_in(state.allocator()))
    }

    pub fn new_with_vec(
        state: &'buf mut FormatState<'ast, C>,
        elements: ArenaVec<'ast, FormatElement<'ast>>,
    ) -> Self {
        Self { state, elements }
    }

    /// Creates a buffer with the specified capacity
    pub fn with_capacity(capacity: usize, state: &'buf mut FormatState<'ast, C>) -> Self {
        let elements = ArenaVec::with_capacity_in(capacity, state.allocator());
        Self { state, elements }
    }

    /// Consumes the buffer and returns the written [`FormatElement]`s as a vector.
    pub fn into_vec(self) -> ArenaVec<'ast, FormatElement<'ast>> {
        self.elements
    }

    /// Takes the elements without consuming self
    pub fn take_vec(&mut self) -> ArenaVec<'ast, FormatElement<'ast>> {
        self.elements.take_in(self.state.allocator())
    }
}

impl<'ast, C> Deref for VecBuffer<'_, 'ast, C> {
    type Target = [FormatElement<'ast>];

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<C> DerefMut for VecBuffer<'_, '_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl<'ast, C> Buffer<'ast, C> for VecBuffer<'_, 'ast, C> {
    fn write_element(&mut self, element: FormatElement<'ast>) {
        self.elements.push(element);
    }

    fn elements(&self) -> &[FormatElement<'ast>] {
        self
    }

    fn state(&self) -> &FormatState<'ast, C> {
        self.state
    }

    fn state_mut(&mut self) -> &mut FormatState<'ast, C> {
        self.state
    }

    fn replace_end(&mut self, start: usize, replacement: &[FormatElement<'ast>]) {
        self.elements.splice(start.., replacement.iter().cloned());
    }
}

/// This struct wraps an existing buffer and emits a preamble text when the first text is written.
///
/// This can be useful if you, for example, want to write some content if what gets written next isn't empty.
///
/// The pre-amble does not get written if no content is written to the buffer.
pub struct PreambleBuffer<'a, 'buf, C, Preamble> {
    /// The wrapped buffer
    inner: &'buf mut dyn Buffer<'a, C>,

    /// The pre-amble to write once the first content gets written to this buffer.
    preamble: Preamble,

    /// Whether some content (including the pre-amble) has been written at this point.
    empty: bool,
}

impl<'ast, 'buf, C, Preamble> PreambleBuffer<'ast, 'buf, C, Preamble> {
    #[expect(unused)]
    pub fn new(inner: &'buf mut dyn Buffer<'ast, C>, preamble: Preamble) -> Self {
        Self { inner, preamble, empty: true }
    }

    /// Returns `true` if the preamble has been written, `false` otherwise.
    #[expect(unused)]
    pub fn did_write_preamble(&self) -> bool {
        !self.empty
    }
}

impl<'ast, C, Preamble> Buffer<'ast, C> for PreambleBuffer<'ast, '_, C, Preamble>
where
    Preamble: Format<'ast, C>,
{
    fn write_element(&mut self, element: FormatElement<'ast>) {
        if self.empty {
            write!(self.inner, [&self.preamble]);
            self.empty = false;
        }

        self.inner.write_element(element);
    }

    fn elements(&self) -> &[FormatElement<'ast>] {
        self.inner.elements()
    }

    fn state(&self) -> &FormatState<'ast, C> {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<'ast, C> {
        self.inner.state_mut()
    }

    fn replace_end(&mut self, _start: usize, _replacement: &[FormatElement<'ast>]) {
        unreachable!()
    }
}

/// Buffer that allows you inspecting elements as they get written to the formatter.
pub struct Inspect<'ast, 'inner, C, Inspector> {
    inner: &'inner mut dyn Buffer<'ast, C>,
    inspector: Inspector,
}

impl<'ast, 'inner, C, Inspector> Inspect<'ast, 'inner, C, Inspector> {
    fn new(inner: &'inner mut dyn Buffer<'ast, C>, inspector: Inspector) -> Self {
        Self { inner, inspector }
    }
}

impl<'a, C, Inspector> Buffer<'a, C> for Inspect<'a, '_, C, Inspector>
where
    Inspector: FnMut(&FormatElement),
{
    fn write_element(&mut self, element: FormatElement<'a>) {
        (self.inspector)(&element);
        self.inner.write_element(element);
    }

    fn elements(&self) -> &[FormatElement<'a>] {
        self.inner.elements()
    }

    fn state(&self) -> &FormatState<'a, C> {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<'a, C> {
        self.inner.state_mut()
    }

    fn replace_end(&mut self, _start: usize, _replacement: &[FormatElement<'a>]) {
        unreachable!()
    }
}

/// A Buffer that removes any soft line breaks.
///
/// * Removes [`lines`](FormatElement::Line) with the mode [`Soft`](LineMode::Soft).
/// * Replaces [`lines`](FormatElement::Line) with the mode [`Soft`](LineMode::SoftOrSpace) with a [`Space`](FormatElement::Space)
pub struct RemoveSoftLinesBuffer<'buf, 'ast, C> {
    inner: &'buf mut dyn Buffer<'ast, C>,

    /// Caches the interned elements after the soft line breaks have been removed.
    ///
    /// The `key` is the [Interned] element as it has been passed to [Self::write_element] or the child of another
    /// [Interned] element. The `value` is the matching document of the key where all soft line breaks have been removed.
    ///
    /// It's fine to not snapshot the cache. The worst that can happen is that it holds on interned elements
    /// that are now unused. But there's little harm in that and the cache is cleaned when dropping the buffer.
    interned_cache: FxHashMap<Interned<'ast>, Interned<'ast>>,

    /// Store the conditional content stack to help determine if the current element is within expanded conditional content.
    conditional_content_stack: Vec<Condition>,
}

impl<'buf, 'ast, C> RemoveSoftLinesBuffer<'buf, 'ast, C> {
    /// Creates a new buffer that removes the soft line breaks before writing them into `buffer`.
    pub fn new(inner: &'buf mut dyn Buffer<'ast, C>) -> Self {
        Self { inner, interned_cache: FxHashMap::default(), conditional_content_stack: Vec::new() }
    }

    /// Removes the soft line breaks from an interned element.
    fn clean_interned(&mut self, interned: Interned<'ast>) -> Interned<'ast> {
        clean_interned(
            interned,
            &mut self.interned_cache,
            &mut self.conditional_content_stack,
            self.inner.state().allocator(),
        )
    }

    /// Marker for whether a `StartConditionalContent(mode: Expanded)` has been
    /// written but not yet closed.
    fn is_in_expanded_conditional_content(&self) -> bool {
        self.conditional_content_stack
            .iter()
            .last()
            .is_some_and(|condition| condition.mode == PrintMode::Expanded)
    }
}

// Extracted to function to avoid monomorphization
fn clean_interned<'ast>(
    interned: Interned<'ast>,
    interned_cache: &mut FxHashMap<Interned<'ast>, Interned<'ast>>,
    condition_content_stack: &mut Vec<Condition>,
    allocator: &'ast Allocator,
) -> Interned<'ast> {
    if let Some(cleaned) = interned_cache.get(&interned) {
        cleaned.clone()
    } else {
        // Find the first soft line break element, interned element, or conditional expanded
        // content that must be changed.
        let result = interned.iter().enumerate().find_map(|(index, element)| match element {
            FormatElement::Line(LineMode::Soft | LineMode::SoftOrSpace)
            | FormatElement::Tag(Tag::StartConditionalContent(_) | Tag::EndConditionalContent)
            | FormatElement::BestFitting(_) => {
                let cleaned = ArenaVec::from_iter_in(interned[..index].iter().cloned(), allocator);
                Some((cleaned, &interned[index..]))
            }
            FormatElement::Interned(inner) => {
                let cleaned_inner = clean_interned(
                    inner.clone(),
                    interned_cache,
                    condition_content_stack,
                    allocator,
                );

                if &cleaned_inner == inner {
                    None
                } else {
                    let mut cleaned = ArenaVec::with_capacity_in(interned.len(), allocator);
                    cleaned.extend(interned[..index].iter().cloned());
                    cleaned.push(FormatElement::Interned(cleaned_inner));
                    Some((cleaned, &interned[index + 1..]))
                }
            }
            _ => None,
        });

        let result = match result {
            // Copy the whole interned buffer so that becomes possible to change the necessary elements.
            Some((mut cleaned, rest)) => {
                let mut element_stack = rest.iter().rev().collect::<Vec<_>>();
                while let Some(element) = element_stack.pop() {
                    match element {
                        FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
                            condition_content_stack.push(condition.clone());
                        }
                        FormatElement::Tag(Tag::EndConditionalContent) => {
                            condition_content_stack.pop();
                        }
                        // All content within an expanded conditional gets dropped. If there's a
                        // matching flat variant, that will still get kept.
                        _ if condition_content_stack
                            .iter()
                            .last()
                            .is_some_and(|condition| condition.mode == PrintMode::Expanded) => {}

                        FormatElement::Line(LineMode::Soft) => {}
                        FormatElement::Line(LineMode::SoftOrSpace) => {
                            cleaned.push(FormatElement::Space);
                        }

                        FormatElement::Interned(interned) => {
                            cleaned.push(FormatElement::Interned(clean_interned(
                                interned.clone(),
                                interned_cache,
                                condition_content_stack,
                                allocator,
                            )));
                        }
                        // Since this buffer aims to simulate infinite print width, we don't need to retain the best fitting.
                        // Just extract the flattest variant and then handle elements within it.
                        FormatElement::BestFitting(best_fitting) => {
                            let most_flat = best_fitting.most_flat();
                            most_flat.iter().rev().for_each(|element| element_stack.push(element));
                        }
                        element => cleaned.push(element.clone()),
                    }
                }

                Interned::new(cleaned)
            }
            // No change necessary, return existing interned element
            None => interned.clone(),
        };

        interned_cache.insert(interned, result.clone());
        result
    }
}

impl<'ast, C> Buffer<'ast, C> for RemoveSoftLinesBuffer<'_, 'ast, C> {
    fn write_element(&mut self, element: FormatElement<'ast>) {
        let mut element_stack = Vec::from_iter([element]);
        while let Some(element) = element_stack.pop() {
            match element {
                FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
                    self.conditional_content_stack.push(condition.clone());
                }
                FormatElement::Tag(Tag::EndConditionalContent) => {
                    self.conditional_content_stack.pop();
                }
                // All content within an expanded conditional gets dropped. If there's a
                // matching flat variant, that will still get kept.
                _ if self.is_in_expanded_conditional_content() => {}

                FormatElement::Line(LineMode::Soft) => {}
                FormatElement::Line(LineMode::SoftOrSpace) => {
                    self.inner.write_element(FormatElement::Space);
                }
                FormatElement::Interned(interned) => {
                    let cleaned = self.clean_interned(interned);
                    self.inner.write_element(FormatElement::Interned(cleaned));
                }
                // Since this buffer aims to simulate infinite print width, we don't need to retain the best fitting.
                // Just extract the flattest variant and then handle elements within it.
                FormatElement::BestFitting(best_fitting) => {
                    let most_flat = best_fitting.most_flat();
                    element_stack.extend(most_flat.iter().rev().cloned());
                }
                element => self.inner.write_element(element),
            }
        }
    }

    fn elements(&self) -> &[FormatElement<'ast>] {
        self.inner.elements()
    }

    fn state(&self) -> &FormatState<'ast, C> {
        self.inner.state()
    }

    fn state_mut(&mut self) -> &mut FormatState<'ast, C> {
        self.inner.state_mut()
    }

    fn replace_end(&mut self, _start: usize, _replacement: &[FormatElement<'ast>]) {
        unreachable!()
    }
}

pub trait BufferExtensions<'ast, C>: Buffer<'ast, C> + Sized {
    /// Returns a new buffer that calls the passed inspector for every element that gets written to the output
    #[must_use]
    #[expect(unused)]
    fn inspect<'inner, F>(&'inner mut self, inspector: F) -> Inspect<'ast, 'inner, C, F>
    where
        F: FnMut(&FormatElement),
    {
        Inspect::new(self, inspector)
    }

    /// Starts a recording that gives you access to all elements that have been written between the start
    /// and end of the recording
    #[must_use]
    fn start_recording(&mut self) -> Recording<'_, Self> {
        Recording::new(self)
    }

    /// Writes a sequence of elements into this buffer.
    fn write_elements<I>(&mut self, elements: I)
    where
        I: IntoIterator<Item = FormatElement<'ast>>,
    {
        for element in elements {
            self.write_element(element);
        }
    }
}

impl<'ast, C, T> BufferExtensions<'ast, C> for T where T: Buffer<'ast, C> {}

#[derive(Debug)]
pub struct Recording<'buf, Buffer> {
    start: usize,
    buffer: &'buf mut Buffer,
}

impl<'buf, B> Recording<'buf, B> {
    fn new<'ast, C>(buffer: &'buf mut B) -> Self
    where
        B: Buffer<'ast, C>,
    {
        Self { start: buffer.elements().len(), buffer }
    }

    #[inline(always)]
    pub fn write_fmt<'ast, C>(&mut self, arguments: Arguments<'_, 'ast, C>)
    where
        B: Buffer<'ast, C>,
    {
        self.buffer.write_fmt(arguments);
    }

    #[inline(always)]
    #[expect(unused)]
    pub fn write_element<'ast, C>(&mut self, element: FormatElement<'ast>)
    where
        B: Buffer<'ast, C>,
    {
        self.buffer.write_element(element);
    }

    pub fn stop<'ast, C>(self) -> Recorded<'buf, 'ast>
    where
        B: Buffer<'ast, C>,
    {
        let buffer: &'buf B = self.buffer;
        let elements = buffer.elements();

        let recorded = if self.start > elements.len() {
            // May happen if buffer was rewound.
            &[]
        } else {
            &elements[self.start..]
        };

        Recorded(recorded)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Recorded<'buf, 'ast>(&'buf [FormatElement<'ast>]);

impl<'ast> Deref for Recorded<'_, 'ast> {
    type Target = [FormatElement<'ast>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
