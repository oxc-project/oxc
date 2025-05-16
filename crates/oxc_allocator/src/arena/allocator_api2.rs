// Allocation methods are small and should always be inlined
#![expect(clippy::inline_always)]

use std::{
    alloc::Layout,
    ptr::{self, NonNull},
};

use allocator_api2::alloc::{AllocError, Allocator};

use crate::alloc::Alloc;

use super::{Arena, config::ArenaConfigExt};

/// SAFETY:
/// <https://github.com/fitzgen/bumpalo/blob/4eeab8847c85d5cde135ca21ae14a54e56b05224/src/lib.rs#L1938>
unsafe impl<Config: ArenaConfigExt> Allocator for Arena<Config> {
    /// TODO: Docs
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.alloc_layout(layout);
        // SAFETY: TODO
        let slice_ptr = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), layout.size()))
        };

        Ok(slice_ptr)
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    #[expect(unused_variables)]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // No-op. We don't attempt to reclaim memory.
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    #[expect(unused_variables)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // We don't attempt to reclaim memory
        // SAFETY: TODO
        let slice_ptr = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), new_layout.size()))
        };
        Ok(slice_ptr)
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // Allocate new layout
        // SAFETY: TODO
        let new_ptr = unsafe { Alloc::grow(self, ptr, old_layout, new_layout) };

        // SAFETY: TODO
        let slice_ptr = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                new_ptr.as_ptr(),
                new_layout.size(),
            ))
        };
        Ok(slice_ptr)
    }

    /// # SAFETY
    /// TODO
    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: TODO
        let new_ptr = unsafe { Alloc::grow(self, ptr, old_layout, new_layout) };

        // Zero the new section
        // SAFETY: TODO
        unsafe {
            let extension_start_ptr = new_ptr.as_ptr().add(old_layout.size());
            ptr::write_bytes(extension_start_ptr, 0, new_layout.size() - old_layout.size());
        }

        // SAFETY: TODO
        let slice_ptr = unsafe {
            NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(
                new_ptr.as_ptr(),
                new_layout.size(),
            ))
        };
        Ok(slice_ptr)
    }
}
