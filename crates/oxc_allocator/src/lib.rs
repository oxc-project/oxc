use std::{
    alloc::Layout,
    cell::Cell,
    convert::From,
    mem::{size_of, transmute},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

mod arena;

pub use arena::{Box, String, Vec};
use bumpalo::Bump;
use static_assertions::assert_eq_size;

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    pub const MIN_SIZE: usize = max(size_of::<ChunkFooter>(), 16);

    /// Construct a static-sized `Allocator` from an existing memory allocation.
    ///
    /// This is unsafe and inadvisable. Only implemented as a temporary stopgap for testing.
    ///
    /// The `Allocator` takes ownership of the memory allocation, and the allocation will be freed
    /// if the `Allocator` is dropped.
    /// If caller wishes to prevent that happening, should wrap the `Allocator` in `ManuallyDrop`.
    ///
    /// The `Allocator` returned by this function cannot grow.
    ///
    /// # SAFETY
    /// * `ptr` must be aligned on 16.
    /// * `size` must be a multiple of 16.
    /// * `size` must be at least `Self::MIN_SIZE`.
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within
    ///   a single allocation.
    ///
    /// # Panics
    /// Panics if cannot determine layout of Bumpalo's `Bump` type.
    #[allow(unsafe_code, clippy::items_after_statements, clippy::missing_safety_doc)]
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        // Get position of chunk footer pointer in `Bump`.
        // This should be const-folded by compiler.
        let chunk_footer_offset: usize = {
            assert_eq_size!(Cell<NonNull<ChunkFooter>>, usize);

            let bump = Bump::new();
            bump.set_allocation_limit(Some(123));
            let parts: [usize; 3] = transmute(bump);
            if parts[2] == 123 {
                assert_eq!(parts[1], 1);
                assert_ne!(parts[0], 0);
                0
            } else {
                assert_eq!(parts[0], 1);
                assert_eq!(parts[1], 123);
                assert_ne!(parts[2], 0);
                2
            }
        };

        // Create empty bump
        let mut bump = Bump::new();
        bump.set_allocation_limit(Some(0));

        // Get pointer to `EmptyChunkFooter`
        let chunk_footer_field = &mut *(std::ptr::addr_of_mut!(bump)
            .cast::<Cell<NonNull<ChunkFooter>>>())
        .add(chunk_footer_offset);
        let empty_chunk_footer_ptr = chunk_footer_field.get();

        const CHUNK_FOOTER_SIZE: usize = size_of::<ChunkFooter>();
        debug_assert_eq!(ptr.as_ptr() as usize % 16, 0);
        debug_assert_eq!(size % 16, 0);
        debug_assert!(size >= Self::MIN_SIZE);

        // Construct `ChunkFooter` and write into end of allocation
        let chunk_footer_ptr = ptr.as_ptr().add(size - CHUNK_FOOTER_SIZE);
        let chunk_footer = ChunkFooter {
            data: ptr,
            layout: Layout::from_size_align_unchecked(size, 16),
            prev: Cell::new(empty_chunk_footer_ptr),
            ptr: Cell::new(NonNull::new_unchecked(chunk_footer_ptr)),
            allocated_bytes: 0,
        };
        #[allow(clippy::cast_ptr_alignment)]
        let chunk_footer_ptr = chunk_footer_ptr.cast::<ChunkFooter>();
        chunk_footer_ptr.write(chunk_footer);

        // Write chunk header into bump's `chunk_header` field
        chunk_footer_field.set(NonNull::new_unchecked(chunk_footer_ptr));

        Self { bump }
    }

    /// Set allocation limit.
    ///
    /// This is disabled as it would allow changing allocation limit from 0 for `Allocator`s
    /// created by `Allocator::from_raw_parts`, which would allow them to add another chunk.
    /// If `Allocator::reset` was then called, it would free the original chunk, which would
    /// deallocate the memory. That must not be allowed to happen as memory may be borrowed,
    /// in which case it could lead to a use-after-free.
    ///
    /// This method is only present to block access to `Bump`'s method of same name via deref.
    /// NB: User could still deref manually and then call the method on `Bump` directly,
    /// but this at least makes it harder to do.
    ///
    /// # Panics
    /// Always panics!
    pub fn set_allocation_limit(&self, _limit: Option<usize>) {
        panic!("set_allocation_limit is not supported");
    }
}

impl From<Bump> for Allocator {
    fn from(bump: Bump) -> Self {
        Self { bump }
    }
}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}

impl DerefMut for Allocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bump
    }
}

#[repr(C)]
#[derive(Debug)]
struct ChunkFooter {
    // Pointer to the start of this chunk allocation. This footer is always at
    // the end of the chunk.
    data: NonNull<u8>,

    // The layout of this chunk's allocation.
    layout: Layout,

    // Link to the previous chunk.
    //
    // Note that the last node in the `prev` linked list is the canonical empty
    // chunk, whose `prev` link points to itself.
    prev: Cell<NonNull<ChunkFooter>>,

    // Bump allocation finger that is always in the range `self.data..=self`.
    ptr: Cell<NonNull<u8>>,

    // The bytes allocated in all chunks so far, the canonical empty chunk has
    // a size of 0 and for all other chunks, `allocated_bytes` will be
    // the allocated_bytes of the current chunk plus the allocated bytes
    // of the `prev` chunk.
    allocated_bytes: usize,
}

const fn max(n1: usize, n2: usize) -> usize {
    if n1 > n2 {
        n1
    } else {
        n2
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::Allocator;
    use bumpalo::Bump;

    #[test]
    fn test_api() {
        let bump = Bump::new();
        let allocator: Allocator = bump.into();
        #[allow(clippy::explicit_deref_methods)]
        {
            _ = allocator.deref();
        }
    }
}
