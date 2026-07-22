use std::{cell::RefCell, mem};

use oxc_allocator::{Allocator, GetAllocator};
use rustc_hash::FxHashMap;

use crate::{FormatElement, GroupId, UniqueGroupIdBuilder, format_element::Interned};

thread_local! {
    /// Cache of heap staging vectors, keeping their high-water capacity alive across format runs on the same thread.
    /// once a thread is warm, staging performs no heap allocation at all.
    /// A stack because format runs nest (embedded-language formatting creates a child [`FormatState`] on the same thread).
    static SCRATCH_CACHE: RefCell<Vec<Vec<FormatElement<'static>>>> =
        const { RefCell::new(Vec::new()) };
}

/// The heap staging vector backing [`crate::HeapVecBuffer`] (see there for why staging is heap-backed).
/// One vector per format run, shared by all nesting levels like a stack via watermarks:
/// each buffer records the length at creation, pushes its elements, and drains its own tail on completion.
/// Sound because IR staging is strictly LIFO (an inner buffer always completes before its enclosing one resumes).
///
/// Checked out of [`SCRATCH_CACHE`] on creation and returned on drop (panics included).
pub struct ScratchBuffer<'ast>(Vec<FormatElement<'ast>>);

impl<'ast> ScratchBuffer<'ast> {
    fn checkout() -> Self {
        let vec = SCRATCH_CACHE.with_borrow_mut(Vec::pop).unwrap_or_default();
        // SAFETY: cached vectors are always empty (checkin clears them);
        // an empty vector holds no values, so re-branding its element lifetime is sound.
        Self(unsafe {
            mem::transmute::<Vec<FormatElement<'static>>, Vec<FormatElement<'ast>>>(vec)
        })
    }
}

impl Drop for ScratchBuffer<'_> {
    fn drop(&mut self) {
        // Every `HeapVecBuffer` drains its own tail on completion;
        // a leftover means a buffer leaked elements past its watermark.
        // Checked here because every format run: root, fragment, or embedded — ends by dropping its `FormatState`.
        // (Skipped mid-unwind: buffers up the stack haven't truncated their tails yet.)
        debug_assert!(
            self.0.is_empty() || std::thread::panicking(),
            "scratch buffer not fully drained at the end of the format run"
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
