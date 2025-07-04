//! Define additional [`Allocator::from_raw_parts`] method, used only by raw transfer.

use std::{
    alloc::Layout,
    cell::Cell,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

use bumpalo::Bump;

use crate::Allocator;

/// Minimum alignment for allocator chunks. This is hard-coded on `bumpalo`.
const MIN_ALIGN: usize = 16;

const CHUNK_FOOTER_SIZE: usize = size_of::<ChunkFooter>();
const _: () = {
    assert!(CHUNK_FOOTER_SIZE >= MIN_ALIGN);
    assert!(align_of::<ChunkFooter>() <= MIN_ALIGN);
};

impl Allocator {
    /// Minimum size for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_SIZE: usize = CHUNK_FOOTER_SIZE;

    /// Minimum alignment for memory chunk passed to [`Allocator::from_raw_parts`].
    pub const RAW_MIN_ALIGN: usize = MIN_ALIGN;

    /// Construct a static-sized [`Allocator`] from an existing memory allocation.
    ///
    /// **IMPORTANT: WE MUST NOT CHANGE THE VERSION OF BUMPALO DEPENDENCY**.
    ///
    /// This code only remains sound as long as the code in version of `bumpalo` we're using matches
    /// the duplicate of `bumpalo`'s internals contained in this file.
    ///
    /// `bumpalo` is pinned to version `=3.19.0` in `Cargo.toml`.
    ///
    /// The [`Allocator`] which is returned takes ownership of the memory allocation,
    /// and the allocation will be freed if the `Allocator` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Allocator` in `ManuallyDrop`.
    ///
    /// The [`Allocator`] returned by this function cannot grow.
    ///
    /// This hack is all very inadvisable!
    /// Only implemented as a temporary stopgap until we replace `bumpalo` with our own allocator.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be aligned on [`RAW_MIN_ALIGN`].
    /// * `size` must be a multiple of [`RAW_MIN_ALIGN`].
    /// * `size` must be at least [`RAW_MIN_SIZE`].
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within
    ///   a single allocation.
    ///
    /// # Panics
    ///
    /// Panics if cannot determine layout of Bumpalo's `Bump` type, or on a big endian system.
    ///
    /// [`RAW_MIN_ALIGN`]: Self::RAW_MIN_ALIGN
    /// [`RAW_MIN_SIZE`]: Self::RAW_MIN_SIZE
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        // Only support little-endian systems.
        // Calculating offset of `current_chunk_footer` on big-endian systems would be difficult.
        #[expect(clippy::manual_assert)]
        if cfg!(target_endian = "big") {
            panic!("`Allocator::from_raw_parts` is not supported on big-endian systems.");
        }

        // Debug assert that `ptr` and `size` fulfill size and alignment requirements
        debug_assert!(is_multiple_of(ptr.as_ptr() as usize, MIN_ALIGN));
        debug_assert!(is_multiple_of(size, MIN_ALIGN));
        debug_assert!(size >= CHUNK_FOOTER_SIZE);

        let current_chunk_footer_field_offset = get_current_chunk_footer_field_offset();

        // Create empty bump with allocation limit of 0 - i.e. it cannot grow.
        // This means that the memory chunk we're about to add to the `Bump` will remain its only chunk.
        // Therefore it can never be deallocated, until the `Allocator` is dropped.
        // `Allocator::reset` would only reset the "cursor" pointer, not deallocate the memory.
        let bump = Bump::new();
        bump.set_allocation_limit(Some(0));

        // Get pointer to `EmptyChunkFooter`.
        // SAFETY: We've established the offset of the `current_chunk_footer` field above.
        let current_chunk_footer_field = unsafe {
            let field_ptr = ptr::addr_of!(bump)
                .cast::<Cell<NonNull<ChunkFooter>>>()
                .add(current_chunk_footer_field_offset);
            &*field_ptr
        };
        let empty_chunk_footer_ptr = current_chunk_footer_field.get();

        // Construct `ChunkFooter` and write into end of allocation.
        // SAFETY: Caller guarantees:
        // 1. `ptr` is the start of an allocation of `size` bytes.
        // 2. `size` is `>= CHUNK_FOOTER_SIZE` - so `size - CHUNK_FOOTER_SIZE` cannot wrap around.
        let chunk_footer_ptr = unsafe { ptr.add(size - CHUNK_FOOTER_SIZE) };
        // SAFETY: Caller guarantees `size` is a multiple of 16
        let layout = unsafe { Layout::from_size_align_unchecked(size, 16) };
        let chunk_footer = ChunkFooter {
            data: ptr,
            layout,
            prev: Cell::new(empty_chunk_footer_ptr),
            ptr: Cell::new(chunk_footer_ptr),
            allocated_bytes: 0,
        };
        let chunk_footer_ptr = chunk_footer_ptr.cast::<ChunkFooter>();
        // SAFETY: If caller has upheld safety requirements, `chunk_footer_ptr` is `CHUNK_FOOTER_SIZE`
        // bytes from the end of the allocation, and aligned on 16.
        // Const assertions at top of this file ensure that is sufficient alignment for `ChunkFooter`.
        // Therefore `chunk_footer_ptr` is valid for writing a `ChunkFooter`.
        unsafe { chunk_footer_ptr.write(chunk_footer) };

        // Write chunk header into bump's `chunk_header` field
        current_chunk_footer_field.set(chunk_footer_ptr);

        Self::from_bump(bump)
    }
}

/// Allocator chunk footer.
///
/// Copied exactly from `bumpalo` v3.19.0.
///
/// This type is not exposed by `bumpalo` crate, but the type is `#[repr(C)]`, so we can rely on our
/// duplicate here having the same layout, as long as we don't change the version of `bumpalo` we use.
#[repr(C)]
#[derive(Debug)]
struct ChunkFooter {
    /// Pointer to the start of this chunk allocation.
    /// This footer is always at the end of the chunk.
    data: NonNull<u8>,

    /// The layout of this chunk's allocation.
    layout: Layout,

    /// Link to the previous chunk.
    ///
    /// Note that the last node in the `prev` linked list is the canonical empty
    /// chunk, whose `prev` link points to itself.
    prev: Cell<NonNull<ChunkFooter>>,

    /// Bump allocation finger that is always in the range `self.data..=self`.
    ptr: Cell<NonNull<u8>>,

    /// The bytes allocated in all chunks so far.
    /// The canonical empty chunk has a size of 0 and for all other chunks, `allocated_bytes` will be
    /// the allocated_bytes of the current chunk plus the allocated bytes of the `prev` chunk.
    allocated_bytes: usize,
}

/// Get offset of `current_chunk_footer` field in `Bump`, in units of `usize`.
///
/// `Bump` is defined as:
///
/// ```ignore
/// pub struct Bump {
///     current_chunk_footer: Cell<NonNull<ChunkFooter>>,
///     allocation_limit: Cell<Option<usize>>,
/// }
/// ```
///
/// `Bump` is not `#[repr(C)]`, so which order the fields are in is unpredictable.
/// Deduce the offset of `current_chunk_footer` field by creating a dummy `Bump` where the value
/// of the `allocation_limit` field is known.
///
/// This should all be const-folded down by compiler.
/// <https://godbolt.org/z/eKdMcdEYa>
/// `#[inline(always)]` because this is essentially a const function.
#[expect(clippy::inline_always)]
#[inline(always)]
fn get_current_chunk_footer_field_offset() -> usize {
    const {
        assert!(size_of::<Bump>() == size_of::<[usize; 3]>());
        assert!(align_of::<Bump>() == align_of::<[usize; 3]>());
        assert!(size_of::<Cell<NonNull<ChunkFooter>>>() == size_of::<usize>());
        assert!(align_of::<Cell<NonNull<ChunkFooter>>>() == align_of::<usize>());
        assert!(size_of::<Cell<Option<usize>>>() == size_of::<[usize; 2]>());
        assert!(align_of::<Cell<Option<usize>>>() == align_of::<usize>());
    }

    let bump = ManuallyDrop::new(Bump::<1>::with_min_align());
    bump.set_allocation_limit(Some(123));

    // SAFETY:
    // `Bump` has same layout as `[usize; 3]` (checked by const assertions above).
    // Strictly speaking, reading the fields as `usize`s is UB, as the layout of `Option`
    // is not specified. But in practice, `Option` stores its discriminant before its payload,
    // so either field order means 3rd `usize` is fully initialized
    // (it's either `NonNull<ChunkFooter>>` or the `usize` in `Option<usize>`).
    unsafe {
        let ptr = ptr::from_ref(&bump).cast::<usize>();
        if *ptr.add(2) == 123 {
            // `allocation_limit` is 2nd field. So `current_chunk_footer` is 1st.
            0
        } else {
            // `allocation_limit` is 1st field. So `current_chunk_footer` is 2nd.
            assert_eq!(*ptr.add(1), 123);
            2
        }
    }
}

/// Returns `true` if `n` is a multiple of `divisor`.
const fn is_multiple_of(n: usize, divisor: usize) -> bool {
    n % divisor == 0
}
