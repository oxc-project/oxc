// All methods just delegate to `Bump`, so all marked `#[inline(always)]`.
// All have same safety preconditions of `Bump` methods of the same name.
#![expect(clippy::inline_always, clippy::undocumented_unsafe_blocks)]

use std::{alloc::Layout, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};

/// SAFETY: See `bump.rs` for the implementation of `Allocator` for `&Bump`.
unsafe impl Allocator for &crate::Allocator {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Allocator::allocate(&self.bump(), layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            Allocator::deallocate(&self.bump(), ptr, layout);
        }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Allocator::shrink(&self.bump(), ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Allocator::grow(&self.bump(), ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { Allocator::grow_zeroed(&self.bump(), ptr, old_layout, new_layout) }
    }
}
