//! Arena allocator.
//!
//! The files in this directory were originally derived from `bumpalo` at commit
//! a47f6d6b7b5fee9c99a285f0de80257a0a982ef3 (2 commits after 3.20.2 release).
//! Changes have been made since.

#![expect(clippy::inline_always, clippy::undocumented_unsafe_blocks)]

use std::{alloc::Layout, cell::Cell, ptr::NonNull};

#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
use crate::tracking::AllocationStats;

mod alloc;
mod alloc_impl;
mod bumpalo_alloc;
mod chunks;
mod create;
mod drop;
mod utils;

#[cfg(feature = "testing")]
pub use bumpalo_alloc::AllocErr;
use create::DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER;
#[cfg(feature = "fixed_size")]
pub(crate) use drop::dealloc_arena_chunk;

#[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
mod fixed_size;
#[cfg(feature = "from_raw_parts")]
mod from_raw_parts;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_alloc;

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
//
// `#[repr(C)]` plus deliberate field ordering to defeat a store-to-load forwarding hazard on aarch64.
// The fast path reads `cursor_ptr` and `start_ptr`, and writes `cursor_ptr` on every allocation.
// If `cursor_ptr` and `start_ptr` were adjacent (offsets 0 and 8), LLVM's aarch64 backend fuses them
// into a single 16-byte `ldp` instruction. That `ldp` then partial-overlaps the 8-byte `cursor_ptr` store
// from the previous iteration, which breaks store-to-load forwarding and causes a ~3x slowdown in tight allocation loops.
// `current_chunk_footer_ptr` is placed between the two hot pointers so they sit at offsets 0 and 16,
// forcing LLVM to emit two independent 8-byte `ldr`s, each of which forwards cleanly.
// More background here: https://eme64.github.io/blog/2024/06/24/Auto-Vectorization-and-Store-to-Load-Forwarding.html
#[cfg_attr(
    not(all(feature = "track_allocations", not(feature = "disable_track_allocations"))),
    expect(clippy::struct_field_names)
)]
#[repr(C)]
#[derive(Debug)]
pub struct Arena<const MIN_ALIGN: usize = 1> {
    /// Bump allocation cursor.
    ///
    /// `Arena` bumps downwards, so this is pointer to the start of the last allocated object in the current chunk,
    /// or to the current chunk's `ChunkFooter` if nothing has been allocated in the current chunk yet.
    ///
    /// When an object is allocated in the arena, the cursor is bumped downwards (towards `start_ptr`),
    /// and the object is written at the new cursor position.
    ///
    /// * When arena is empty, and owns no chunks (`current_chunk_footer_ptr` is `None`):
    ///   This is set to [`EMPTY_ARENA_DATA_PTR`] (same as `start_ptr`).
    /// * When arena owns at least one chunk (`current_chunk_footer_ptr` is `Some`):
    ///   `cursor_ptr` is always in the range `self.start_ptr..=self.current_chunk_footer_ptr`.
    ///
    /// This field is duplicated in `ChunkFooter`, but the pointer here is authoritative for current chunk.
    cursor_ptr: Cell<NonNull<u8>>,

    /// Pointer to footer of current chunk we are bump allocating within.
    ///
    /// `None` if `Arena` is empty (owns no chunks).
    current_chunk_footer_ptr: Cell<Option<NonNull<ChunkFooter>>>,

    /// Pointer to the start of the current chunk's allocatable region.
    ///
    /// When arena owns no chunks (`current_chunk_footer_ptr` is `None`), this is set to [`EMPTY_ARENA_DATA_PTR`]
    /// (equal to `cursor_ptr`).
    ///
    /// This field is duplicated in `ChunkFooter`.
    start_ptr: Cell<NonNull<u8>>,

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

/// Footer containing details about a chunk of memory owned by an `Arena`.
///
/// This footer is written into the allocation at its very end.
/// The chunk's bump cursor initially points to the `ChunkFooter`.
/// i.e. The memory region which can be allocated into is `start_ptr..cursor_ptr`.
///
/// Chunks form a linked list with `Arena::current_chunk_footer_ptr` pointing to current chunk's footer,
/// and each chunk's `ChunkFooter::previous_chunk_footer_ptr` pointing to the previous chunk's footer.
/// The list ends with `previous_chunk_footer_ptr` of the oldest chunk being `None`.
///
/// An empty `Arena`, which does not own any memory, has `Arena::current_chunk_footer_ptr` set to `None`.
#[repr(C, align(16))]
#[derive(Debug)]
pub(crate) struct ChunkFooter {
    /// Pointer to the start of the allocation backing this chunk.
    ///
    /// This pointer is passed to `alloc::dealloc` (or `System.dealloc` if `is_fixed_size` is `true`)
    /// when deallocating the chunk.
    backing_alloc_ptr: NonNull<u8>,

    /// The layout of this chunk's backing allocation.
    layout: Layout,

    /// Link to the previous chunk.
    ///
    /// `None` for the oldest chunk in the linked list.
    previous_chunk_footer_ptr: Cell<Option<NonNull<ChunkFooter>>>,

    /// Bump allocation cursor, valid only for retired (non-current) chunks.
    /// This field is written when a chunk is retired (when a new chunk is created).
    /// Allocation methods use `Arena::cursor_ptr` instead, which is the authoritative pointer for current chunk.
    /// This field is only used in `ChunkIter` and `ChunkRawIter` iterators, and `used_bytes` method.
    cursor_ptr: Cell<NonNull<u8>>,

    /// `true` if backing allocation was made via [`System`] allocator (rather than the global allocator).
    ///
    /// `Arena`'s [`Drop`] impl uses this to know whether to free the backing allocation via [`System`]
    /// or the global allocator.
    ///
    /// Set to `true` for chunks created via [`Arena::from_raw_parts`], `false` otherwise.
    ///
    /// [`System`]: std::alloc::System
    is_fixed_size: bool,
}

/// We only support alignments of up to 16 bytes for `iter_allocated_chunks`.
const SUPPORTED_ITER_ALIGNMENT: usize = 16;
pub const CHUNK_ALIGN: usize = SUPPORTED_ITER_ALIGNMENT;
pub const CHUNK_FOOTER_SIZE: usize = size_of::<ChunkFooter>();

const _: () = assert!(align_of::<ChunkFooter>() == CHUNK_ALIGN);

/// Sentinel value used for `Arena::start_ptr` and `Arena::cursor_ptr` when the arena owns no chunks.
///
/// This is a dangling, non-null pointer aligned to `CHUNK_ALIGN`, which is `>= MIN_ALIGN` for any valid
/// `MIN_ALIGN`. So the cursor satisfies the `Arena` invariant of being aligned to `MIN_ALIGN`.
///
/// Setting `cursor_ptr == start_ptr == EMPTY_ARENA_DATA_PTR` gives a chunk with zero capacity, so any
/// non-zero-sized allocation fails the fast path bounds check and falls through to the slow path
/// (which allocates a new chunk).
const EMPTY_ARENA_DATA_PTR: NonNull<u8> = NonNull::<ChunkFooter>::dangling().cast::<u8>();
