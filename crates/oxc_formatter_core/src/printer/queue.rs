use std::{fmt::Debug, iter::FusedIterator, marker::PhantomData};

use crate::{FormatElement, Tag, TagKind};

use super::{
    PrintResult, invalid_end_tag, invalid_start_tag,
    stack::{Stack, StackedStack},
};

/// Queue of [FormatElement]s.
///
/// The queue is a cursor (`current`) over the top-most slice plus a stack of suspended slices.
/// Keeping the cursor in a dedicated field makes [`Queue::pop`] and [`Queue::top`] a single
/// length check in the common case; the stack is only consulted at slice boundaries.
///
/// Invariant: `current` is never empty while the stack holds further slices, and slices stored
/// on the stack are never empty. Consequently, the queue is exhausted iff `current` is empty.
pub(super) trait Queue<'a> {
    type Stack: Stack<&'a [FormatElement<'a>]>;

    /// The stack of suspended slices.
    /// Never push empty slices onto it (it would break the invariant above);
    /// use [`Queue::extend_back`] to queue new content instead.
    fn stack_mut(&mut self) -> &mut Self::Stack;

    /// Returns the not-yet-consumed rest of the top-most slice.
    fn current(&self) -> &'a [FormatElement<'a>];

    fn set_current(&mut self, slice: &'a [FormatElement<'a>]);

    /// Pops the element at the end of the queue.
    #[inline]
    fn pop(&mut self) -> Option<&'a FormatElement<'a>> {
        match self.current().split_first() {
            Some((element, rest)) => {
                if rest.is_empty() {
                    // Slice exhausted, eagerly refill from the stack to uphold the invariant
                    // that `current` is only empty once the whole queue is exhausted.
                    let next = match self.stack_mut().pop() {
                        Some(next) => {
                            debug_assert!(
                                !next.is_empty(),
                                "slices stored on the stack must never be empty"
                            );
                            next
                        }
                        None => &[],
                    };
                    self.set_current(next);
                } else {
                    self.set_current(rest);
                }

                Some(element)
            }
            None => None,
        }
    }

    /// Returns the next element, not traversing into [FormatElement::Interned].
    #[inline]
    fn top_with_interned(&self) -> Option<&'a FormatElement<'a>> {
        self.current().first()
    }

    /// Returns the next element, recursively resolving the first element of [FormatElement::Interned].
    fn top(&self) -> Option<&'a FormatElement<'a>> {
        let mut top = self.top_with_interned();

        while let Some(FormatElement::Interned(interned)) = top {
            top = interned.first();
        }

        top
    }

    /// Queues a single element to process before the other elements in this queue.
    fn push(&mut self, element: &'a FormatElement) {
        self.extend_back(std::slice::from_ref(element));
    }

    /// Queues a slice of elements to process before the other elements in this queue.
    #[inline]
    fn extend_back(&mut self, elements: &'a [FormatElement]) {
        match elements {
            [] => {
                // Don't push empty slices
            }
            slice => {
                let current = self.current();
                if !current.is_empty() {
                    self.stack_mut().push(current);
                }
                self.set_current(slice);
            }
        }
    }

    /// Removes the top slice and returns its not-yet-consumed remainder
    /// (the whole slice if no element has been popped from it yet).
    fn pop_slice(&mut self) -> Option<&'a [FormatElement<'a>]> {
        let current = self.current();
        let next = self.stack_mut().pop().unwrap_or(&[]);
        self.set_current(next);

        if current.is_empty() { None } else { Some(current) }
    }

    /// Skips all content until it finds the corresponding end tag with the given kind.
    fn skip_content(&mut self, kind: TagKind)
    where
        Self: Sized,
    {
        let iter = self.iter_content(kind);

        for _ in iter {
            // consume whole iterator until end
        }
    }

    /// Iterates over all elements until it finds the matching end tag of the specified kind.
    fn iter_content<'q>(&'q mut self, kind: TagKind) -> QueueContentIterator<'a, 'q, Self>
    where
        Self: Sized,
    {
        QueueContentIterator::new(self, kind)
    }
}

/// Queue with the elements to print.
#[derive(Debug, Default, Clone)]
pub(super) struct PrintQueue<'a> {
    current: &'a [FormatElement<'a>],
    slices: Vec<&'a [FormatElement<'a>]>,
}

impl<'a> PrintQueue<'a> {
    pub(super) fn new(slice: &'a [FormatElement<'a>]) -> Self {
        Self { current: slice, slices: Vec::default() }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.current.is_empty()
    }
}

impl<'a> Queue<'a> for PrintQueue<'a> {
    type Stack = Vec<&'a [FormatElement<'a>]>;

    fn stack_mut(&mut self) -> &mut Self::Stack {
        &mut self.slices
    }

    #[inline]
    fn current(&self) -> &'a [FormatElement<'a>] {
        self.current
    }

    #[inline]
    fn set_current(&mut self, slice: &'a [FormatElement<'a>]) {
        self.current = slice;
    }
}

/// Queue for measuring if an element fits on the line.
///
/// The queue is a view on top of the [PrintQueue] because no elements should be removed
/// from the [PrintQueue] while measuring.
#[must_use]
#[derive(Debug)]
pub(super) struct FitsQueue<'a, 'print> {
    current: &'a [FormatElement<'a>],
    stack: StackedStack<'print, &'a [FormatElement<'a>]>,
}

impl<'a, 'print> FitsQueue<'a, 'print> {
    pub(super) fn new(
        print_queue: &'print PrintQueue<'a>,
        saved: Vec<&'a [FormatElement]>,
    ) -> Self {
        let stack = StackedStack::with_vec(&print_queue.slices, saved);

        Self { current: print_queue.current, stack }
    }

    pub(super) fn finish(self) -> Vec<&'a [FormatElement<'a>]> {
        self.stack.into_vec()
    }
}

impl<'a, 'print> Queue<'a> for FitsQueue<'a, 'print> {
    type Stack = StackedStack<'print, &'a [FormatElement<'a>]>;

    fn stack_mut(&mut self) -> &mut Self::Stack {
        &mut self.stack
    }

    #[inline]
    fn current(&self) -> &'a [FormatElement<'a>] {
        self.current
    }

    #[inline]
    fn set_current(&mut self, slice: &'a [FormatElement<'a>]) {
        self.current = slice;
    }
}

pub(super) struct QueueContentIterator<'a, 'q, Q: Queue<'a>> {
    queue: &'q mut Q,
    kind: TagKind,
    depth: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a, 'q, Q> QueueContentIterator<'a, 'q, Q>
where
    Q: Queue<'a>,
{
    fn new(queue: &'q mut Q, kind: TagKind) -> Self {
        Self { queue, kind, depth: 1, lifetime: PhantomData }
    }
}

impl<'a, Q> Iterator for QueueContentIterator<'a, '_, Q>
where
    Q: Queue<'a>,
{
    type Item = &'a FormatElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.depth == 0 {
            None
        } else {
            let mut top = self.queue.pop();

            while let Some(FormatElement::Interned(interned)) = top {
                self.queue.extend_back(interned);
                top = self.queue.pop();
            }

            match top.expect("Missing end signal.") {
                element @ FormatElement::Tag(tag) if tag.kind() == self.kind => {
                    if tag.is_start() {
                        self.depth += 1;
                    } else {
                        self.depth -= 1;

                        if self.depth == 0 {
                            return None;
                        }
                    }

                    Some(element)
                }
                element => Some(element),
            }
        }
    }
}

impl<'a, Q> FusedIterator for QueueContentIterator<'a, '_, Q> where Q: Queue<'a> {}

/// A predicate determining when to end measuring if some content fits on the line.
///
/// Called for every [`element`](FormatElement) in the [FitsQueue] when measuring if a content
/// fits on the line. The measuring of the content ends after the first element [`element`](FormatElement) for which this
/// predicate returns `true` (similar to a take while iterator except that it takes while the predicate returns `false`).
pub(super) trait FitsEndPredicate {
    fn is_end(&mut self, element: &FormatElement) -> PrintResult<bool>;
}

/// Filter that includes all elements until it reaches the end of the document.
pub(super) struct AllPredicate;

impl FitsEndPredicate for AllPredicate {
    fn is_end(&mut self, _element: &FormatElement) -> PrintResult<bool> {
        Ok(false)
    }
}

/// Filter that takes all elements between two matching [Tag::StartEntry] and [Tag::EndEntry] tags.
#[derive(Debug)]
pub(super) enum SingleEntryPredicate {
    Entry { depth: usize },
    Done,
}

impl SingleEntryPredicate {
    pub(super) const fn is_done(&self) -> bool {
        matches!(self, SingleEntryPredicate::Done)
    }
}

impl Default for SingleEntryPredicate {
    fn default() -> Self {
        SingleEntryPredicate::Entry { depth: 0 }
    }
}

impl FitsEndPredicate for SingleEntryPredicate {
    fn is_end(&mut self, element: &FormatElement) -> PrintResult<bool> {
        let result = match self {
            SingleEntryPredicate::Done => true,
            SingleEntryPredicate::Entry { depth } => match element {
                FormatElement::Tag(Tag::StartEntry) => {
                    *depth += 1;

                    false
                }
                FormatElement::Tag(Tag::EndEntry) => {
                    if *depth == 0 {
                        return invalid_end_tag(TagKind::Entry, None);
                    }

                    *depth -= 1;

                    let is_end = *depth == 0;

                    if is_end {
                        *self = SingleEntryPredicate::Done;
                    }

                    is_end
                }
                FormatElement::Interned(_) => false,
                element if *depth == 0 => {
                    return invalid_start_tag(TagKind::Entry, Some(element));
                }
                _ => false,
            },
        };

        Ok(result)
    }
}
