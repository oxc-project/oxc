//! Define additional methods, used only by raw transfer:
//!
//! * [`Allocator::from_raw_parts`]
//! * [`Allocator::set_cursor_ptr`]
//! * [`Allocator::data_end_ptr`]
//! * [`Allocator::end_ptr`]

use std::{
    alloc::Layout,
    cell::Cell,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

use crate::{Allocator, bump::Bump};

/// Minimum alignment for allocator chunks.
const MIN_ALIGN: usize = 16;

const CHUNK_FOOTER_SIZE: usize = size_of::<ChunkFooter>();
const _: () = {
    // Check the hard-coded value in `ast_tools` raw transfer generator is accurate.
    // We can only do this check if we're on a 64-bit little-endian platform with the `fixed_size` feature enabled,
    // because the `fixed_size_constants` module is only compiled under those conditions.
    // That's good enough, as the size of `ChunkFooter` only matters in that case anyway (Oxlint JS plugins).
    #[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
    {
        use crate::generated::fixed_size_constants::CHUNK_FOOTER_SIZE as EXPECTED_CHUNK_FOOTER_SIZE;
        assert!(CHUNK_FOOTER_SIZE == EXPECTED_CHUNK_FOOTER_SIZE);
    }

    // Check alignment requirements
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
    /// This code relies on specific internal layout of the arena allocator.
    /// Changes to `Bump` internals may break this code.
    ///
    /// The [`Allocator`] which is returned takes ownership of the memory allocation,
    /// and the allocation will be freed if the `Allocator` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Allocator` in `ManuallyDrop`.
    ///
    /// The [`Allocator`] returned by this function cannot grow.
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
    /// Panics if cannot determine layout of `Bump` type, or on a big endian system.
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
        debug_assert!((ptr.as_ptr() as usize).is_multiple_of(MIN_ALIGN));
        debug_assert!(size.is_multiple_of(MIN_ALIGN));
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

    /// Set cursor pointer for this [`Allocator`]'s current chunk.
    ///
    /// This is dangerous, and this method should not ordinarily be used.
    /// It is only here for manually resetting the allocator.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    ///   It is UB to call this method on an `Allocator` which has not allocated
    ///   i.e. fresh from `Allocator::new`.
    /// * `ptr` must point to within the `Allocator`'s current chunk.
    /// * `ptr` must be equal to or after data pointer for this chunk.
    pub unsafe fn set_cursor_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees `Allocator` has at least 1 allocated chunk.
        // We don't take any action with the `Allocator` while the `&mut ChunkFooter` reference
        // is alive, beyond setting the cursor pointer.
        let chunk_footer = unsafe { self.chunk_footer_mut() };
        chunk_footer.ptr.set(ptr);
    }

    /// Get pointer to end of the data region of this [`Allocator`]'s current chunk
    /// i.e to the start of the `ChunkFooter`.
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        self.chunk_footer_ptr().cast::<u8>()
    }

    /// Get pointer to end of this [`Allocator`]'s current chunk (after the `ChunkFooter`).
    pub fn end_ptr(&self) -> NonNull<u8> {
        // SAFETY: `chunk_footer_ptr` returns pointer to a valid `ChunkFooter`,
        // so stepping past it cannot be out of bounds of the chunk's allocation.
        // If `Allocator` has not allocated, so `chunk_footer_ptr` returns a pointer to the static
        // empty chunk, it's still valid.
        unsafe { self.chunk_footer_ptr().add(1).cast::<u8>() }
    }

    /// Get mutable reference to current [`ChunkFooter`].
    ///
    /// It would be safer if this method took a `&mut self`, but that would preclude using this method
    /// while any references to data in the arena exist, which is too restrictive.
    /// So we just need to be careful how we use this method.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    ///   It is UB to call this method on an `Allocator` which has not allocated
    ///   i.e. fresh from `Allocator::new`.
    /// * Caller must not allocate into this `Allocator`, or perform any other action which would
    ///   read or alter the `ChunkFooter`, or create another reference to it, while the `&mut ChunkFooter`
    ///   reference returned by this method is alive.
    #[expect(clippy::mut_from_ref)]
    unsafe fn chunk_footer_mut(&self) -> &mut ChunkFooter {
        let mut chunk_footer_ptr = self.chunk_footer_ptr();
        // SAFETY: Caller guarantees `Allocator` has an allocated chunk, so this isn't `EmptyChunkFooter`,
        // which it'd be UB to obtain a mutable reference to.
        // Caller promises not to take any other action which would generate another reference to the
        // `ChunkFooter` while this reference is alive.
        unsafe { chunk_footer_ptr.as_mut() }
    }

    /// Get pointer to current chunk's [`ChunkFooter`].
    fn chunk_footer_ptr(&self) -> NonNull<ChunkFooter> {
        let current_chunk_footer_field_offset = get_current_chunk_footer_field_offset();

        // Get pointer to current `ChunkFooter`.
        // SAFETY: We've established the offset of the `current_chunk_footer` field above.
        let current_chunk_footer_field = unsafe {
            let bump = self.bump();
            let field_ptr = ptr::from_ref(bump)
                .cast::<Cell<NonNull<ChunkFooter>>>()
                .add(current_chunk_footer_field_offset);
            &*field_ptr
        };
        current_chunk_footer_field.get()
    }
}

/// Allocator chunk footer.
///
/// This type must match the layout of `ChunkFooter` in `bump.rs`.
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

    let bump = ManuallyDrop::new(Bump::new());
    bump.set_allocation_limit(Some(123));

    // SAFETY:
    // `Bump` has same layout as `[usize; 3]` (checked by const assertions above).
    // Strictly speaking, reading the fields as `usize`s is UB, as the layout of `Option`
    // is not specified. But in practice, `Option` stores its discriminant before its payload,
    // so either field order means 3rd `usize` is fully initialized
    // (it's either `NonNull<ChunkFooter>>` or the `usize` in `Option<usize>`).
    unsafe {
        let ptr = ptr::from_ref::<ManuallyDrop<Bump>>(&bump).cast::<usize>();
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
