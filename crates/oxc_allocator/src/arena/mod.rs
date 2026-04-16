//! Arena allocator.
//!
//! The files in this directory were originally derived from `bumpalo` at commit
//! a47f6d6b7b5fee9c99a285f0de80257a0a982ef3 (2 commits after 3.20.2 release).
//! Changes have been made since.

#![expect(clippy::inline_always, clippy::undocumented_unsafe_blocks)]

use std::{
    alloc::Layout,
    cell::Cell,
    ptr::{self, NonNull},
};

#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
use crate::tracking::AllocationStats;

mod alloc;
mod alloc_impl;
mod bumpalo_alloc;
mod chunks;
mod create;
mod drop;
mod utils;

pub use bumpalo_alloc::AllocErr;
use create::DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER;

#[cfg(feature = "from_raw_parts")]
mod from_raw_parts;

#[cfg(test)]
mod tests;

/// An arena to allocate into.
///
/// # No `Drop`s
///
/// Objects that are allocated will never have their [`Drop`] implementation called - unless you do it manually
/// yourself. This makes it relatively easy to leak memory or other resources.
///
/// If you have a type which internally manages
///
/// * an allocation from the global heap (e.g. [`Vec<T>`]),
/// * open file descriptors (e.g. [`std::fs::File`]), or
/// * any other resource that must be cleaned up (e.g. an `mmap`)
///
/// and relies on its `Drop` implementation to clean up the internal resource, then if you allocate that
/// type with an `Arena`, you need to find a new way to clean up after it yourself.
///
/// Potential solutions are:
///
/// * Using [`oxc_allocator::Box::new_in`] instead of [`Arena::alloc`], that will drop wrapped values similarly
///   to [`std::boxed::Box`]. This is often the easiest solution.
///
/// * Calling `drop_in_place` or using [`std::mem::ManuallyDrop`] to manually drop these types.
///
/// * Using [`oxc_allocator::Vec`] instead of [`std::vec::Vec`].
///
/// * Avoiding allocating these problematic types within an `Arena`.
///
/// Note that not calling `Drop` is memory safe! Destructors are never guaranteed to run in Rust, you can't rely
/// on them for enforcing memory safety.
///
/// # Example
///
/// ```
/// # use oxc_allocator::arena::Arena;
///
/// // Create a new arena.
/// let arena = Arena::new();
///
/// // Allocate values into the arena.
/// let forty_two = arena.alloc(42);
/// assert_eq!(*forty_two, 42);
///
/// // Mutable references are returned from allocation.
/// let mut n = arena.alloc(123u64);
/// *n = 456;
/// ```
///
/// # Allocation Methods Come in Many Flavors
///
/// There are various allocation methods on [`Arena`], the simplest being [`alloc`]. The others exist to
/// satisfy some combination of fallible allocation and initialization. The allocation methods are summarized
/// in the following table:
///
/// <table>
///   <thead>
///     <tr>
///       <th></th>
///       <th>Infallible Allocation</th>
///       <th>Fallible Allocation</th>
///     </tr>
///   </thead>
///     <tr>
///       <th>By Value</th>
///       <td><a href="#method.alloc"><code>alloc</code></a></td>
///       <td><a href="#method.try_alloc"><code>try_alloc</code></a></td>
///     </tr>
///     <tr>
///       <th>Infallible Initializer Function</th>
///       <td><a href="#method.alloc_with"><code>alloc_with</code></a></td>
///       <td><a href="#method.try_alloc_with"><code>try_alloc_with</code></a></td>
///     </tr>
///   <tbody>
///   </tbody>
/// </table>
///
/// ## Fallible Allocation: The `try_alloc_` Method Prefix
///
/// These allocation methods let you recover from out-of-memory (OOM) scenarios, rather than raising a panic on OOM.
///
/// ```
/// # use oxc_allocator::arena::Arena;
///
/// let arena = Arena::new();
///
/// match arena.try_alloc(MyStruct {
///     // ...
/// }) {
///     Ok(my_struct) => {
///         // Allocation succeeded.
///     }
///     Err(e) => {
///         // Out of memory.
///     }
/// }
///
/// struct MyStruct {
///     // ...
/// }
/// ```
///
/// ## Initializer Functions: The `_with` Method Suffix
///
/// Calling one of the generic `...alloc(x)` methods is essentially equivalent to the matching
/// [`...alloc_with(|| x)`](?search=alloc_with). However if you use `...alloc_with`, then the closure will not be
/// invoked until after allocating space for storing `x` on the heap.
///
/// This can be useful in certain edge-cases related to compiler optimizations. When evaluating for example
/// `arena.alloc(x)`, semantically `x` is first put on the stack and then moved onto the heap. In some cases, the
/// compiler is able to optimize this into constructing `x` directly on the heap, however in many cases it does not.
///
/// The `...alloc_with` functions try to help the compiler be smarter. In most cases doing for example
/// `arena.try_alloc_with(|| x)` on release mode will be enough to help the compiler realize that this
/// optimization is valid and to construct `x` directly onto the heap.
///
/// ### Warning
///
/// These functions critically depend on compiler optimizations to achieve their desired effect.
/// This means that it is not an effective tool when compiling without optimizations on.
///
/// Even when optimizations are on, these functions do not **guarantee** that the value is constructed on the heap.
/// To the best of our knowledge no such guarantee can be made in stable Rust as of 1.54.
///
/// [`Vec<T>`]: std::vec::Vec
/// [`oxc_allocator::Vec`]: crate::Vec
/// [`oxc_allocator::Box::new_in`]: crate::Box::new_in
/// [`alloc`]: Arena::alloc
#[derive(Debug)]
pub struct Arena<const MIN_ALIGN: usize = 1> {
    /// The current chunk we are bump allocating within.
    current_chunk_footer: Cell<NonNull<ChunkFooter>>,
    /// Whether this `Arena` is allowed to allocate additional chunks when the current one is full.
    can_grow: bool,
    /// Used to track number of allocations made in this `Arena` when `track_allocations` feature is enabled.
    #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
    pub(crate) stats: AllocationStats,
}

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Alignment at which all allocations in this arena will be made.
    ///
    /// e.g. if `MIN_ALIGN` is 8, then a `u8` allocated in the arena will be placed at an address
    /// which is a multiple of 8, even though `u8` has no alignment requirements.
    //
    // This constant must be referenced in all code paths which create an `Arena`, in order to validate `MIN_ALIGN`.
    pub const MIN_ALIGN: usize = {
        assert!(MIN_ALIGN.is_power_of_two(), "MIN_ALIGN must be a power of 2");
        assert!(MIN_ALIGN <= CHUNK_ALIGN, "MIN_ALIGN may not be larger than `CHUNK_ALIGN`");
        MIN_ALIGN
    };

    /// Get this arena's minimum alignment.
    ///
    /// All allocations in this arena will be made at an address which is a multiple of this value.
    ///
    /// e.g. if `min_align()` is 8, then a `u8` allocated in the arena will be placed at an address
    /// which is a multiple of 8, even though `u8` has no alignment requirements.
    ///
    /// This value is also available as [`MIN_ALIGN`] constant on the [`Arena`] type itself.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena2 = Arena::<2>::with_min_align();
    /// assert_eq!(arena2.min_align(), 2);
    ///
    /// let arena4 = Arena::<4>::with_min_align();
    /// assert_eq!(arena4.min_align(), 4);
    /// ```
    ///
    /// [`MIN_ALIGN`]: Self::MIN_ALIGN
    #[expect(clippy::unused_self)]
    #[inline(always)]
    pub const fn min_align(&self) -> usize {
        Self::MIN_ALIGN
    }
}

// SAFETY:
// `Arena`s are safe to send between threads because nothing aliases its owned chunks until you start allocating
// from it. But by the time you allocate from it, the returned references to allocations borrow the `Arena` and
// therefore prevent sending the `Arena` across threads until the borrows end.
unsafe impl<const MIN_ALIGN: usize> Send for Arena<MIN_ALIGN> {}

#[repr(C)]
#[repr(align(16))]
#[derive(Debug)]
struct ChunkFooter {
    /// Pointer to the start of this chunk allocation.
    /// This footer is always at the end of the chunk.
    start_ptr: NonNull<u8>,

    /// The layout of this chunk's allocation.
    layout: Layout,

    /// Link to the previous chunk.
    ///
    /// The last node in the `previous_chunk_footer_ptr` linked list is the canonical empty chunk,
    /// whose `previous_chunk_footer_ptr` link points to itself.
    previous_chunk_footer_ptr: Cell<NonNull<ChunkFooter>>,

    /// Bump allocation cursor that is always in the range `self.start_ptr..=self`.
    cursor_ptr: Cell<NonNull<u8>>,
}

/// We only support alignments of up to 16 bytes for `iter_allocated_chunks`.
const SUPPORTED_ITER_ALIGNMENT: usize = 16;
pub const CHUNK_ALIGN: usize = SUPPORTED_ITER_ALIGNMENT;
pub const CHUNK_FOOTER_SIZE: usize = size_of::<ChunkFooter>();

const _: () = assert!(align_of::<ChunkFooter>() == CHUNK_ALIGN);

/// A wrapper type for the canonical, statically allocated empty chunk.
///
/// For the canonical empty chunk to be `static`, its type must be `Sync`, which is the purpose of this wrapper type.
/// This is safe because the empty chunk is immutable and never actually modified.
#[repr(transparent)]
struct EmptyChunkFooter(ChunkFooter);

unsafe impl Sync for EmptyChunkFooter {}

static EMPTY_CHUNK: EmptyChunkFooter = EmptyChunkFooter(ChunkFooter {
    // This chunk is empty (except the foot itself)
    layout: Layout::new::<ChunkFooter>(),

    // The start of the (empty) allocatable region for this chunk is itself
    start_ptr: NonNull::from_ref(&EMPTY_CHUNK).cast::<u8>(),

    // The end of the (empty) allocatable region for this chunk is also itself
    cursor_ptr: Cell::new(NonNull::from_ref(&EMPTY_CHUNK).cast::<u8>()),

    // Invariant: The last chunk footer in all `ChunkFooter::previous_chunk_footer_ptr` linked lists
    // is the empty chunk footer, whose `previous_chunk_footer_ptr` points to itself
    previous_chunk_footer_ptr: Cell::new(NonNull::from_ref(&EMPTY_CHUNK.0)),
});

impl EmptyChunkFooter {
    fn get(&'static self) -> NonNull<ChunkFooter> {
        NonNull::from(&self.0)
    }
}

impl ChunkFooter {
    /// Returns the start and length of the currently allocated region of this chunk.
    fn as_raw_parts(&self) -> (*mut u8, usize) {
        let start_ptr = self.start_ptr.as_ptr().cast_const();
        let cursor_ptr = self.cursor_ptr.get().as_ptr();
        let end_ptr = ptr::from_ref(self).cast::<u8>();
        debug_assert!(start_ptr <= cursor_ptr.cast_const());
        debug_assert!(cursor_ptr.cast_const() <= end_ptr);
        // SAFETY: `cursor_ptr` is always before or equal to `end_ptr`
        let len = unsafe { end_ptr.offset_from_unsigned(cursor_ptr) };
        (cursor_ptr, len)
    }

    /// Returns `true` if this chunk is the empty chunk (end of the linked list).
    fn is_empty(&self) -> bool {
        ptr::eq(self, EMPTY_CHUNK.get().as_ptr())
    }
}
