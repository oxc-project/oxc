use std::{cell::RefCell, collections::VecDeque, mem};

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, GetAllocator};

use crate::{
    FormatElement, GroupId, UniqueGroupIdBuilder, buffer::AccumulatorBuffer,
    format_element::Interned,
};

/// Maximum total element-storage capacity retained by the scratch cache on each thread.
///
/// This bounds only idle capacity between format runs;
/// a run can still grow beyond it, and the oversized vector is then released at run end instead of cached.
/// The cache retains only what a run actually grew, so a generous ceiling is near-free,
/// while a tight one would make every re-format of a large file re-grow its scratch from nothing.
///
/// 8 MB keeps every measured non-bundled fixture cached (`checker.ts` included);
/// only the bundled `antd.js` overflows and re-grows per run
/// (its elevated sys-alloc numbers in `allocs_formatter.yaml` are this constant at work),
/// a deliberate trade-off: repeatedly re-formatting bundles is not worth pinning tens of MB per thread.
const MAX_CACHED_SCRATCH_BYTES: usize = 8 * 1024 * 1024;

/// Maximum number of scratch vectors retained on each thread.
///
/// Sized for the concurrent users: nested format runs (embedded languages) plus overlapping accumulators
/// (nested JSX child lists hold up to two each, so 64 keeps roughly 30 levels of JSX nesting warm).
/// Low-stakes in both directions: retained memory is already bounded by the byte limit above,
/// overflow just makes excess accumulators allocate fresh small vectors, and the count mainly keeps the `take_largest` scan short.
const MAX_CACHED_SCRATCH_BUFFERS: usize = 64;

thread_local! {
    /// Cache of heap staging vectors, reusing capacity across format runs on the same thread.
    /// Retained memory and buffer count are bounded by the constants above.
    /// Multiple vectors are kept because format runs can nest
    /// (embedded-language formatting creates a child [`FormatState`] on the same thread) and accumulators can overlap.
    ///
    /// Accumulator buffers use LIFO order.
    /// A [`FormatState`] explicitly takes the largest cached buffer,
    /// so rejecting an oversized run scratch cannot make the next run start with a small accumulator buffer.
    /// When a limit is reached, [`ScratchCache::insert`] evicts the oldest buffers first;
    /// an individually oversized vector is released instead of cached.
    static SCRATCH_CACHE: RefCell<ScratchCache> = const { RefCell::new(ScratchCache::new()) };
}

struct ScratchCache {
    buffers: VecDeque<Vec<FormatElement<'static>>>,
    retained_bytes: usize,
}

impl ScratchCache {
    const fn new() -> Self {
        Self { buffers: VecDeque::new(), retained_bytes: 0 }
    }

    fn take(&mut self) -> Vec<FormatElement<'static>> {
        let Some(buffer) = self.buffers.pop_back() else { return Vec::new() };
        self.retained_bytes -= Self::capacity_in_bytes(buffer.capacity());
        buffer
    }

    fn take_largest(&mut self) -> Vec<FormatElement<'static>> {
        let Some((index, _)) =
            self.buffers.iter().enumerate().max_by_key(|(_, buffer)| buffer.capacity())
        else {
            return Vec::new();
        };
        let buffer = self.buffers.remove(index).unwrap();
        self.retained_bytes -= Self::capacity_in_bytes(buffer.capacity());
        buffer
    }

    fn insert(&mut self, buffer: Vec<FormatElement<'static>>) {
        let buffer_bytes = Self::capacity_in_bytes(buffer.capacity());
        if buffer_bytes == 0 || buffer_bytes > MAX_CACHED_SCRATCH_BYTES {
            return;
        }

        // Evict the least recently returned buffers until both limits have room
        while self.buffers.len() >= MAX_CACHED_SCRATCH_BUFFERS
            || self.retained_bytes > MAX_CACHED_SCRATCH_BYTES - buffer_bytes
        {
            let Some(evicted) = self.buffers.pop_front() else { break };
            self.retained_bytes -= Self::capacity_in_bytes(evicted.capacity());
        }

        self.retained_bytes += buffer_bytes;
        self.buffers.push_back(buffer);
    }

    const fn capacity_in_bytes(capacity: usize) -> usize {
        capacity.saturating_mul(size_of::<FormatElement<'static>>())
    }
}

/// A heap staging vector backed by the thread-local `SCRATCH_CACHE`.
///
/// Pooled capacity is fetched on first use and returned on drop (panics included),
/// so its high-water capacity is reused across instances.
///
/// Two users:
/// - [`FormatState`] holds one per format run, shared by all [`crate::HeapVecBuffer`]s like a stack via watermarks:
///   each buffer records the length at creation, pushes its elements, and drains its own tail on completion.
///   Sound because IR staging is strictly LIFO (an inner buffer always completes before its enclosing one resumes)
/// - Accumulators that outlive a single staging scope (interleaved or suspended use, written through [`crate::AccumulatorBuffer`]) each own one.
///   Accumulators fetch cached vectors in LIFO order; [`FormatState`] takes the largest vector
///
/// Content leaves the buffer only through the named consumption paths
/// ([`crate::Formatter::intern_elements`], [`ScratchBuffer::drain`], or [`ScratchBuffer::discard`])
/// all of which leave it empty, satisfying the drop-time assertion below.
#[derive(Debug)]
pub struct ScratchBuffer<'ast>(Vec<FormatElement<'ast>>);

impl<'ast> ScratchBuffer<'ast> {
    /// An empty scratch buffer. Costs nothing until written to:
    /// pooled capacity is fetched lazily by [`ScratchBuffer::writer`],
    /// so an accumulator that is never written (e.g. a builder born disabled) never touches the cache.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    fn from_cached(vec: Vec<FormatElement<'static>>) -> Self {
        // SAFETY: cached vectors are always empty (checkin clears them);
        // an empty vector holds no values, so re-branding its element lifetime is sound.
        Self(unsafe {
            mem::transmute::<Vec<FormatElement<'static>>, Vec<FormatElement<'ast>>>(vec)
        })
    }

    fn checkout() -> Self {
        Self::from_cached(SCRATCH_CACHE.with_borrow_mut(ScratchCache::take))
    }

    fn checkout_largest() -> Self {
        Self::from_cached(SCRATCH_CACHE.with_borrow_mut(ScratchCache::take_largest))
    }

    /// Adapts this scratch vector into a [`Buffer`](crate::Buffer) writing into it,
    /// fetching pooled capacity on the way if the vector has none yet.
    ///
    /// This is the only way to construct an [`AccumulatorBuffer`] and the only write path into the vector,
    /// so accumulated content is always guarded by the drain-before-drop assertion below.
    pub fn writer<'buf, C>(
        &'buf mut self,
        state: &'buf mut FormatState<'ast, C>,
    ) -> AccumulatorBuffer<'buf, 'ast, C> {
        if self.0.capacity() == 0 {
            *self = Self::checkout();
        }
        AccumulatorBuffer::new(state, &mut self.0)
    }

    /// Removes and returns the accumulated elements, leaving the buffer empty (re-emit consumption path).
    pub fn drain(&mut self) -> std::vec::Drain<'_, FormatElement<'ast>> {
        self.0.drain(..)
    }

    /// Inserts an element at `index`, shifting the rest.
    /// For post-hoc adjustment of already-accumulated content
    /// (e.g. slipping a separator into the last written entry); new content goes through [`ScratchBuffer::writer`].
    pub fn insert(&mut self, index: usize, element: FormatElement<'ast>) {
        self.0.insert(index, element);
    }

    /// Abandons any accumulated content and returns the pooled capacity to the cache immediately,
    /// so other accumulators (e.g. nested ones still running) can reuse it.
    pub fn discard(&mut self) {
        self.0.clear();
        // The taken-out buffer's drop checks its capacity back in
        drop(mem::take(self));
    }
}

impl<'ast> std::ops::Deref for ScratchBuffer<'ast> {
    type Target = [FormatElement<'ast>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ScratchBuffer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScratchBuffer<'_> {
    fn drop(&mut self) {
        // Every user empties the vector on completion (`HeapVecBuffer`s drain their own tail,
        // accumulators finish via `intern_elements`/`drain`/`discard`); a leftover means elements leaked.
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
        SCRATCH_CACHE.with_borrow_mut(|cache| cache.insert(vec));
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
            scratch: ScratchBuffer::checkout_largest(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer_with_capacity(capacity: usize) -> Vec<FormatElement<'static>> {
        Vec::with_capacity(capacity)
    }

    fn assert_retained_bytes_consistent(cache: &ScratchCache) {
        assert_eq!(
            cache.retained_bytes,
            cache
                .buffers
                .iter()
                .map(|buffer| ScratchCache::capacity_in_bytes(buffer.capacity()))
                .sum::<usize>()
        );
    }

    #[test]
    fn scratch_cache_rejects_an_oversized_buffer() {
        let mut cache = ScratchCache::new();
        let oversized_capacity = MAX_CACHED_SCRATCH_BYTES / size_of::<FormatElement<'static>>() + 1;

        cache.insert(buffer_with_capacity(oversized_capacity));

        assert!(cache.buffers.is_empty());
        assert_eq!(cache.retained_bytes, 0);
    }

    #[test]
    fn scratch_cache_bounds_total_retained_bytes() {
        let mut cache = ScratchCache::new();
        let half_budget_capacity =
            MAX_CACHED_SCRATCH_BYTES / size_of::<FormatElement<'static>>() / 2;

        for _ in 0..3 {
            cache.insert(buffer_with_capacity(half_budget_capacity));
        }

        assert!(cache.retained_bytes <= MAX_CACHED_SCRATCH_BYTES);
        assert_retained_bytes_consistent(&cache);
    }

    #[test]
    fn scratch_cache_bounds_buffer_count() {
        let mut cache = ScratchCache::new();

        for _ in 0..=MAX_CACHED_SCRATCH_BUFFERS {
            cache.insert(buffer_with_capacity(1));
        }

        assert_eq!(cache.buffers.len(), MAX_CACHED_SCRATCH_BUFFERS);
    }

    #[test]
    fn scratch_cache_updates_retained_bytes_on_take() {
        let mut cache = ScratchCache::new();
        cache.insert(buffer_with_capacity(16));
        let retained_bytes = cache.retained_bytes;

        let buffer = cache.take();

        assert_eq!(ScratchCache::capacity_in_bytes(buffer.capacity()), retained_bytes);
        assert_eq!(cache.retained_bytes, 0);
    }

    #[test]
    fn scratch_cache_takes_the_largest_buffer_for_format_state() {
        let mut cache = ScratchCache::new();
        cache.insert(buffer_with_capacity(8));
        cache.insert(buffer_with_capacity(32));
        cache.insert(buffer_with_capacity(16));

        let buffer = cache.take_largest();

        assert_eq!(buffer.capacity(), 32);
        assert_retained_bytes_consistent(&cache);
    }

    #[test]
    fn scratch_cache_evicts_the_oldest_buffer() {
        let mut cache = ScratchCache::new();
        cache.insert(buffer_with_capacity(2));
        for _ in 1..MAX_CACHED_SCRATCH_BUFFERS {
            cache.insert(buffer_with_capacity(1));
        }

        cache.insert(buffer_with_capacity(3));

        assert_eq!(cache.buffers.len(), MAX_CACHED_SCRATCH_BUFFERS);
        assert_eq!(cache.buffers.back().unwrap().capacity(), 3);
        assert!(!cache.buffers.iter().any(|buffer| buffer.capacity() == 2));
    }
}
