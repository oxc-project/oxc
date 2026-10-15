#![expect(clippy::mutable_key_type)]
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, ArenaVec};

use crate::{
    Arguments, FormatElement, FormatState, Formatter,
    format_element::{
        Interned, LineMode, PrintMode,
        tag::{Condition, Tag},
    },
};

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
    fn write_fmt(&mut self, arguments: Arguments<'_, 'ast, C>);

    /// Returns the formatting state relevant for this formatting session.
    fn state(&self) -> &FormatState<'ast, C>;

    /// Returns the mutable formatting state relevant for this formatting session.
    fn state_mut(&mut self) -> &mut FormatState<'ast, C>;

    /// Replaces the elements starting at `start` with `replacement`.
    ///
    /// Used by streaming IR transforms (currently `SortImportsTransform`)
    /// to splice a reordered chunk back into the buffer.
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

/// An owned heap staging vector for accumulators that outlive a single staging scope.
///
/// For element sequences that outlive a single exclusive borrow of the [`Formatter`]:
/// builders that accumulate across separate `write()` calls while other content is formatted in between
/// (multiple builders open at once, see the JSX child-list builders in `oxc_formatter`),
/// or staging that must release the formatter between being written and being consumed
/// (the assignment-like left hand side, whose layout is computed with the formatter in between).
/// Those can't stage in an arena vector of their own (every growth would strand the grown-out-of allocation,
/// where the element stack is reserved once by the root)
/// nor on the format run's element stack (suspended or interleaved use breaks its LIFO discipline);
/// a write goes past the element stack and moves here.
///
/// Content leaves the buffer only through the named consumption paths ([`crate::Formatter::intern_elements`],
/// [`ScratchBuffer::drain`], or [`ScratchBuffer::discard`]) all of which leave it empty,
/// satisfying the drop-time assertion below.
#[derive(Debug, Default)]
pub struct ScratchBuffer<'ast>(pub(crate) Vec<FormatElement<'ast>>);

impl<'ast> ScratchBuffer<'ast> {
    /// An empty scratch buffer.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Appends what `content` writes to this buffer, returning what `content` returns.
    ///
    /// The only write path into the vector,
    /// so accumulated content is always guarded by the drain-before-drop assertion below.
    pub fn write<'buf, C, R>(
        &mut self,
        f: &mut Formatter<'buf, 'ast, C>,
        content: impl FnOnce(&mut Formatter<'_, 'ast, C>) -> R,
    ) -> R {
        // Written past the elements on the stack, then moved here
        let state = f.state_mut();
        let start = state.elements().len();
        let result = content(&mut Formatter::nested(&mut *state));
        self.0.extend(state.elements_mut().drain(start..));
        result
    }

    /// Removes and returns the accumulated elements, leaving the buffer empty (re-emit consumption path).
    pub fn drain(&mut self) -> std::vec::Drain<'_, FormatElement<'ast>> {
        self.0.drain(..)
    }

    /// Inserts an element at `index`, shifting the rest.
    /// For post-hoc adjustment of already-accumulated content
    /// (e.g. slipping a separator into the last written entry); new content goes through [`ScratchBuffer::write`].
    pub fn insert(&mut self, index: usize, element: FormatElement<'ast>) {
        self.0.insert(index, element);
    }

    /// Abandons any accumulated content, releasing the allocation immediately
    /// (no caller writes again after a discard, so holding the capacity until drop would only pin peak memory).
    pub fn discard(&mut self) {
        self.0 = Vec::new();
    }
}

impl<'ast> Deref for ScratchBuffer<'ast> {
    type Target = [FormatElement<'ast>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for ScratchBuffer<'_> {
    fn drop(&mut self) {
        // Accumulators finish via `intern_elements`/`drain`/`discard`; a leftover means elements leaked.
        // (Skipped mid-unwind: the owner may unwind while content is still staged.)
        debug_assert!(
            self.0.is_empty() || std::thread::panicking(),
            "scratch buffer dropped before being fully drained"
        );
    }
}

/// State of the [`crate::Formatter::remove_soft_lines`] mode.
///
/// * Removes [`lines`](FormatElement::Line) with the mode [`Soft`](LineMode::Soft).
/// * Replaces [`lines`](FormatElement::Line) with the mode [`Soft`](LineMode::SoftOrSpace) with a [`Space`](FormatElement::Space)
pub struct SoftLines<'ast> {
    /// Caches the interned elements after the soft line breaks have been removed.
    ///
    /// The `key` is the [Interned] element as it has been passed to [Self::write] or the child of another
    /// [Interned] element. The `value` is the matching document of the key where all soft line breaks have been removed.
    ///
    /// It's fine to not snapshot the cache. The worst that can happen is that it holds on interned elements
    /// that are now unused. But there's little harm in that and the cache is cleaned when the mode ends.
    interned_cache: FxHashMap<Interned<'ast>, Interned<'ast>>,

    /// Store the conditional content stack to help determine if the current element is within expanded conditional content.
    conditional_content_stack: Vec<Condition>,

    /// Drops every element: the mode was entered inside an enclosing mode's expanded conditional content,
    /// all of which the enclosing mode drops.
    discard: bool,
}

impl<'ast> SoftLines<'ast> {
    /// Starts the mode, nested in `enclosing` if one is active.
    ///
    /// The nested mode's output passes the enclosing one unchanged (it holds no soft lines, conditional content, or best fitting),
    /// so only whether the enclosing one drops everything carries over.
    pub fn new(enclosing: Option<&SoftLines<'ast>>) -> Self {
        Self {
            interned_cache: FxHashMap::default(),
            conditional_content_stack: Vec::new(),
            discard: enclosing.is_some_and(|s| s.discard || s.is_in_expanded_conditional_content()),
        }
    }

    /// Marker for whether a `StartConditionalContent(mode: Expanded)` has been
    /// written but not yet closed.
    fn is_in_expanded_conditional_content(&self) -> bool {
        self.conditional_content_stack
            .iter()
            .last()
            .is_some_and(|condition| condition.mode == PrintMode::Expanded)
    }

    /// Writes `element` to the element stack of `state` with the soft line breaks removed.
    // Streaming twin of `push_cleaned_element`; see the note there before changing either.
    // Cold: kept out of `Formatter::write_element`, which is inlined into every builder.
    #[cold]
    #[inline(never)]
    pub fn write<C>(&mut self, element: FormatElement<'ast>, state: &mut FormatState<'ast, C>) {
        let allocator = state.allocator();
        self.write_cleaned(element, state.elements_mut(), allocator);
    }

    fn write_cleaned(
        &mut self,
        element: FormatElement<'ast>,
        out: &mut ArenaVec<'ast, FormatElement<'ast>>,
        allocator: &'ast Allocator,
    ) {
        match element {
            _ if self.discard => {}
            FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
                self.conditional_content_stack.push(condition);
            }
            FormatElement::Tag(Tag::EndConditionalContent) => {
                self.conditional_content_stack.pop();
            }
            // All content within an expanded conditional gets dropped.
            // If there's a matching flat variant, that will still get kept.
            _ if self.is_in_expanded_conditional_content() => {}

            FormatElement::Line(LineMode::Soft) => {}
            FormatElement::Line(LineMode::SoftOrSpace) => {
                out.push(FormatElement::Space);
            }
            FormatElement::Interned(interned) => {
                let cleaned = clean_interned(
                    interned,
                    &mut self.interned_cache,
                    &mut self.conditional_content_stack,
                    allocator,
                );
                out.push(FormatElement::Interned(cleaned));
            }
            // Since this mode aims to simulate infinite print width, we don't need to retain the best fitting.
            // Just extract the flattest variant and then handle elements within it.
            FormatElement::BestFitting(best_fitting) => {
                for element in best_fitting.most_flat() {
                    self.write_cleaned(element.clone(), out, allocator);
                }
            }
            element => out.push(element),
        }
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
                let cleaned = ArenaVec::from_iter_in(interned[..index].iter().cloned(), &allocator);
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
                    let mut cleaned = ArenaVec::with_capacity_in(interned.len(), &allocator);
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
                for element in rest {
                    push_cleaned_element(
                        element,
                        &mut cleaned,
                        interned_cache,
                        condition_content_stack,
                        allocator,
                    );
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

/// NOTE: Materializing twin of [`SoftLines::write`]: applies the same soft-line-removal rules,
/// but pushes into a new `Interned` body (borrowed input, clones) instead of streaming owned elements to the element stack.
/// Kept separate so the streaming path avoids clones.
/// The two `match`es must stay rule-for-rule in sync.
fn push_cleaned_element<'ast>(
    element: &FormatElement<'ast>,
    cleaned: &mut ArenaVec<'ast, FormatElement<'ast>>,
    interned_cache: &mut FxHashMap<Interned<'ast>, Interned<'ast>>,
    condition_content_stack: &mut Vec<Condition>,
    allocator: &'ast Allocator,
) {
    match element {
        FormatElement::Tag(Tag::StartConditionalContent(condition)) => {
            condition_content_stack.push(condition.clone());
        }
        FormatElement::Tag(Tag::EndConditionalContent) => {
            condition_content_stack.pop();
        }
        // All content within an expanded conditional gets dropped.
        // If there's a matching flat variant, that will still get kept.
        _ if condition_content_stack
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
            for element in best_fitting.most_flat() {
                push_cleaned_element(
                    element,
                    cleaned,
                    interned_cache,
                    condition_content_stack,
                    allocator,
                );
            }
        }
        element => cleaned.push(element.clone()),
    }
}

pub trait BufferExtensions<'ast, C>: Buffer<'ast, C> + Sized {
    /// Starts a recording that gives you access to all elements that have been written between the start
    /// and end of the recording.
    #[must_use]
    fn start_recording(&mut self) -> Recording<'_, 'ast, Self, C> {
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
pub struct Recording<'buf, 'ast, B, C> {
    start: usize,
    buffer: &'buf mut B,
    _marker: PhantomData<&'ast C>,
}

impl<'ast, 'buf, B, C> Recording<'buf, 'ast, B, C>
where
    B: Buffer<'ast, C>,
{
    fn new(buffer: &'buf mut B) -> Self {
        Self { start: buffer.elements().len(), buffer, _marker: PhantomData }
    }

    #[inline(always)]
    pub fn write_fmt(&mut self, arguments: Arguments<'_, 'ast, C>) {
        self.buffer.write_fmt(arguments);
    }

    #[inline(always)]
    pub fn write_element(&mut self, element: FormatElement<'ast>) {
        self.buffer.write_element(element);
    }

    pub fn stop(self) -> Recorded<'buf, 'ast> {
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
