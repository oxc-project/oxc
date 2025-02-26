// All methods just delegate to `bumpalo`, so all marked `#[inline(always)]`.
// All have same safety preconditions of `bumpalo` methods of the same name.
#![expect(clippy::inline_always, clippy::undocumented_unsafe_blocks)]

use std::{alloc::Layout, ptr::NonNull};

use allocator_api2::alloc::{AllocError, Allocator};

/// SAFETY:
/// <https://github.com/fitzgen/bumpalo/blob/4eeab8847c85d5cde135ca21ae14a54e56b05224/src/lib.rs#L1938>
unsafe impl Allocator for &crate::Allocator {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.bump().allocate(layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.bump().deallocate(ptr, layout);
        }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.bump().shrink(ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.bump().grow(ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.bump().grow_zeroed(ptr, old_layout, new_layout) }
    }
}
