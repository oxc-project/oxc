use std::{alloc::Layout, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};

/// SAFETY:
/// <https://github.com/fitzgen/bumpalo/blob/4eeab8847c85d5cde135ca21ae14a54e56b05224/src/lib.rs#L1938>
unsafe impl Allocator for &crate::Allocator {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        (&self.bump).allocate(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        (&self.bump).deallocate(ptr, layout);
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        (&self.bump).shrink(ptr, old_layout, new_layout)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        (&self.bump).grow(ptr, old_layout, new_layout)
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        (&self.bump).grow_zeroed(ptr, old_layout, new_layout)
    }
}
