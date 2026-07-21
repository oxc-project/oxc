use std::{cell::RefCell, mem};

use oxc_allocator::{Allocator, GetAllocator};
use rustc_hash::FxHashMap;

use crate::{
    FormatElement, GroupId, UniqueGroupIdBuilder, buffer::AccumulatorBuffer,
    format_element::Interned,
};

thread_local! {
    /// Cache of heap staging vectors, keeping their high-water capacity alive across format runs on the same thread.
    /// Once a thread is warm, staging performs no heap allocation at all.
    /// Holds several because checkouts overlap (a run's shared scratch and spare slot,
    /// accumulators, and embedded-language child [`FormatState`]s on the same thread);
    /// [`ScratchBuffer::checkout`] picks by capacity, not order.
    static SCRATCH_CACHE: RefCell<Vec<Vec<FormatElement<'static>>>> =
        const { RefCell::new(Vec::new()) };
}

/// A heap staging vector checked out of the thread-local `SCRATCH_CACHE` on creation and returned on drop (panics included),
/// so its high-water capacity is reused across checkouts.
///
/// Two users:
/// - [`FormatState`] holds one per format run, shared by all [`crate::HeapVecBuffer`]s like a stack via watermarks:
///   each buffer records the length at creation, pushes its elements, and drains its own tail on completion.
///   Sound because IR staging is strictly LIFO (an inner buffer always completes before its enclosing one resumes)
/// - Accumulators that outlive a single staging scope (interleaved or suspended use, written through [`crate::AccumulatorBuffer`]) check out their own.
///   The cache itself is an unordered pool with no LIFO requirement
#[derive(Debug)]
pub struct ScratchBuffer<'ast>(Vec<FormatElement<'ast>>);

impl<'ast> ScratchBuffer<'ast> {
    pub fn checkout() -> Self {
        // Take the roomiest vector, not the most recently returned one:
        // capacities differ wildly between users (a run's shared scratch grows to the
        // deepest staging stack, an assignment accumulator to one left hand side),
        // and a blind LIFO pop depends on the order buffers happen to be returned in —
        // get it wrong and the run scratch re-grows from nothing every run while the
        // grown-out vector idles in a small role. The cache stays a handful of entries,
        // so the scan is cheap.
        let vec = SCRATCH_CACHE
            .with_borrow_mut(|cache| {
                let index = cache
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, vec)| vec.capacity())
                    .map(|(index, _)| index);
                index.map(|index| cache.swap_remove(index))
            })
            .unwrap_or_default();
        // SAFETY: cached vectors are always empty (checkin clears them);
        // an empty vector holds no values, so re-branding its element lifetime is sound.
        Self(unsafe {
            mem::transmute::<Vec<FormatElement<'static>>, Vec<FormatElement<'ast>>>(vec)
        })
    }

    /// An unpooled, empty scratch buffer, for accumulators that may never be written (e.g. a builder born disabled):
    /// costs no thread-local access on creation, nor on drop while still empty.
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Adapts this scratch vector into a [`Buffer`](crate::Buffer) writing into it.
    ///
    /// This is the only way to construct an [`AccumulatorBuffer`],
    /// so an accumulator always writes into a pooled vector guarded by the drain-before-drop assertion below.
    pub fn writer<'buf, C>(
        &'buf mut self,
        state: &'buf mut FormatState<'ast, C>,
    ) -> AccumulatorBuffer<'buf, 'ast, C> {
        AccumulatorBuffer::new(state, &mut self.0)
    }
}

impl<'ast> std::ops::Deref for ScratchBuffer<'ast> {
    type Target = Vec<FormatElement<'ast>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ScratchBuffer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for ScratchBuffer<'_> {
    fn drop(&mut self) {
        // Every user drains the vector on completion (`HeapVecBuffer`s their own tail,
        // accumulators via `Formatter::intern_elements`); a leftover means elements leaked.
        // (Skipped mid-unwind: buffers up the stack haven't truncated their tails yet.)
        debug_assert!(
            self.0.is_empty() || std::thread::panicking(),
            "scratch buffer returned to the cache before being fully drained"
        );
        let mut vec = mem::take(&mut self.0);
        // A buffer that never grew has nothing worth caching
        if vec.capacity() == 0 {
            return;
        }
        vec.clear();
        // SAFETY: just cleared; see `checkout`
        let vec =
            unsafe { mem::transmute::<Vec<FormatElement<'_>>, Vec<FormatElement<'static>>>(vec) };
        SCRATCH_CACHE.with_borrow_mut(|cache| cache.push(vec));
    }
}

/// This structure stores the state that is relevant for the formatting of the whole document.
///
/// This structure is different from [crate::Formatter] in that the formatting infrastructure
/// creates a new [crate::Formatter] for every [`crate::write!`] call, whereas this structure stays alive
/// for the whole process of formatting a root with [crate::format!].
pub struct FormatState<'ast, C> {
    context: C,
    allocator: &'ast Allocator,
    group_id_builder: UniqueGroupIdBuilder,
    // For the document IR printing process
    /// The interned elements that have been printed to this point
    printed_interned_elements: FxHashMap<Interned<'ast>, usize>,
    /// Heap staging vector for [`crate::HeapVecBuffer`]; see [`ScratchBuffer`].
    scratch: ScratchBuffer<'ast>,
    /// Spare staging vector slot for per-node accumulators on hot paths;
    /// see [`FormatState::take_spare_scratch`].
    spare_scratch: ScratchBuffer<'ast>,
}

impl<C: std::fmt::Debug> std::fmt::Debug for FormatState<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState").field("context", &self.context).finish()
    }
}

impl<'ast, C> FormatState<'ast, C> {
    /// Creates a new state with the given language specific context
    pub fn new(context: C, allocator: &'ast Allocator) -> Self {
        Self {
            context,
            allocator,
            group_id_builder: UniqueGroupIdBuilder::default(),
            printed_interned_elements: FxHashMap::default(),
            scratch: ScratchBuffer::checkout(),
            spare_scratch: ScratchBuffer::empty(),
        }
    }

    /// Takes the spare staging vector for a short-lived accumulator
    /// (staging that must release the state between being written and being consumed,
    /// see [`crate::AccumulatorBuffer`]).
    ///
    /// This is a per-run slot, not a checkout from the thread-local cache:
    /// on hot paths (e.g. every assignment-like left hand side) the thread-local
    /// round-trip of [`ScratchBuffer::checkout`] is measurable, a field swap is not.
    /// Return it with [`FormatState::return_spare_scratch`] once drained.
    ///
    /// When the slot has no capacity to offer — first take of the run, or a nested
    /// take while the slot is already out — it falls back to a pooled checkout,
    /// so repeated takes always reuse grown capacity from somewhere.
    /// On a nested take, whichever buffer is returned last wins the slot
    /// (the loser drops back to the thread-local cache).
    pub fn take_spare_scratch(&mut self) -> ScratchBuffer<'ast> {
        let taken = mem::replace(&mut self.spare_scratch, ScratchBuffer::empty());
        if taken.capacity() == 0 { ScratchBuffer::checkout() } else { taken }
    }

    /// Puts the spare staging vector back into its slot; see [`FormatState::take_spare_scratch`].
    pub fn return_spare_scratch(&mut self, scratch: ScratchBuffer<'ast>) {
        self.spare_scratch = scratch;
    }

    /// The heap staging vector shared by all [`crate::HeapVecBuffer`]s of this format run.
    pub(crate) fn scratch(&self) -> &[FormatElement<'ast>] {
        &self.scratch.0
    }

    /// Mutable access to the heap staging vector; see [`FormatState::scratch`].
    pub(crate) fn scratch_mut(&mut self) -> &mut Vec<FormatElement<'ast>> {
        &mut self.scratch.0
    }

    /// Returns the allocator used for arena-allocating format elements.
    pub fn allocator(&self) -> &'ast Allocator {
        self.allocator
    }

    pub fn into_context(self) -> C {
        self.context
    }

    /// Returns the context specifying how to format the current CST
    pub fn context(&self) -> &C {
        &self.context
    }

    /// Returns a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut C {
        &mut self.context
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.group_id_builder.group_id(debug_name)
    }

    /// Returns a reference to the unique group id builder.
    pub fn group_id_builder(&self) -> &UniqueGroupIdBuilder {
        &self.group_id_builder
    }

    #[expect(clippy::mutable_key_type)]
    pub fn printed_interned_elements(&mut self) -> &mut FxHashMap<Interned<'ast>, usize> {
        &mut self.printed_interned_elements
    }
}

impl<'ast, C> GetAllocator<'ast> for FormatState<'ast, C> {
    #[inline]
    fn allocator(&self) -> &'ast Allocator {
        self.allocator
    }
}
