// All methods just delegate to `Arena`, so all marked `#[inline(always)]`.
// All have same safety preconditions of `Arena` methods of the same name.
#![expect(clippy::inline_always, clippy::undocumented_unsafe_blocks)]

use std::{alloc::Layout, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};

/// SAFETY: See `Arena` impl of `Allocator`.
unsafe impl Allocator for &crate::Allocator {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.arena().allocate(layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.arena().deallocate(ptr, layout);
        }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.arena().shrink(ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.arena().grow(ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.arena().grow_zeroed(ptr, old_layout, new_layout) }
    }
}
