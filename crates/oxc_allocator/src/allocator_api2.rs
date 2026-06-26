// All methods just delegate to `Arena`, so all marked `#[inline(always)]`.
// All have same safety preconditions of `Arena` methods of the same name.
#![expect(clippy::inline_always, clippy::undocumented_unsafe_blocks)]

use std::{alloc::Layout, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};

/// SAFETY: See `arena.rs` for the implementation of `Allocator` for `&Arena`.
unsafe impl Allocator for &crate::Allocator {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Allocator::allocate(&self.arena(), layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            Allocator::deallocate(&self.arena(), ptr, layout);
        }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Allocator::shrink(&self.arena(), ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Allocator::grow(&self.arena(), ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Allocator::grow_zeroed(&self.arena(), ptr, old_layout, new_layout) }
    }
}
