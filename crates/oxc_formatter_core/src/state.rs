use std::{cell::RefCell, mem};

use oxc_allocator::{Allocator, GetAllocator};
use rustc_hash::FxHashMap;

use crate::{
    FormatElement, GroupId, UniqueGroupIdBuilder, buffer::AccumulatorBuffer,
    format_element::Interned,
};

thread_local! {
    /// Cache of heap staging vectors, keeping their high-water capacity alive across format runs on the same thread.
    /// once a thread is warm, staging performs no heap allocation at all.
    /// A stack because format runs nest (embedded-language formatting creates a child [`FormatState`] on the same thread).
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
        let vec = SCRATCH_CACHE.with_borrow_mut(Vec::pop).unwrap_or_default();
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
    /// Whether texts borrowed from the context's source may be stored as offset-based
    /// [`FormatElement::SourceText`]. Off by default so that forgetting the opt-in only
    /// costs memory, never correctness: embedded-language IR is spliced into a host
    /// document and printed against the host's source, where offsets into the embedded
    /// source would silently resolve to the wrong text. See
    /// [`FormatState::enable_source_text`].
    allow_source_text: bool,
    /// Per-document cache for static tokens longer than
    /// [`FormatElement::INLINE_TOKEN_MAX`]: each unique token is arena-allocated once
    /// instead of once per write. Keyed by the `&'static str` data pointer.
    long_token_cache: FxHashMap<*const u8, crate::format_element::ArenaText<'ast>>,
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
            allow_source_text: false,
            long_token_cache: FxHashMap::default(),
        }
    }

    /// Opts this run into offset-based [`FormatElement::SourceText`] storage.
    ///
    /// Only entry formatters whose IR is printed against the same source held by the
    /// context may call this (i.e. `format()`-style entry points). Embedded runs
    /// (`format_to_ir`-style entry points, whose IR is spliced into a host document)
    /// must NOT: their offsets would resolve against the host's source. Forgetting
    /// this call is safe — texts fall back to arena copies.
    pub fn enable_source_text(&mut self) {
        self.allow_source_text = true;
    }

    /// A [`FormatElement`] for a static token longer than
    /// [`FormatElement::INLINE_TOKEN_MAX`], arena-allocated once per document.
    pub(crate) fn long_token(&mut self, text: &'static str) -> FormatElement<'ast> {
        let allocator = self.allocator;
        let arena_text = *self.long_token_cache.entry(text.as_ptr()).or_insert_with(|| {
            #[expect(clippy::cast_possible_truncation)]
            crate::format_element::ArenaText::alloc_in(
                text,
                crate::format_element::TextWidth::single(text.len() as u32),
                allocator,
            )
        });
        FormatElement::ArenaText(arena_text)
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

    /// Byte offset of `text` inside the context's source, if `text` borrows from it and
    /// offset-based storage is enabled (see [`FormatState::enable_source_text`]).
    pub fn source_offset_of(&self, text: &str) -> Option<u32>
    where
        C: crate::FormatContext,
    {
        if !self.allow_source_text {
            return None;
        }
        let source = self.context.source_code();
        let start = source.as_ptr().addr();
        let ptr = text.as_ptr().addr();
        if ptr >= start && ptr + text.len() <= start + source.len() {
            u32::try_from(ptr - start).ok()
        } else {
            None
        }
    }

    /// Returns a reference to the unique group id builder.
    pub fn group_id_builder(&self) -> &UniqueGroupIdBuilder {
        &self.group_id_builder
    }

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
